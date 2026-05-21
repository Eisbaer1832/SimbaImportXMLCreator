mod update_files;
mod zip;

use crate::filter::easy_regex;
use crate::generate::update_files::{generate_xml, update_csv};
use crate::generate::zip::zip_directory;
use crate::read_ids::get_pdf_ids;
use std::fs;
use std::path::{Path, PathBuf};

pub fn generate(pdf_path: PathBuf, csv_path: PathBuf, pattern: String) {
    let regex = easy_regex(&*pattern).expect("invalid regex");

    let mut ids: Vec<String> = get_pdf_ids(pdf_path.clone());

    generate_xml(ids.clone(), pdf_path.clone(), regex.clone()).expect("couldn't create xml");

    ids = ids.iter().filter_map(|id| {
        regex.find(id).map(|m| m.as_str().to_string().to_lowercase())
    }).collect();

    let export_path = pdf_path.parent().unwrap().to_path_buf();
    update_csv(ids, csv_path, export_path.clone(), regex.clone()).expect("couldn't update csv");

    let csv_result_name = &export_path.join("EXTF_Import-Buchungsstapel.csv");
    let csv_result_path = Path::new(csv_result_name);
    zip_directory(pdf_path.as_path(), csv_result_path, Path::new(&export_path.join("EXTF_Import-Buchungsstapel.csv.zip")),regex).expect("zipping failed");

    fs::remove_file(csv_result_path).unwrap();
}