//! Joséphine's voice — a little variety so she doesn't say things the exact
//! same way every time.
//!
//! A guardian spirit with some character: warm, quietly playful, and never the
//! same line twice in a row if we can help it. Two rules hold everywhere:
//!
//! * **Flavour only.** Variety lives on personality lines (greetings, "all
//!   clear", sign-offs, recoveries). The *facts* of an alert — the number, the
//!   command to run — stay stable and precise. An SRE should never see the
//!   shape of a disk alert change from one run to the next.
//! * **English and French, always**, and never `ERROR` / `FATAL` / `PANIC`.
//!
//! Each pool is a slice of `(en, fr)` pairs; [`pick`] returns one at random in
//! the active language. [`index`] gives a random slot for callers that build
//! interpolated strings themselves.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::check::Severity;
use crate::i18n::{self, Lang};

/// Monotonic nudge so two picks in the same microsecond still differ.
static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A random slot in `0..len` (always `0` for a single-element pool). Mixes the
/// wall clock, the process id and a per-call counter — enough entropy for
/// picking a phrasing, with no external dependency.
pub fn index(len: usize) -> usize {
    if len <= 1 {
        return 0;
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut x =
        nanos ^ seq.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ ((std::process::id() as u64) << 17);
    // A round of splitmix64 finishing to spread low bits.
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x % len as u64) as usize
}

/// Pick one `(en, fr)` phrasing from `pool`, in the active language.
pub fn pick(pool: &[(&'static str, &'static str)]) -> &'static str {
    let (en, fr) = pool[index(pool.len())];
    match i18n::lang() {
        Lang::En => en,
        Lang::Fr => fr,
    }
}

// --- status / default -------------------------------------------------------

/// The dimmed line under the title on `josephine` / `josephine status`.
pub fn status_tagline() -> &'static str {
    pick(&[
        (
            "Your machine, watched over.",
            "Votre machine, sous bonne garde.",
        ),
        ("On watch, as ever.", "De garde, comme toujours."),
        (
            "Keeping a quiet eye on things.",
            "Je garde un œil, discrètement.",
        ),
        (
            "Everything under a watchful eye.",
            "Tout est sous l'œil bienveillant.",
        ),
        (
            "Here, minding the machine.",
            "Là, à veiller sur la machine.",
        ),
    ])
}

/// The `status` footer when nothing needs attention.
pub fn all_clear() -> &'static str {
    pick(&[
        ("All clear.", "Tout est au vert."),
        ("Nothing needs you.", "Rien ne réclame votre attention."),
        ("All quiet. Carry on.", "Calme plat. Continuez."),
        ("Not a cloud in sight.", "Pas un nuage à l'horizon."),
        ("Everything's in its right place.", "Tout est à sa place."),
    ])
}

// --- doctor -----------------------------------------------------------------

/// The opening verdict for `josephine doctor`, by overall severity — the
/// diagnosis before the check-by-check exam.
pub fn doctor_verdict(worst: Severity) -> &'static str {
    match worst {
        Severity::Info => pick(&[
            (
                "Clean bill of health — nothing needs you.",
                "Rien à signaler — vous n'avez rien à faire.",
            ),
            (
                "Everything checks out. A model patient.",
                "Tout est bon. Un patient modèle.",
            ),
            ("Fit as a fiddle.", "En pleine forme."),
            (
                "A thorough look, and nothing to report.",
                "Examen complet, rien à signaler.",
            ),
        ]),
        Severity::Attention => pick(&[
            (
                "A couple of things worth a glance — nothing urgent.",
                "Deux ou trois choses à regarder — rien d'urgent.",
            ),
            (
                "Mostly well, with a note or two.",
                "Globalement en forme, à une note ou deux près.",
            ),
            (
                "Nothing serious, but let's keep an eye on it.",
                "Rien de grave, mais gardons un œil dessus.",
            ),
        ]),
        Severity::Critique => pick(&[
            (
                "Something wants your attention now. Details below.",
                "Quelque chose réclame votre attention, maintenant. Détails plus bas.",
            ),
            (
                "Let's not dawdle — see below.",
                "Ne traînons pas — voyez plus bas.",
            ),
            (
                "One thing can't wait. It's just below.",
                "Une chose ne peut pas attendre. C'est juste en dessous.",
            ),
        ]),
    }
}

// --- remedies (her finger-snap) ---------------------------------------------

/// Closing line for the "what's left to do" section — she guides, you act.
pub fn remedy_hands_off() -> &'static str {
    pick(&[
        (
            "✧ No magic wand here: I show the way, you keep the wheel.",
            "✧ Pas de baguette magique ici : je montre le chemin, vous gardez le volant.",
        ),
        (
            "✧ I point, you press. Nothing runs behind your back.",
            "✧ Je montre, vous appuyez. Rien ne s'exécute dans votre dos.",
        ),
        (
            "✧ The wheel stays yours — I just lean over your shoulder.",
            "✧ Le volant reste à vous — je me contente de regarder par-dessus votre épaule.",
        ),
    ])
}

// --- daemon (on watch) ------------------------------------------------------

pub fn daemon_started() -> &'static str {
    pick(&[
        (
            "Daemon started — on watch, eyes open. Go about your day.",
            "Démon démarré — de garde, l'œil ouvert. Vaquez tranquille.",
        ),
        (
            "On duty. Off you go — I've got this.",
            "En poste. Filez — je m'en occupe.",
        ),
        (
            "Settled in for the watch. Sleep easy.",
            "Installée pour la veille. Dormez tranquille.",
        ),
    ])
}

