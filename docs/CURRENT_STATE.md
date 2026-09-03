# Joséphine — État actuel du code

**Version :** 0.13.1  
**Dernière mise à jour :** 2026-09-03  
**Langage :** Rust (workspace Cargo)  
**Cible :** Linux (Debian 13+ recommandé)

Ce document est la **source de vérité** pour l'état du dépôt. En cas de divergence avec la spec v0.1 initiale, **ce fichier prévaut**.

---

## Livré

### Checks (16)

| Check | Métriques principales | Source |
|-------|----------------------|--------|
| `cpu` | `usage_percent`, load avg, top 3 processus | `sysinfo` |
| `memory` | `usage_percent`, `swap_percent`, top 3 processus | `sysinfo` |
| `disk` | partitions montées, `usage_percent_worst` | `sysinfo` |
| `temperature` | `temp_max_celsius` | `/sys/class/thermal`, NVMe hwmon |
| `systemd` | `failed_units`, `max_restarts` | `systemctl` |
| `updates` | `updates_available` | `apt` / `dnf` / `pacman` |
| `network` | `gateway_latency_ms` (LAN, 100 % local) | `/proc/net/route`, `ping` |
| `battery` | `charge_percent`, `battery_depletion_percent` | `/sys/class/power_supply` |
| `inode` | `inode_usage_percent_worst` | `df -iPT`* |
| `smart` | `smart_failing`, `smart_wear_percent` (opt-in, root requis) | `smartctl -H`, `smartctl -j -a` |
| `kernel` | `kernel_incidents` (OOM, oops…) | `journalctl -k` |
| `filesystem` | `readonly_mounts` | `/proc/mounts` |
| `timesync` | `clock_unsynced` | `timedatectl` |
| `security` | `failed_auths` | `journalctl` |
| `reboot` | `reboot_required` | `/run/reboot-required`, `needs-restarting`, `/run/{booted,current}-system`, `/lib/modules` |
| `pressure` | `memory_pressure_full_avg60` (seuil) + `some` cpu/io (info) | `/proc/pressure/{memory,cpu,io}` |

\* Le `T` ajoute le type de filesystem à la sortie de `df`, pour ignorer les
montages en lecture seule de type image (`squashfs`, `iso9660`, `erofs` — ex.
snaps), toujours à 100 % d'inodes par construction et jamais actionnables.

Chaque check implémente le trait `Check` (`josephine-core/src/check.rs`), est indépendant, configurable via YAML.

### CLI

| Commande | Statut |
|----------|--------|
| `status` (défaut, `--json`, `--oneline`, code de sortie 0/1/2 ; 64/70 en cas d’échec) | ✅ |
| `doctor` (`--verbose`, `--json`) | ✅ |
| `history` | ✅ |
| `daemon start/stop/restart/status/logs/run` | ✅ |
| `config show/validate/edit` | ✅ |
| `clean` (`--apply`), `report` (`-o`, `--json`, `--since`) | ✅ |
| `notify test` | ✅ |
| `update` (`--check`, `--yes`) | ✅ |
| `completions <bash\|zsh\|fish…>` | ✅ |
| `explain` (`<check>` optionnel) | ✅ |

**Supprimé du scope :** `watch` (TUI), check Docker.

**Capacités transverses (increment B, depuis 0.8.0) :** `--json` (sortie
machine-readable pour `status`/`doctor`/`report`, sévérité `ok`/`warning`/
`critical`) ; complétions shell (`completions <shell>`, via `clap_complete`) ;
notifications **terminal** — le démon honore `notifications.terminal` et émet
les alertes dans son journal en parallèle du canal desktop ; `--help`
**localisé** — l'aide suit `language` (anglais par défaut, français avec
`language: fr`).

### Rendu CLI — « Constellation sobre » (depuis 0.7.0)

