//! Minimal runtime internationalisation.
//!
//! English is the default; French is opt-in via `language: fr` in the config.
//! The active language is a process-wide setting applied once at startup from
//! the loaded config, so every thread (CLI or daemon task) renders alike.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};

/// The languages Joséphine can speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    En,
    Fr,
}

static CURRENT: AtomicU8 = AtomicU8::new(Lang::En as u8);

/// Apply the active language process-wide (called once at startup).
pub fn set_lang(lang: Lang) {
    CURRENT.store(lang as u8, Ordering::Relaxed);
}

/// The active language (English until [`set_lang`] is called).
pub fn lang() -> Lang {
    if CURRENT.load(Ordering::Relaxed) == Lang::Fr as u8 {
        Lang::Fr
    } else {
        Lang::En
    }
}

/// Choose between an English and a French string literal for the active language.
pub fn t(en: &'static str, fr: &'static str) -> &'static str {
    match lang() {
        Lang::En => en,
        Lang::Fr => fr,
    }
}

/// Serialises every test — in this crate or in any dependent crate's own
/// test binary — that reads or mutates the active language.
///
/// [`lang`]/[`set_lang`] are one process-wide [`AtomicU8`], on purpose: the
/// whole app (CLI, daemon threads, ...) must agree on the same language from
/// a single `set_lang` call at startup, with nothing to thread through every
/// call site. Rust's test harness, though, runs a crate's tests concurrently
/// by default, and a test that pins the language before asserting an English
/// or French string can race another test doing the same for the other
/// language — a bare atomic load racing a store, no lock in between. The
/// failure is flaky and reproduces on maybe one run in a hundred, pointing
/// nowhere near the actual bug it's masking.
///
/// Any test that reads or writes the active language must hold this guard
/// for as long as the language has to stay put — one assertion, or a loop
/// walking every [`Lang`] variant. It is deliberately a normal public item,
/// not `#[cfg(test)]`-gated: `cfg(test)` only applies within this crate's own
/// test build, so a dependent crate's tests — which link `josephine-core` as
/// an ordinary library — could never see it if it were gated. Do not delete
/// this as unused dead weight; it exists to be called from test code in
/// other crates.
///
/// Poison-tolerant: a panic while holding the guard (an assertion failing
/// inside the locked section, say) must not fail every later language test
/// with a poisoned-lock error and bury the real failure, so a poisoned lock
/// is recovered rather than unwrapped.
pub fn lock_for_test() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_uses_lowercase_codes() {
        assert_eq!(serde_yaml::to_string(&Lang::Fr).unwrap().trim(), "fr");
        assert_eq!(serde_yaml::from_str::<Lang>("en").unwrap(), Lang::En);
    }

    #[test]
    fn defaults_to_english() {
        // No test sets the global language, so it stays at its English default.
        assert_eq!(Lang::default(), Lang::En);
        assert_eq!(t("hello", "bonjour"), "hello");
    }
}
