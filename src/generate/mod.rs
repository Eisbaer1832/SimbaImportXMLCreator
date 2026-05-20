mod update_files;
mod zip;

use crate::filter::easy_regex;
use crate::generate::update_files::{generate_xml, update_csv};
use crate::generate::zip::zip_directory;
use crate::read_ids::get_pdf_ids;
use regex::Regex;
use std::path::{Path, PathBuf};

pub fn generate(pdf_path: PathBuf, csv_path: PathBuf, mut pattern: String) {
    pattern = easy_regex(&*pattern);

    let mut ids: Vec<String> = get_pdf_ids(pdf_path.clone());

    generate_xml(ids.clone(), pdf_path.clone(), pattern.clone()).expect("couldn't create xml");

    let re = Regex::new(pattern.as_str()).expect("invalid regex pattern");
    ids = ids.iter().filter_map(|id| {
        re.find(id).map(|m| m.as_str().to_string())
    }).collect();

    update_csv(ids, csv_path, pdf_path.clone(), pattern).expect("couldn't update csv");

    zip_directory(pdf_path.as_path(), Path::new("out.zip")).expect("zipping failed");
}