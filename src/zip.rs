use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
pub(crate) use zip::{write::FileOptions, ZipWriter};

pub fn zip_directory(src_dir: &Path, dst_file: &Path) -> io::Result<()> {
    let file = File::create(dst_file)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    add_dir_to_zip(&mut zip, src_dir, src_dir, &options)?;
    zip.finish()?;
    Ok(())
}

fn add_dir_to_zip<W: Write + io::Seek>(
    zip: &mut ZipWriter<W>,
    base: &Path,
    current: &Path,
    options: &FileOptions,
) -> io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();

        let name = path.strip_prefix(base).unwrap().to_string_lossy();

        if path.is_dir() {
            zip.add_directory(format!("{}/", name), *options)?;
            add_dir_to_zip(zip, base, &path, options)?; // recursion
        } else {
            zip.start_file(name.into_owned(), *options)?;
            let mut f = File::open(&path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            zip.write_all(&buf)?;
        }
    }
    Ok(())
}