mod update_files;
pub mod ui;
pub mod read_ids;
pub mod utils;
pub mod filter;
pub mod zip;

use std::fs::File;
use crate::read_ids::get_pdf_ids;
use crate::ui::ui;
use crate::update_files::{generate_xml, update_csv};
use std::path::{Path, PathBuf};
use std::process;
use regex::Regex;
use zip::ZipWriter;
use crate::zip::zip_directory;

fn main() {
    println!("Creating import files!");
    ui();
}

fn generate(pdf_path: PathBuf, csv_path: PathBuf, pattern: String) {
    let mut ids: Vec<String> = get_pdf_ids(pdf_path.clone());

    generate_xml(ids.clone(), pdf_path.clone(), pattern.clone()).expect("couldn't create xml");

    let re = Regex::new(pattern.as_str()).expect("invalid regex pattern");
    ids = ids.iter().filter_map(|id| {
        re.find(id).map(|m| m.as_str().to_string())
    }).collect();

    update_csv(ids, csv_path, pdf_path.clone(), pattern).expect("couldn't update csv");

    zip_directory(pdf_path.as_path(), Path::new("out.zip")).expect("zipping failed");
    process::exit(0)
}