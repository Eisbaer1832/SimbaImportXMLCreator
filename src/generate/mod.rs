mod update_files;
mod zip;

use std::string::String;
use crate::filter::easy_regex;
use crate::generate::update_files::{generate_xml, update_csv, update_scs};
use crate::generate::zip::zip_directory;
use crate::get_ids::get_pdf_ids;
use std::fs;
use std::path::{Path, PathBuf};

pub fn generate(pdf_path: PathBuf, csv_path: PathBuf, pattern: String, column: String) {
    let regex = easy_regex(&*pattern).expect("invalid regex");

    let mut ids: Vec<String> = get_pdf_ids(pdf_path.clone());

    generate_xml(ids.clone(), pdf_path.clone(), regex.clone()).expect("couldn't create xml");

    ids = ids.iter().filter_map(|id| {
        regex.find(id).map(|m| m.as_str().to_string().to_lowercase().replace("_", " "))
    }).collect();

    let export_path = pdf_path.parent().unwrap().to_path_buf();

    let name:&str;
    let mut is_csv = true;

    if csv_path.extension() == Some(std::ffi::OsStr::new("csv")) {
        update_csv(ids, csv_path, export_path.clone(), regex.clone(), column).expect("couldn't update csv");
        name = "EXTF_Import-Buchungsstapel.csv"
    }else {
        is_csv = false;
        update_scs(ids, csv_path, export_path.clone(), regex.clone()).expect("couldn't update scs");
        name = "Import-Buchungsstapel.scs"
    }

    let csv_result_name = &export_path.join(name);
    let csv_result_path = Path::new(csv_result_name);

    let result_file_name = export_path.join(format!("{}.zip", name));
    let result_path = Path::new(&result_file_name);
    zip_directory(pdf_path.as_path(), csv_result_path, result_path,regex, is_csv).expect("zipping failed");

    fs::remove_file(csv_result_path).unwrap();
    showfile::show_path_in_file_manager(result_path);
}