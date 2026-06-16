use crate::readers::{get_csv_reader, get_scs_reader};
use std::fs;
use std::path::PathBuf;

pub fn get_pdf_ids(pdf_path: PathBuf) -> Vec<String> {
    let paths = fs::read_dir(pdf_path.clone()).unwrap();
    let mut ids: Vec<String> = Vec::new();

    for path in paths {
        let label = path.unwrap().file_name().into_string().unwrap();
        if label.ends_with(".pdf") {
            ids.push(label);
        } else {}
    }

    ids
}

// wrapper for scs/csv id fetching
pub fn get_buchungsstapel_ids(path: PathBuf, column: String) -> Vec<String> {
    if path.extension() == Some(std::ffi::OsStr::new("csv")) {
        get_csv_ids(path, column)
    }else  {
        get_scs_ids(path)
    }
}


// assumes the scs file is following the spec provided by https://www.gem-gruppe.de/download/simba_software_support?file=files/downloads/simba/Simba-Schnittstellenbeschreibung.pdf&cid=7883
pub fn get_scs_ids(path: PathBuf) -> Vec<String> {
    let (_first_line, mut reader) = get_scs_reader(path);
    let mut ids: Vec<String> = Vec::new();

    reader.records();

    for result in reader.records() {
        println!("{:#?}", result);
        if let Some(value) = result.unwrap().get(13) {
            ids.push(value.to_string());
        }
    }
    ids
}

pub fn get_csv_ids(csv_path: PathBuf, column: String) -> Vec<String> {
    let (_first_line, mut reader) = get_csv_reader(csv_path);
    let mut ids: Vec<String> = Vec::new();

    reader.records();

    // Get headers to find column index
    let headers = reader.headers().expect("can't get headers").clone();
    let col_index = headers
        .iter()
        .position(|h| h == column)
        .expect(&*(column + " konnte nicht gefunden werden"));

    for result in reader.records() {
        if let Some(value) = result.unwrap().get(col_index) {
            ids.push(value.to_string());
        }
    }
    ids
}
