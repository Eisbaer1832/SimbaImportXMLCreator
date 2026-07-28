use std::io::Write;
use std::fs::{write, File};
use std::io;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use regex::Regex;
use zip::read::ZipFile;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};
use crate::generate;
use crate::generate::zip::{add_dir_to_zip, unzip_with_interrupt, zip_directory};
use crate::ui::MatchedPattern;

pub fn cleanup_pds(path: PathBuf) {
    unzip_with_interrupt(path.clone(), |file, path | remove_kst_ktr(file, path), String::from(".xml"));

    let zip_path = path.parent().unwrap().join("Importiermich.zip");
    let zip_file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(zip_file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    path.parent().unwrap().join("temp").read_dir().unwrap().for_each(|entry| {

        let p = entry.unwrap().path();
        let file = File::open(&p).unwrap();
        let file_name = p.file_name().unwrap().to_str().unwrap();
        zip.start_file(file_name, options).expect("can't create zip file");
        let mut buffer = Vec::new();
        io::copy(&mut file.take(u64::MAX), &mut buffer).unwrap();
        zip.write_all(&buffer).unwrap();
    });

    zip.finish().expect("cant finish zip");

    showfile::show_path_in_file_manager(zip_path);
}

fn remove_kst_ktr(mut f: ZipFile, p: PathBuf) {
    let mut buffer = String::new();
    f.read_to_string(&mut buffer).expect("Can't read xml");

    let p1 = Regex::new("<costCategoryId>[a-zA-Z0-9]*</costCategoryId>");
    buffer = p1.unwrap().replace(&*buffer, "").into();
    let p1 = Regex::new("<costCategoryId2>[a-zA-Z0-9]*</costCategoryId2>");
    buffer = p1.unwrap().replace(&*buffer, "").into();

    let mut out = File::create(p).unwrap();
    write!(out, "{}", buffer).expect("Can't create xml");

}

