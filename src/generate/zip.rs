use regex::Regex;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
pub(crate) use zip::{write::FileOptions, ZipWriter};
use zip::read::ZipFile;

pub fn zip_directory(src_dir: &Path, csv_file: &Path, dst_file: &Path, pattern: Regex, is_csv: bool) -> io::Result<()> {
    let file = File::create(dst_file)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    add_csv(csv_file,&mut zip,options).expect("writing csv failed");

    if is_csv {
        zip.add_directory("Belege/".to_string(), Default::default())?;
    }

    add_dir_to_zip(&mut zip, src_dir, src_dir, &options,pattern, is_csv)?;
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

pub fn add_dir_to_zip<W: Write + io::Seek>(
    zip: &mut ZipWriter<W>,
    base: &Path,
    current: &Path,
    options: &FileOptions,
    pattern: Regex,
    is_csv: bool,
) -> io::Result<()> {

    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();

        let name = path.strip_prefix(base).unwrap().to_string_lossy();

        let mut subdir = "Belege/";
        if !is_csv {
            subdir = "";
        }
        if path.is_dir() {
            zip.add_directory(format!("Belege/{}/", name), *options)?;
            add_dir_to_zip(zip, base, &path, options, pattern.clone(), is_csv)?; // recursion
        } else {
            if name.ends_with("pdf") || name.ends_with("PDF") {
                let n = pattern.find(&*name).map(|name| name.as_str()).unwrap_or(&*name);
                zip.start_file(subdir.to_owned() + &*n.to_owned().to_lowercase() + ".pdf", *options)?;
            }else {
                zip.start_file(subdir.to_owned() + &*name.into_owned(), *options)?;
            }

            let mut f = File::open(&path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            zip.write_all(&buf)?;
        }
    }
    Ok(())
}

pub fn unzip_with_interrupt(path:PathBuf, interrupt: fn(file: ZipFile, outpath: PathBuf), at: String) {
    let file = File::open(path.clone()).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let output_dir = path.parent().unwrap().join("temp").to_path_buf();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(p) => output_dir.join(p),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).unwrap();
                }
            }
            if file.name().ends_with(at.as_str()) {
                interrupt(file, outpath);
            }else {
                let mut outfile = File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }
        }
    }
}