use crate::utils::get_csv_reader;
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
