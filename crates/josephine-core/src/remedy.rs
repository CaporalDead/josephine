//! What to do about a check that came back in default.
//!
//! Single source of truth for remedies: `explain` renders the static
//! [`Advice`] copy, `doctor` renders [`Remedy`] values derived from a live
//! [`CheckResult`]. No remedy ever points back at `josephine doctor` — inside
//! doctor that would be circular.

use crate::check::{CheckResult, Severity};
use crate::i18n::{self, Lang};

/// Static per-check copy, independent of any run. Each field is an (EN, FR) pair.
pub struct Advice {
    pub name: &'static str,
    pub what: (&'static str, &'static str),
    pub why: (&'static str, &'static str),
    pub remedy: (&'static str, &'static str),
}

impl Advice {
    /// The "what" line, in the current language.
    pub fn what(&self) -> &'static str {
        i18n::t(self.what.0, self.what.1)
    }

    /// The "why" line, in the current language.
    pub fn why(&self) -> &'static str {
        i18n::t(self.why.0, self.why.1)
    }

    /// The remedy line, in the current language.
    pub fn remedy(&self) -> &'static str {
        i18n::t(self.remedy.0, self.remedy.1)
    }
}

/// The checks, in the order `explain` lists them.
const ADVICE: &[Advice] = &[
    Advice {
        name: "cpu",
        what: (
            "Processor load and the busiest processes.",
            "Charge processeur et processus les plus actifs.",
        ),
        why: (
            "Sustained high CPU can slow everything down or point to a runaway process.",
            "Une charge CPU élevée ralentit tout ou signale un processus incontrôlé.",
        ),
        remedy: (
            "Stop the hungriest process with `kill <PID>` if it's not expected.",
            "Arrêtez le plus gourmand avec `kill <PID>`, sauf si c'est normal.",
        ),
    },
    Advice {
        name: "memory",
        what: (
            "RAM and swap usage, plus the hungriest processes.",
            "Utilisation RAM et swap, et processus les plus gourmands.",
        ),
        why: (
            "Low free memory triggers swapping and OOM kills — work slows or apps vanish.",
            "Peu de mémoire libre provoque du swap et des OOM — tout ralentit ou des apps disparaissent.",
        ),
        remedy: (
            "Close the heaviest apps first; add swap or RAM if it repeats.",
            "Fermez d'abord les apps les plus lourdes ; ajoutez swap ou RAM.",
        ),
    },
    Advice {
        name: "disk",
        what: (
            "Free space on each mounted partition.",
            "Espace libre sur chaque partition montée.",
        ),
        why: (
            "A full disk stops writes, breaks updates and can corrupt databases.",
            "Un disque plein bloque les écritures, casse les mises à jour et peut corrompre des bases.",
        ),
        remedy: (
            "`josephine clean` previews what can be freed; clear the biggest.",
            "`josephine clean` estime ce qui peut être libéré ; videz le plus gros.",
        ),
    },
    Advice {
        name: "temperature",
        what: (
            "CPU and NVMe sensor temperatures.",
            "Températures des capteurs CPU et NVMe.",
        ),
        why: (
            "Overheating throttles performance and shortens hardware life.",
            "La surchauffe limite les performances et réduit la durée de vie du matériel.",
        ),
        remedy: (
            "Check airflow and dust; `sensors` for detail. On a duvet? Move it.",
            "Vérifiez ventilation et poussière ; `sensors` pour le détail.",
        ),
    },
    Advice {
        name: "systemd",
        what: (
            "Failed units and services that restart too often.",
            "Unités en échec et services qui redémarrent trop souvent.",
        ),
        why: (
            "A failed service means something you rely on may be down; crash loops hide root causes.",
            "Un service en échec signifie qu'un composant est peut-être arrêté ; les boucles de redémarrage masquent la cause.",
        ),
        remedy: (
            "Restart it: `sudo systemctl restart <unit>`, then `systemctl status`.",
            "Relancez-la : `sudo systemctl restart <unité>`, puis `systemctl status`.",
        ),
    },
    Advice {
        name: "updates",
        what: (
            "Pending package updates (apt, dnf or pacman).",
            "Mises à jour de paquets en attente (apt, dnf ou pacman).",
        ),
        why: (
            "Unpatched packages leave known vulnerabilities and bugs on the system.",
            "Des paquets non mis à jour laissent des failles et bugs connus sur le système.",
        ),
        remedy: (
            "Apply them when convenient: `sudo apt upgrade`, `sudo dnf upgrade`.",
            "Appliquez-les quand vous pouvez : `sudo apt upgrade`, `sudo dnf upgrade`.",
        ),
    },
    Advice {
        name: "network",
        what: (
            "Round-trip latency to the default gateway.",
            "Latence aller-retour vers la passerelle par défaut.",
        ),
        why: (
            "High latency or packet loss means local network trouble before the wider internet.",
            "Une latence élevée ou des pertes signalent un souci réseau local avant Internet.",
        ),
        remedy: (
            "Check Wi-Fi signal, cables and router; `ping` the gateway to confirm.",
            "Vérifiez signal Wi-Fi, câbles et routeur ; `ping` la passerelle.",
        ),
    },
    Advice {
        name: "battery",
        what: (
            "Charge level and depletion rate on battery power.",
            "Niveau de charge et vitesse de décharge sur batterie.",
        ),
        why: (
            "A battery draining fast or stuck low means you may lose work mid-session.",
            "Une batterie qui se vide vite ou reste basse peut couper votre travail en cours.",
        ),
        remedy: (
            "Plug in if you can; check power settings and what keeps the GPU awake.",
            "Branchez si possible ; vérifiez l'alimentation et le GPU qui veille.",
        ),
    },
    Advice {
        name: "inode",
        what: (
            "Inode usage on writable filesystems.",
            "Utilisation des inodes sur les systèmes de fichiers accessibles en écriture.",
        ),
        why: (
            "A disk can be \"full\" on inodes while still showing free space — many tiny files.",
            "Un disque peut être « plein » en inodes tout en affichant de l'espace libre — beaucoup de petits fichiers.",
        ),
        remedy: (
            "Prune caches and temp trees; `du --inodes -x -d1 /` finds the nests.",
            "Purgez caches et fichiers temporaires ; `du --inodes -x -d1 /` aide.",
        ),
    },
    Advice {
        name: "smart",
        what: (
            "SMART health status of disks (opt-in, often needs root).",
            "État de santé SMART des disques (opt-in, souvent root requis).",
        ),
        why: (
            "SMART warnings often precede hard drive failure by days or weeks.",
            "Les alertes SMART précèdent souvent une panne disque de quelques jours ou semaines.",
        ),
        remedy: (
            "Back up immediately; enable the check in config if `smartctl` allows.",
            "Sauvegardez immédiatement ; activez le check si `smartctl` le permet.",
        ),
    },
    Advice {
        name: "kernel",
        what: (
            "Kernel incidents in the last hour (OOM kills, oops, panics).",
            "Incidents noyau sur la dernière heure (OOM, oops, panics).",
        ),
        why: (
            "Kernel faults destabilise the whole machine — not just one app.",
            "Les fautes noyau déstabilisent toute la machine — pas seulement une app.",
        ),
        remedy: (
            "Read it: `journalctl -k -p err --since -1h`; back up if it repeats.",
            "Lisez : `journalctl -k -p err --since -1h` ; sauvegardez si ça répète.",
        ),
    },
    Advice {
        name: "filesystem",
        what: (
            "Writable filesystems unexpectedly mounted read-only.",
            "Systèmes de fichiers habituellement accessibles en écriture montés en lecture seule.",
        ),
        why: (
            "A silent read-only remount often means disk errors or corruption — data loss risk.",
            "Un remontage silencieux en lecture seule signale souvent erreurs disque ou corruption — risque de perte.",
        ),
        remedy: (
            "Back up what matters now, then `dmesg -T | grep -i read-only`.",
            "Sauvegardez ce qui compte, puis `dmesg -T | grep -i read-only`.",
        ),
    },
    Advice {
        name: "timesync",
        what: (
            "Whether the system clock is synchronised via NTP.",
            "Si l'horloge système est synchronisée via NTP.",
        ),
        why: (
            "Clock drift breaks log ordering, TLS validation and scheduled jobs.",
            "Une horloge qui dérive casse l'ordre des journaux, la validation TLS et les tâches planifiées.",
        ),
        remedy: (
            "`timedatectl set-ntp true` usually fixes it; check `timedatectl`.",
            "`timedatectl set-ntp true` règle souvent ça ; voyez `timedatectl`.",
        ),
    },
    Advice {
        name: "security",
        what: (
            "Failed login and authentication attempts in the last hour.",
            "Tentatives de connexion et d'authentification échouées sur la dernière heure.",
        ),
        why: (
            "Bursts of failed logins may mean someone is probing your machine.",
            "Des rafales de connexions échouées peuvent signifier qu'on sonde votre machine.",
        ),
        remedy: (
            "Review `journalctl -u sshd`; keys-only SSH and fail2ban if not you.",
            "Voyez `journalctl -u sshd` ; SSH par clés et fail2ban si pas vous.",
        ),
    },
    Advice {
        name: "reboot",
        what: (
            "Whether a restart is needed to finish applying updates.",
            "Si un redémarrage est nécessaire pour finir d'appliquer des mises à jour.",
        ),
        why: (
            "A new kernel or libraries only take effect after a reboot — security fixes included.",
            "Un nouveau noyau ou de nouvelles bibliothèques ne prennent effet qu'après un redémarrage.",
        ),
        remedy: (
            "Save your work, then reboot when convenient: `systemctl reboot`.",
            "Enregistrez, puis redémarrez quand vous voulez : `systemctl reboot`.",
        ),
    },
    Advice {
        name: "pressure",
        what: (
            "Time tasks spend stalled waiting for memory, CPU or IO (Linux PSI).",
            "Le temps passé par les tâches à attendre mémoire, CPU ou E/S (PSI).",
        ),
        why: (
            "Rising pressure means the machine is thrashing — it slows well before it fails.",
            "Une pression qui monte signale du thrashing — ça ralentit bien avant de lâcher.",
        ),
        remedy: (
            "Close the biggest memory user; add swap or RAM if it keeps stalling.",
            "Fermez le plus gros consommateur mémoire ; ajoutez swap ou RAM.",
        ),
    },
];

