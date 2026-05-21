pub mod ui;
pub mod read_ids;
pub mod utils;
pub mod generate;
mod filter;
pub mod profiles;


use crate::profiles::get_profiles;
use crate::read_ids::get_pdf_ids;
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


fn main() {
    println!("Creating import files!");

    let args = Arguments::parse();
    let pdf_directory = resolve_path(&args.directory);
    let csv_file = resolve_path(&args.csv);

    let profiles = get_profiles().expect("Could not get profiles");

    ui(pdf_directory, csv_file, profiles);
}
