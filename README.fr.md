# Joséphine

> **L'esprit gardien de votre ordinateur.**

Joséphine observe votre machine en silence et n'intervient que lorsqu'une intervention est utile. Elle surveille, détecte, prévient et accompagne — sans jamais devenir intrusive.

Tout reste **100 % local** : aucune donnée envoyée sur Internet, aucun compte, aucun cloud.

---

## Philosophie

| Principe | Description |
|----------|-------------|
| **Invisible** | Pas de fenêtre, pas d'interface graphique. Elle n'apparaît que quand c'est utile. |
| **Bienveillante** | Jamais `ERROR`, `FATAL`, `PANIC`. Toujours un ton chaleureux et protecteur, jamais alarmiste. |
| **Locale** | Exécution entièrement sur votre machine. |

---

## Fonctionnalités

- **Surveillance** — CPU, mémoire, disque, température, services systemd
- **Démon** — veille en arrière-plan avec intervalles configurables
- **Règles anti-spam** — une notification par changement d'état (NORMAL → WARNING → CRITICAL → RECOVERED)
- **Notifications desktop** — via libnotify, messages avec humour bienveillant
- **Historique SQLite** — métriques et événements sur 90 jours (configurable)
- **CLI soigné** — tableaux, barres de progression, barres d'indicateurs colorées

---

## Installation

**Prérequis :** Rust 1.75+, Linux (Debian 13+ recommandé).

```bash
git clone <repo>
cd josephine
cargo install --path crates/josephine
```

**Notifications desktop :**

```bash
sudo apt install libnotify-bin
```

---

## Démarrage rapide

```bash
josephine status          # résumé instantané
josephine doctor          # diagnostic détaillé
josephine daemon start    # lancer la surveillance
josephine history         # synthèse des dernières 24 h
```

---

## Commandes

| Commande | Description |
|----------|-------------|
| `josephine` | Alias de `status` |
| `josephine status` | Résumé CPU, RAM, disque, température, systemd |
| `josephine doctor` | Diagnostic complet avec détails par check |
| `josephine history` | Historique 24 h (max métriques + événements) |
| `josephine daemon start` | Démarre le démon de surveillance |
| `josephine daemon stop` | Arrête le démon |
| `josephine daemon status` | État du démon (PID, uptime) |
| `josephine daemon logs` | Dernières lignes du journal |
| `josephine daemon restart` | Redémarre le démon |
| `josephine config show` | Affiche la configuration YAML |
| `josephine config validate` | Valide la configuration |

**Hors périmètre :** Docker, TUI/`watch` (voir roadmap).

---

## Notifications

Les notifications sont envoyées par le **démon** lorsqu'un seuil est franchi ou qu'une situation revient à la normale.

Exemple :

```
✨ Joséphine

Votre disque est à 92 % — il tousse un peu.
Même au paradis, on n'a pas de stockage illimité.

Je peux vous aider à voir ce qui encombre : josephine doctor.
```

Activer / désactiver dans `~/.config/josephine/config.yaml` :

```yaml
notifications:
  desktop: true
  terminal: false   # non implémenté
```

---

## Configuration

Fichier : `~/.config/josephine/config.yaml` (créé automatiquement au premier lancement).

```yaml
checks:
  cpu:
    enabled: true
    interval_secs: 30
    warning: 85
    critical: 95
  memory:
    enabled: true
    interval_secs: 60
    warning: 85
    critical: 95
  disk:
    enabled: true
    interval_secs: 120
    warning: 85
    critical: 95
  temperature:
    enabled: true
    interval_secs: 60
    warning: 75
    critical: 90
  systemd:
    enabled: true
    interval_secs: 120
    failed_warning: 1
    failed_critical: 3
    restarts_warning: 5
    restarts_critical: 10

notifications:
  desktop: true
  terminal: false

history:
  enabled: true
  retention_days: 90
```

**Chemins :**

| Fichier | Emplacement |
|---------|-------------|
| Config | `~/.config/josephine/config.yaml` |
| Base SQLite | `~/.local/share/josephine/josephine.db` |
| Logs démon | `~/.local/share/josephine/daemon.log` |
| PID démon | `~/.local/share/josephine/daemon.pid` |

---

## Architecture

```
josephine  →  josephine-core
                    ├── checks/      cpu, memory, disk, temperature, systemd
                    ├── rules/       moteur d'états anti-spam
                    ├── scheduler/   boucle tokio (démon)
                    ├── storage/     SQLite
                    ├── messages/    voix des notifications
                    └── notify/      libnotify
```

**Stack :** Rust, clap, tokio, sysinfo, rusqlite, notify-rust, indicatif, comfy-table.

---

## Développement

```bash
cargo build
cargo test
cargo run -p josephine -- status
cargo run -p josephine -- doctor
```

**Documentation développeur :** [docs/README.md](docs/README.md) · [docs/CURRENT_STATE.md](docs/CURRENT_STATE.md) · [AGENTS.md](AGENTS.md)

---

## Roadmap

Voir [docs/ROADMAP.md](docs/ROADMAP.md). Prochaine cible : **v0.2** (`report`, `daemon install`, packaging).

---

## Licence

MIT
