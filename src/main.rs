#![windows_subsystem = "windows"]

pub mod ui;
pub mod get_ids;
pub mod readers;
pub mod generate;
mod filter;
pub mod profiles;


use crate::profiles::get_profiles;
use crate::get_ids::get_pdf_ids;
use crate::ui::ui;
use clap::Parser;
use preferences::{AppInfo};

const APP_INFO: AppInfo = AppInfo{name: "SimbaImportCreator", author: "Tino Brinker"};

/// Generator for SIMBA import files
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arguments {
    /// PDF directory
    #[arg(short, long, default_value_t)]
    directory: String,

    /// CSV file
    #[arg(short, long, default_value_t)]
    csv: String,
}


fn resolve_path(path: &str) -> String {
    shellexpand::tilde(path).into_owned()
}

fn setup_panic_handling() {
    std::panic::set_hook(Box::new(|panic_info| {
        let msg = panic_info.to_string();

        rfd::MessageDialog::new()
            .set_title("Etwas ist schiefgelaufen :(")
            .set_description(&format!("{msg}"))
            .set_level(rfd::MessageLevel::Error)
            .set_buttons(rfd::MessageButtons::Ok)
            .show();

        eprintln!("{msg}");
    }));

}

fn main() {
    println!("Creating import files!");
    setup_panic_handling();

    let args = Arguments::parse();
    let pdf_directory = resolve_path(&args.directory);
    let csv_file = resolve_path(&args.csv);

    let profiles = get_profiles().expect("Could not get profiles");

    ui(pdf_directory, csv_file, profiles);
}
