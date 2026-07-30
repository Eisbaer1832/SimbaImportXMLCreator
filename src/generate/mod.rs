mod update_files;
pub mod zip;

use std::string::String;
use crate::filter::easy_regex;
use crate::generate::update_files::{generate_xml, update_csv, update_scs};
use crate::generate::zip::zip_directory;
use crate::get_ids::get_pdf_ids;
use std::fs;
use std::path::{Path, PathBuf};

pub fn generate(pdf_path: PathBuf, csv_path: PathBuf, pattern: String, column: String) -> (usize, usize, i32) {
    let regex = easy_regex(&*pattern).expect("invalid regex");
    let mut ids: Vec<String> = get_pdf_ids(pdf_path.clone());

    // statistics
    let total_lines:usize;
    let matched_lines:i32;
    let pdfs_total = ids.len();
    let pdfs_matched:usize;
    
    generate_xml(ids.clone(), pdf_path.clone(), regex.clone()).expect("couldn't create xml");

    ids = ids.iter().filter_map(|id| {
        regex.find(id).map(|m| m.as_str().to_string().to_lowercase())
    }).collect();
    pdfs_matched = ids.len();
    
    let export_path = pdf_path.parent().unwrap().to_path_buf();

    let name:&str;
    let mut is_csv = true;


    if csv_path.extension() == Some(std::ffi::OsStr::new("csv")) {
        (total_lines, matched_lines) = update_csv(ids, csv_path, export_path.clone(), regex.clone(), column).expect("couldn't update csv");
        name = "EXTF_Import-Buchungsstapel.csv"
    }else {
        is_csv = false;
        (total_lines, matched_lines) =  update_scs(ids, csv_path, export_path.clone(), regex.clone()).expect("couldn't update scs");
        name = "Import-Buchungsstapel.scs"
    }

    println!("----{name}------");
    println!("Es wurden {} Buchungssätze gefunden, davon haben {} einen Beleglink!", total_lines, matched_lines);
    println!("Von den {} Pdfs, wurden bei {} eine Übereinstimmung mit dem Muster festgestellt!", pdfs_total, pdfs_matched);

    let csv_result_name = &export_path.join(name);
    let csv_result_path = Path::new(csv_result_name);

    let result_file_name = export_path.join(format!("{}.zip", name));
    let result_path = Path::new(&result_file_name);
    zip_directory(pdf_path.as_path(), csv_result_path, result_path,regex, is_csv).expect("zipping failed");

    fs::remove_file(csv_result_path).unwrap();
    showfile::show_path_in_file_manager(result_path);

    (total_lines, pdfs_total, matched_lines)
}