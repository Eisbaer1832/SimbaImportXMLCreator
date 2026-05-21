use regex::Regex;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
pub(crate) use zip::{write::FileOptions, ZipWriter};

pub fn zip_directory(src_dir: &Path, csv_file: &Path, dst_file: &Path, pattern: Regex) -> io::Result<()> {
    let file = File::create(dst_file)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    add_csv(csv_file,&mut zip,options).expect("writing csv failed");

    zip.add_directory("Belege/".to_string(), Default::default())?;
    add_dir_to_zip(&mut zip, src_dir, src_dir, &options,pattern)?;
    zip.finish()?;
    Ok(())
}

fn add_csv(csv_file: &Path, zip: &mut ZipWriter<File>, options: FileOptions) -> io::Result<()> {
    let name = csv_file.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("default.txt")
        .to_string();

    zip.start_file(name, options)?;
    let mut f = File::open(&csv_file)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    zip.write_all(&buf)?;
    Ok(())
}

fn add_dir_to_zip<W: Write + io::Seek>(
    zip: &mut ZipWriter<W>,
    base: &Path,
    current: &Path,
    options: &FileOptions,
    pattern: Regex,
) -> io::Result<()> {

    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();

        let name = path.strip_prefix(base).unwrap().to_string_lossy();

        if path.is_dir() {
            zip.add_directory(format!("Belege/{}/", name), *options)?;
            add_dir_to_zip(zip, base, &path, options,pattern.clone())?; // recursion
        } else {
            if name.ends_with("pdf") || name.ends_with("PDF") {
                let n = pattern.find(&*name).map(|name| name.as_str()).unwrap();
                zip.start_file("Belege/".to_owned() + &*n.to_owned().to_lowercase() + ".pdf", *options)?;
            }else {
                zip.start_file("Belege/".to_owned() + &*name.into_owned(), *options)?;
            }

            let mut f = File::open(&path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            zip.write_all(&buf)?;
        }
    }
    Ok(())
}