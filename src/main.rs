mod update_files;
pub mod ui;

use crate::update_files::{generate_xml, update_csv};
use regex::Regex;
use std::{fs, process};
use std::path::PathBuf;
use crate::ui::ui;

fn main() {
    println!("Creating import files!");
    ui();
}

fn generate(pdf_path: PathBuf, csv_path: PathBuf) {
    let re = Regex::new(r"R\d{4}.pdf").unwrap();
    let paths = fs::read_dir(pdf_path.clone()).unwrap();

    let mut ids: Vec<String> = Vec::new();

    for path in paths {
        let label = path.unwrap().path().display().to_string();
        if re.is_match(&*label) {
            let id = re.find(&*label).unwrap().as_str();
            ids.push(id.to_string());
        }
    }

    generate_xml(ids.clone(), pdf_path.clone()).expect("couldn't create xml");
    update_csv(ids, csv_path, pdf_path).expect("couldn't update csv");
    
    process::exit(0)
}