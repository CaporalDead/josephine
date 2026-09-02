use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::checks::{build_checks, interval_for_check};
use crate::config::{CheckThresholds, Config};
use crate::notify;
use crate::paths::Paths;
use crate::rules::RulesEngine;
use crate::storage::Storage;

pub struct Scheduler {
    config: Config,
    paths: Paths,
    storage: Storage,
    rules: RulesEngine,
}

impl Scheduler {
    pub fn new(config: Config, paths: Paths) -> Result<Self> {
        let storage = Storage::open(&paths)?;
        Ok(Self {
            config,
            paths,
            storage,
            rules: RulesEngine::new(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Joséphine démarre la surveillance");

        let checks = build_checks(&self.config.checks);
        if checks.is_empty() {
            warn!("Aucun check activé — le démon reste inactif");
            tokio::signal::ctrl_c().await?;
            return Ok(());
        }

        let engine = Arc::new(Mutex::new(std::mem::take(&mut self.rules)));
        let storage = Arc::new(Mutex::new(Storage::open(&self.paths)?));
        let config = self.config.clone();
        let retention = self.config.history.retention_days;
        let desktop_notify = self.config.notifications.desktop;
        let terminal_notify = self.config.notifications.terminal;

        let mut handles = Vec::new();

        for mut check in checks {
            let check_name = check.name().to_string();
            let interval = interval_for_check(&check_name, &config.checks);
            let thresholds = thresholds_for(&check_name, &config);
            let lang = config.language;
            let engine = Arc::clone(&engine);
            let storage = Arc::clone(&storage);

            handles.push(tokio::spawn(async move {
                loop {
                    let start = Instant::now();
                    let result = check.run();

                    match result {
                        Ok(result) => {
                            let metrics: Vec<(String, f64)> = result
                                .metrics
                                .iter()
                                .map(|m| (m.name.clone(), m.value))
                                .collect();

                            {
                                let store = storage.lock().await;
                                let _ = store.insert_metrics(&check_name, &metrics);
                                let _ = store.log_check_run(
                                    &check_name,
                                    true,
                                    start.elapsed().as_millis() as u64,
                                    None,
                                );
                            }

                            {
                                let mut rules = engine.lock().await;
                                let transitions = rules.evaluate_check(
                                    &check_name,
                                    &result.metrics,
                                    &thresholds,
                                    lang,
                                );

                                for transition in transitions {
                                    let store = storage.lock().await;
                                    if let Ok(event_id) = store.insert_event(&transition)
                                        && transition.notify
                                    {
                                        // Terminal channel: surface the alert in the daemon's
                                        // log (journal / foreground `daemon run`).
                                        if terminal_notify {
                                            warn!(target: "josephine::alert", "{}", transition.message);
                                            let _ = store.insert_notification(event_id, "terminal");
                                        }
                                        // Desktop channel: libnotify.
                                        if desktop_notify {
                                            if let Err(e) =
                                                notify::send_josephine(&transition.message)
                                            {
                                                error!("notification : {e}");
                                            } else {
                                                let _ =
                                                    store.insert_notification(event_id, "desktop");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("check {check_name} : {e}");
                            let store = storage.lock().await;
                            let _ = store.log_check_run(
                                &check_name,
                                false,
                                start.elapsed().as_millis() as u64,
                                Some(&e.to_string()),
                            );
                        }
                    }

                    tokio::time::sleep(Duration::from_secs(interval)).await;
                }
            }));
        }

        let storage_purge = Arc::clone(&storage);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
                let store = storage_purge.lock().await;
                let _ = store.purge_older_than(retention);
            }
        });

        tokio::signal::ctrl_c().await?;
        info!("Arrêt demandé — Joséphine se repose");

        for handle in handles {
            handle.abort();
        }

        Ok(())
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }
}

fn thresholds_for(name: &str, config: &Config) -> CheckThresholds {
    match name {
        "cpu" => config.checks.cpu.clone(),
        "memory" => config.checks.memory.clone(),
        "disk" => config.checks.disk.clone(),
        "temperature" => CheckThresholds {
            enabled: config.checks.temperature.enabled,
            interval_secs: config.checks.temperature.interval_secs,
            warning: config.checks.temperature.warning,
            critical: config.checks.temperature.critical,
        },
        "systemd" => CheckThresholds {
            enabled: config.checks.systemd.enabled,
            interval_secs: config.checks.systemd.interval_secs,
            warning: config.checks.systemd.failed_warning,
            critical: config.checks.systemd.failed_critical,
        },
        "kernel" => CheckThresholds {
            enabled: config.checks.kernel.enabled,
            interval_secs: config.checks.kernel.interval_secs,
            warning: config.checks.kernel.warning,
            critical: config.checks.kernel.critical,
        },
        "filesystem" => CheckThresholds {
            enabled: config.checks.filesystem.enabled,
            interval_secs: config.checks.filesystem.interval_secs,
            warning: config.checks.filesystem.warning,
            critical: config.checks.filesystem.critical,
        },
        "timesync" => CheckThresholds {
            enabled: config.checks.timesync.enabled,
            interval_secs: config.checks.timesync.interval_secs,
            warning: config.checks.timesync.warning,
            critical: config.checks.timesync.critical,
        },
        "security" => CheckThresholds {
            enabled: config.checks.security.enabled,
            interval_secs: config.checks.security.interval_secs,
            warning: config.checks.security.warning,
            critical: config.checks.security.critical,
        },
        "reboot" => CheckThresholds {
            enabled: config.checks.reboot.enabled,
            interval_secs: config.checks.reboot.interval_secs,
            warning: config.checks.reboot.warning,
            critical: config.checks.reboot.critical,
        },
        _ => CheckThresholds::default(),
    }
}

/// Exécute tous les checks une fois (pour status/doctor CLI).
pub fn run_all_checks(config: &Config) -> Result<Vec<crate::check::CheckResult>> {
    let mut checks = build_checks(&config.checks);
    let mut results = Vec::new();
    for check in checks.iter_mut() {
        results.push(check.run()?);
    }
    Ok(results)
}
