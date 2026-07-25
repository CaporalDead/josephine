//! `josephine explain` — what each check watches, why it matters, and how to act.
//!
//! Pure rendering: the copy lives in `josephine_core::remedy`, shared with the
//! remedies `doctor` prints.

use anyhow::Result;
use josephine_core::i18n::{self, Lang};
use josephine_core::remedy::{Advice, advice, all};

use crate::output::{check_label, sober_header};

pub fn run(check: Option<&str>) -> Result<()> {
    sober_header(Some(i18n::t("explain", "explain")), None);

    match check {
        None => print_list(),
        Some(name) => match advice(name) {
            Some(entry) => print_detail(entry),
            None => print_unknown(name),
        },
    }

    Ok(())
}

fn print_list() {
    println!(
        "{}",
        i18n::t(
            "What Joséphine watches — one line each. Detail: `josephine explain <check>`.",
            "Ce que Joséphine surveille — une ligne chacun. Détail : `josephine explain <check>`.",
        )
    );
    println!();
    for entry in all() {
        let label = check_label(entry.name);
        let what = i18n::t(entry.what.0, entry.what.1);
        println!("  {label} ({}) — {what}", entry.name);
    }
}

fn print_detail(entry: &Advice) {
    let label = check_label(entry.name);
    println!("{label} ({})", entry.name);
    println!();
    println!(
        "{} {}",
        i18n::t("What:", "Quoi :"),
        i18n::t(entry.what.0, entry.what.1)
    );
    println!(
        "{} {}",
        i18n::t("Why:", "Pourquoi :"),
        i18n::t(entry.why.0, entry.why.1)
    );
    println!(
        "{} {}",
        i18n::t("Remedy:", "Remède :"),
        i18n::t(entry.remedy.0, entry.remedy.1)
    );
}

fn print_unknown(name: &str) {
    let names: Vec<&str> = all().iter().map(|a| a.name).collect();
    match i18n::lang() {
        Lang::En => {
            println!("Unknown check \"{name}\". Known checks:");
            for n in &names {
                println!("  {n}");
            }
        }
        Lang::Fr => {
            println!("Check inconnu « {name} ». Checks connus :");
            for n in &names {
                println!("  {n}");
            }
        }
    }
}