/// The advice for one check, by its internal name (`cpu`, `disk`, …).
pub fn advice(check: &str) -> Option<&'static Advice> {
    ADVICE.iter().find(|a| a.name == check)
}

/// Every check's advice, in listing order.
pub fn all() -> &'static [Advice] {
    ADVICE
}

/// At most this many entries per check in the "what's left to do" section;
/// beyond it, a counted overflow line. Nothing is dropped silently.
pub const MAX_ENTRIES_PER_CHECK: usize = 5;

/// One actionable step, ready to render.
pub struct Remedy {
    /// The action — usually a command to run.
    pub action: String,
    /// Optional second line: how to understand the problem before acting.
    pub hint: Option<String>,
}

impl Remedy {
    fn action(action: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            hint: None,
        }
    }

    fn with_hint(action: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            hint: Some(hint.into()),
        }
    }
}

/// The actions for a check that came back Attention or Critique.
///
/// Contextual where the collected state allows it (`systemd`, `cpu`,
/// `memory`), static [`Advice`] copy otherwise. A healthy check yields
/// nothing — Joséphine does not manufacture work.
pub fn for_result(result: &CheckResult) -> Vec<Remedy> {
    if result.worst_severity() == Severity::Info {
        return Vec::new();
    }
    let Some(advice) = advice(&result.check_name) else {
        return Vec::new();
    };

    match result.check_name.as_str() {
        "systemd" if !result.top_processes.is_empty() => failed_units(&result.top_processes),
        "cpu" | "memory" => match result.top_processes.first() {
            Some(top) => vec![Remedy::with_hint(
                match i18n::lang() {
                    Lang::En => format!("Biggest consumer right now: {top}"),
                    Lang::Fr => format!("Le plus gourmand en ce moment : {top}"),
                },
                i18n::t(advice.remedy.0, advice.remedy.1),
            )],
            None => vec![Remedy::action(i18n::t(advice.remedy.0, advice.remedy.1))],
        },
        _ => vec![Remedy::action(i18n::t(advice.remedy.0, advice.remedy.1))],
    }
}