`status`, `doctor` et `history` sont rendus dans un langage visuel sobre : un
en-tête discret `✦ Joséphine`, le statut porté par des glyphes de forme *et*
de couleur (`●` ok, `▲` attention, `✕` critique — dégradés en
`[ok]/[!]/[x]` hors TTY), des colonnes alignées, et un texte détoné
(bilingue, ton « chaleur sobre » : direct et rassurant, sans mascotte ni
emoji). Un `banner.txt` personnalisé (`~/.config/josephine/banner.txt`) reste
honoré au-dessus de l'en-tête. Les notifications desktop
(`messages.rs`) suivent le même détonage. Toutes les commandes sont
désormais passées à la voix « chaleur sobre » : `clean` et `update`
s'ouvrent elles aussi avec l'en-tête sobre `✦` ; `report` garde son propre
titre de document (« Joséphine — rapport système ») sans en-tête `✦`, pour
éviter un « Joséphine » en double ; `daemon`, `config` et `notify` n'affichent
aucun en-tête, seulement des lignes de résultat détonées.

### Voix & variété — l'ange gardien (depuis 0.10.0)

Joséphine a désormais un peu de **caractère**, et surtout de la **variété** :
le module `josephine-core/src/voice.rs` fournit un moteur qui tire au sort une
formulation parmi plusieurs (toujours **EN + FR**), pour qu'elle ne dise pas la
même phrase à chaque fois. Sont variés : l'accroche de `status`, la ligne
« tout va bien », la clôture des remèdes de `doctor`, les lignes de `daemon`, le message
de `notify test`, et les notifications de **rétablissement**. Règle d'or : la
variété ne touche que les **lignes de caractère** — les *faits* d'une alerte
(le nombre, la commande à lancer) restent stables et précis. Un
`✧` discret ferme la section « ce qu'il reste à faire » de `josephine
doctor`.

`doctor` **diagnostique** maintenant vraiment : il ouvre sur un **verdict**
(rien à signaler / une note ou deux / quelque chose réclame votre attention)
avant l'examen check par check. `history` s'est réchauffé : accroche 24 h,
transitions d'état adoucies façon garde (`▲ attention → ● résolu` au lieu de
`WARNING → RECOVERED`), et une ligne de clôture — le tout bilingue.

Depuis 0.11.0, il ferme aussi sur **ce qu'il reste à faire** : la liste des
actions à mener, la plus grave d'abord, et seulement quand il y en a.

### Démon

- Binaire unique : `josephine --__daemon__` (flag interne)
- PID : `~/.local/share/josephine/daemon.pid`
- Logs : `~/.local/share/josephine/daemon.log`
- Scheduler tokio : une tâche async par check activé

### Règles & notifications

- États : NORMAL → WARNING → CRITICAL → RECOVERED
- Anti-spam : pas de notification si l'état ne change pas
- Messages : module `messages.rs`, ton « chaleur sobre » depuis 0.7.0 —
  direct, calme, rassurant, jamais alarmiste (identité ange gardien,
  sans mascotte ni emoji). Depuis 0.10.0, les messages de
  **rétablissement** sont variés via `voice.rs` (plusieurs formulations
  EN + FR) ; le **corps des alertes reste stable** (fait + commande)
- Canal : desktop via `notify-rust` / libnotify
- `notifications.terminal` : présent en config, **non implémenté**

### Stockage

- SQLite : `~/.local/share/josephine/josephine.db`
- Tables : `metrics`, `events`, `notifications`, `checks_log`
- Purge horaire selon `history.retention_days`

### Affichage CLI

- Module `josephine/src/output/` : `bars`, `status`, `doctor`, `runner`, `style`
- `indicatif` : progression pendant les checks (`status`, `doctor`)
- `style.rs` : primitives partagées « Constellation sobre » (glyphes de
  statut, en-tête, accent) ; `bars` : mini-barres, utilisées par `doctor`
  uniquement
- `comfy-table` : encore utilisé pour les tableaux de `history` (tendance,
  événements) ; `status`/`doctor` sont passés aux colonnes alignées sobres
  (plus de tableau encadré) — couleurs toujours via l'API table, jamais
  d'ANSI brut dans les cellules
- Mode plain si sortie non-TTY (glyphes `[ok]/[!]/[x]`, pas de couleur)
- **Pas de logo ASCII par défaut** — en-tête sobre `✦ Joséphine` ;
  `banner.txt` personnalisé toujours possible

---

## Configuration

Fichier : `~/.config/josephine/config.yaml`

**Langue :** `language: en` (défaut) ou `fr` — tout le texte user-facing est
bilingue (voir `i18n.rs`). L'anglais est la langue par défaut depuis la v0.5.0.

Structures notables :

- `CheckThresholds` — cpu, memory, disk (%, intervalles)
- `FilesystemCheckConfig`, `TimesyncCheckConfig`, `SecurityCheckConfig` — seuils dédiés (pas les défauts 85/95)
- `TemperatureThresholds` — seuils en °C (20–150)
- `SystemdCheckConfig` — seuils `failed_*` et `restarts_*` (comptes entiers ≥ 1)
- `ForecastConfig` — prévoyance (régression déterministe sur l'historique) :
  `horizon_days`, `min_samples`, `min_fit` (défauts 30 / 12 / 0.5). Le module
  `forecast.rs` projette disque/inodes/usure SSD — pas la mémoire, qui est
  libérée et non accumulée ; rendu **doctor-only**
  (section « Prévoyance »), sans notification.

Validation dans `config.rs::validate()`.

---

## Tests

Tests unitaires (`josephine-core`) + intégration CLI (`assert_cmd`) couvrant
config, règles, messages, self-update, réseau, batterie et le parsing des
commandes (`clean`).

Les checks reposant sur `/proc` / `systemctl` / `ping` ne sont pas exécutés
en CI ; leur logique pure est testée via des helpers dédiés.

Commande : `cargo test --workspace`

---

## Stack (workspace)

| Crate | Usage |
|-------|--------|
| clap 4 | CLI |
| tokio | Démon / scheduler |
| sysinfo | CPU, RAM, disque |
| rusqlite | Historique |
| notify-rust | Notifications |
| serde_yaml | Config |
| indicatif | Barre de progression CLI |
| comfy-table | Tableaux |
| colored | Bannières, résumés |
| tracing | Logs démon |

---

## Décisions produit actées

| Décision | Détail |
|----------|--------|
| Langage | Rust (pas Python du PVD original) |
| Docker | Hors périmètre |
| TUI / `watch` | Hors périmètre |
| Logo ASCII | Retiré au profit d'un en-tête sobre `✦` (0.7.0) ; `banner.txt` personnalisé toujours possible |
| Notifications | Démon uniquement, pas le CLI interactif |
| Ton | Bilingue (anglais par défaut, français en option) ; identité ange gardien, sucre visuel retiré — « chaleur sobre » depuis 0.7.0 ; **variété + caractère joueur** via `voice.rs` depuis 0.10.0 (lignes de caractère uniquement, faits d'alerte stables) |

---

## Fichiers clés

```
crates/josephine-core/src/
  check.rs           trait Check
  checks/            implémentations
  config.rs          YAML + validation
  rules.rs           moteur d'états
  messages.rs        textes notifications (faits d'alerte, stables)
  remedy.rs          conseils par check : quoi, pourquoi, remède (EN + FR)
  voice.rs           variété des lignes de caractère (EN + FR)
  scheduler.rs       boucle démon + run_all_checks
  storage.rs         SQLite
  daemon.rs          start/stop/status
  notify.rs          libnotify

crates/josephine/src/
  main.rs            clap
  output/            rendu terminal
  commands/          sous-commandes
```

---

## Prochaine étape documentée

Voir [ROADMAP.md](ROADMAP.md) et le programme d'increments CLI en cours :
[superpowers/specs/2026-07-08-josephine-cli-render-tone-design.md](superpowers/specs/2026-07-08-josephine-cli-render-tone-design.md)
(increment A, livré) et
[superpowers/plans/2026-07-09-josephine-cli-increment-c.md](superpowers/plans/2026-07-09-josephine-cli-increment-c.md)
(increment C, en cours).