pub fn daemon_stopped() -> &'static str {
    pick(&[
        (
            "Daemon stopped. Call me at the slightest trouble.",
            "Démon arrêté. Appelez-moi au moindre souci.",
        ),
        (
            "Off duty. You know where to find me.",
            "Repos. Vous savez où me trouver.",
        ),
        (
            "Standing down. A word and I'm back.",
            "Je me retire. Un mot et je reviens.",
        ),
    ])
}

pub fn daemon_restarted() -> &'static str {
    pick(&[
        (
            "Daemon restarted — back on watch.",
            "Démon redémarré — de nouveau de garde.",
        ),
        (
            "Back on my feet, back on watch.",
            "De nouveau sur pied, de nouveau de garde.",
        ),
    ])
}

pub fn daemon_back_on_guard() -> &'static str {
    pick(&[
        (
            "A `josephine daemon start` and I'm back on guard.",
            "Un `josephine daemon start` et je reprends la garde.",
        ),
        (
            "Say `josephine daemon start` whenever you'd like me watching again.",
            "Dites `josephine daemon start` quand vous voudrez que je reprenne la veille.",
        ),
    ])
}

// --- notify test ------------------------------------------------------------

pub fn notify_test_body() -> &'static str {
    pick(&[
        (
            "Just looking in on you — if you can read this, desktop notifications are working.",
            "Je viens prendre de vos nouvelles — si vous lisez ceci, les notifications de bureau fonctionnent.",
        ),
        (
            "A little wave from Joséphine — if you can see this, notifications are set.",
            "Un petit coucou de Joséphine — si vous voyez ceci, les notifications sont en place.",
        ),
        (
            "Testing, testing — and if this reached you, all's wired up.",
            "Test, test — et si ceci vous parvient, tout est branché.",
        ),
    ])
}

// --- errors (warm, never flippant) ------------------------------------------

/// Lead-in for the top-level error banner; the underlying error is appended.
/// Kept gentle — a stumble, not a catastrophe.
pub fn error_lead() -> &'static str {
    pick(&[
        (
            "✦ Joséphine ran into a snag:",
            "✦ Joséphine a rencontré un souci :",
        ),
        (
            "✦ Joséphine tripped on something:",
            "✦ Joséphine a buté sur quelque chose :",
        ),
        (
            "✦ Something got in Joséphine's way:",
            "✦ Quelque chose a gêné Joséphine :",
        ),
    ])
}

// --- history ----------------------------------------------------------------

/// Opening line for `history` when the last 24 h were uneventful.
pub fn history_calm() -> &'static str {
    pick(&[
        (
            "A calm 24 hours. Nothing worth losing sleep over.",
            "24 h tranquilles. Rien qui vaille de perdre le sommeil.",
        ),
        (
            "Twenty-four quiet hours on watch.",
            "Vingt-quatre heures de veille sans histoire.",
        ),
        (
            "Nothing dramatic overnight — just the way we like it.",
            "Rien de spectaculaire cette nuit — comme on aime.",
        ),
    ])
}

/// Closing line for `history` after showing the events.
pub fn history_closing() -> &'static str {
    pick(&[
        (
            "All of it noted, so you didn't have to.",
            "Tout est noté, pour que vous n'ayez pas à le faire.",
        ),
        (
            "Kept the log so you could sleep on it.",
            "J'ai tenu le journal pour que vous puissiez dormir dessus.",
        ),
        (
            "That's the last day, from where I sit.",
            "Voilà la dernière journée, vue d'ici.",
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORBIDDEN: [&str; 5] = ["ERROR", "FATAL", "PANIC", "CRASH", "ÉCHEC"];

    /// Every phrasing in every pool ships EN + FR, is non-empty, and stays
    /// clear of alarmist vocabulary — in either language.
    ///
    /// Holds `i18n::lock_for_test()` across the whole two-language loop:
    /// this test pins the global language to French for a stretch, and
    /// without the guard another test asserting English text could observe
    /// that mid-run.
    #[test]
    fn every_pool_is_bilingual_and_calm() {
        let _guard = i18n::lock_for_test();
        let prev = i18n::lang();
        let mut all: Vec<&'static str> = Vec::new();
        for lang in [Lang::En, Lang::Fr] {
            i18n::set_lang(lang);
            for _ in 0..80 {
                all.push(status_tagline());
                all.push(all_clear());
                all.push(remedy_hands_off());
                all.push(daemon_started());
                all.push(daemon_stopped());
                all.push(daemon_restarted());
                all.push(daemon_back_on_guard());
                all.push(notify_test_body());
                all.push(error_lead());
                all.push(history_calm());
                all.push(history_closing());
                for sev in [Severity::Info, Severity::Attention, Severity::Critique] {
                    all.push(doctor_verdict(sev));
                }
            }
        }
        i18n::set_lang(prev);
        for s in all {
            assert!(!s.trim().is_empty(), "empty phrasing");
            let upper = s.to_uppercase();
            for word in FORBIDDEN {
                assert!(!upper.contains(word), "alarmist word in: {s}");
            }
        }
    }

    #[test]
    fn index_stays_in_bounds_and_varies() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..200 {
            let i = index(5);
            assert!(i < 5);
            seen.insert(i);
        }
        // Over 200 draws from a 5-slot pool we expect real spread, not a stuck value.
        assert!(seen.len() >= 2, "index() looks stuck: {seen:?}");
    }

    #[test]
    fn single_element_pool_is_slot_zero() {
        assert_eq!(index(1), 0);
        assert_eq!(index(0), 0);
    }
}