/// One restart per failed unit, capped, with a counted overflow line.
fn failed_units(units: &[String]) -> Vec<Remedy> {
    let mut remedies: Vec<Remedy> = units
        .iter()
        .take(MAX_ENTRIES_PER_CHECK)
        .map(|unit| {
            Remedy::with_hint(
                format!("sudo systemctl restart {unit}"),
                match i18n::lang() {
                    Lang::En => format!("(find out why: systemctl status {unit})"),
                    Lang::Fr => format!("(comprendre pourquoi : systemctl status {unit})"),
                },
            )
        })
        .collect();

    let hidden = units.len().saturating_sub(MAX_ENTRIES_PER_CHECK);
    if hidden > 0 {
        remedies.push(Remedy::action(match i18n::lang() {
            Lang::En => format!("+ {hidden} more failed units — `systemctl --failed` lists them"),
            Lang::Fr => {
                format!("+ {hidden} autres unités en échec — `systemctl --failed` les liste")
            }
        }));
    }
    remedies
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{self, Lang};

    /// The fallback tests read the EN string, so pin the language. Holds
    /// `i18n::lock_for_test()` for the duration: `set_lang`/`lang()` are a
    /// bare process-wide atomic, and this crate's tests run concurrently, so
    /// without the guard another test pinning French could interleave
    /// between our `set_lang(En)` and the read inside `for_result`.
    fn with_english<T>(f: impl FnOnce() -> T) -> T {
        let _guard = i18n::lock_for_test();
        let prev = i18n::lang();
        i18n::set_lang(Lang::En);
        let out = f();
        i18n::set_lang(prev);
        out
    }

    #[test]
    fn all_checks_have_advice() {
        // `smart` is opt-in (off by default) — flip it on so every check gets
        // built, otherwise this would exercise one fewer than the real count.
        let mut config = crate::config::ChecksConfig::default();
        config.smart.enabled = true;
        let checks = crate::checks::build_checks(&config);

        assert_eq!(checks.len(), 16, "expected sixteen checks to be built");
        for check in &checks {
            assert!(
                advice(check.name()).is_some(),
                "check `{}` has no advice entry in remedy.rs — it would be \
                 diagnosed by `doctor` but silently un-actionable",
                check.name()
            );
        }
    }

    #[test]
    fn advice_is_bilingual_and_non_empty() {
        for entry in ADVICE {
            for (en, fr) in [entry.what, entry.why, entry.remedy] {
                assert!(
                    !en.trim().is_empty(),
                    "{} has an empty EN string",
                    entry.name
                );
                assert!(
                    !fr.trim().is_empty(),
                    "{} has an empty FR string",
                    entry.name
                );
                assert_ne!(en, fr, "{} looks untranslated", entry.name);
            }
        }
    }

    #[test]
    fn no_remedy_is_circular() {
        for entry in ADVICE {
            for text in [entry.remedy.0, entry.remedy.1] {
                assert!(
                    !text.contains("josephine doctor"),
                    "{} sends the reader back to doctor: {text}",
                    entry.name
                );
                assert!(
                    !text.contains("josephine fix"),
                    "{} references the removed fix command: {text}",
                    entry.name
                );
            }
        }
    }

    #[test]
    fn remedies_fit_a_terminal_line() {
        // 80 columns minus the section's "    1. " prefix.
        for entry in ADVICE {
            for text in [entry.remedy.0, entry.remedy.1] {
                let width = text.chars().count();
                assert!(
                    width <= 73,
                    "{} remedy is {width} chars, over the 73 budget: {text}",
                    entry.name
                );
            }
        }
    }

    #[test]
    fn advice_is_found_by_name() {
        assert_eq!(advice("cpu").unwrap().name, "cpu");
        assert!(advice("nope").is_none());
    }

    use crate::check::{CheckResult, Metric, Severity};

    /// A `CheckResult` whose single metric lands on the given severity.
    fn result(name: &str, severity: Severity, top: &[&str]) -> CheckResult {
        let value = match severity {
            Severity::Info => 0.0,
            Severity::Attention => 80.0,
            Severity::Critique => 95.0,
        };
        CheckResult {
            check_name: name.into(),
            metrics: vec![Metric {
                name: "usage_percent".into(),
                value,
                unit: "%".into(),
                threshold_warning: Some(75.0),
                threshold_critical: Some(90.0),
            }],
            details: Vec::new(),
            top_processes: top.iter().map(|s| s.to_string()).collect(),
            status_value: None,
        }
    }

    #[test]
    fn healthy_check_yields_no_remedy() {
        assert!(for_result(&result("disk", Severity::Info, &[])).is_empty());
    }

    #[test]
    fn systemd_yields_one_entry_per_failed_unit() {
        let r = result(
            "systemd",
            Severity::Critique,
            &["nginx.service", "backup.timer"],
        );
        let remedies = for_result(&r);
        assert_eq!(remedies.len(), 2);
        assert_eq!(remedies[0].action, "sudo systemctl restart nginx.service");
        assert_eq!(remedies[1].action, "sudo systemctl restart backup.timer");
        assert!(
            remedies[0]
                .hint
                .as_ref()
                .unwrap()
                .contains("systemctl status nginx.service")
        );
    }

    #[test]
    fn systemd_caps_the_list_and_counts_the_overflow() {
        let units = [
            "a.service",
            "b.service",
            "c.service",
            "d.service",
            "e.service",
            "f.service",
            "g.service",
        ];
        let remedies = for_result(&result("systemd", Severity::Critique, &units));
        assert_eq!(remedies.len(), MAX_ENTRIES_PER_CHECK + 1);
        let overflow = &remedies[MAX_ENTRIES_PER_CHECK].action;
        assert!(
            overflow.contains('2'),
            "overflow must count the rest: {overflow}"
        );
        assert!(remedies[MAX_ENTRIES_PER_CHECK].hint.is_none());
    }

    #[test]
    fn systemd_at_the_cap_has_no_overflow_line() {
        let units = [
            "a.service",
            "b.service",
            "c.service",
            "d.service",
            "e.service",
        ];
        let remedies = for_result(&result("systemd", Severity::Critique, &units));
        assert_eq!(remedies.len(), MAX_ENTRIES_PER_CHECK);
        assert!(remedies.iter().all(|r| r.hint.is_some()));
    }

    #[test]
    fn systemd_one_past_the_cap_overflows_by_one() {
        let units = [
            "a.service",
            "b.service",
            "c.service",
            "d.service",
            "e.service",
            "f.service",
        ];
        let remedies = for_result(&result("systemd", Severity::Critique, &units));
        assert_eq!(remedies.len(), MAX_ENTRIES_PER_CHECK + 1);
        let overflow = &remedies[MAX_ENTRIES_PER_CHECK].action;
        assert!(
            overflow.contains('1'),
            "overflow must count the rest: {overflow}"
        );
        assert!(remedies[MAX_ENTRIES_PER_CHECK].hint.is_none());
    }

    #[test]
    fn systemd_without_failed_units_falls_back_to_static_advice() {
        with_english(|| {
            let remedies = for_result(&result("systemd", Severity::Critique, &[]));
            assert_eq!(remedies.len(), 1);
            assert_eq!(remedies[0].action, advice("systemd").unwrap().remedy.0);
        });
    }

    #[test]
    fn cpu_quotes_the_top_process_verbatim() {
        let r = result("cpu", Severity::Attention, &["firefox (PID 1234) — 87.3 %"]);
        let remedies = for_result(&r);
        assert_eq!(remedies.len(), 1);
        assert!(
            remedies[0].action.contains("firefox (PID 1234) — 87.3 %"),
            "got: {}",
            remedies[0].action
        );
        assert!(remedies[0].hint.is_some());
    }

    #[test]
    fn cpu_without_process_list_falls_back_to_static_advice() {
        with_english(|| {
            let remedies = for_result(&result("cpu", Severity::Attention, &[]));
            assert_eq!(remedies.len(), 1);
            assert_eq!(remedies[0].action, advice("cpu").unwrap().remedy.0);
        });
    }

    #[test]
    fn memory_without_process_list_falls_back_to_static_advice() {
        with_english(|| {
            let remedies = for_result(&result("memory", Severity::Attention, &[]));
            assert_eq!(remedies.len(), 1);
            assert_eq!(remedies[0].action, advice("memory").unwrap().remedy.0);
        });
    }

    #[test]
    fn check_without_contextual_rule_falls_back_to_static_advice() {
        with_english(|| {
            let remedies = for_result(&result("timesync", Severity::Attention, &[]));
            assert_eq!(remedies.len(), 1);
            assert_eq!(remedies[0].action, advice("timesync").unwrap().remedy.0);
            assert!(remedies[0].hint.is_none());
        });
    }

    #[test]
    fn unknown_check_yields_no_remedy() {
        assert!(for_result(&result("nope", Severity::Critique, &[])).is_empty());
    }
}
