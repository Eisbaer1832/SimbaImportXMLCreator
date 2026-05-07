use csv::{Reader, ReaderBuilder};
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::{DecodeReaderBytes, DecodeReaderBytesBuilder};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub fn get_csv_reader(csv: PathBuf) -> (String, Reader<BufReader<DecodeReaderBytes<File, Vec<u8>>>>) {
    let file = File::open(csv).expect("Could not open csv file");
    let transcoded_reader = DecodeReaderBytesBuilder::new()
        .encoding(Some(WINDOWS_1252))
        .build(file);
    let mut reader = BufReader::new(transcoded_reader);
    let mut first_line = String::new();
    reader.read_line(&mut first_line).expect("Can't read first line"); // consume the first line, so that the header is read correctly

    let reader = ReaderBuilder::new()
        .delimiter(b';')
        .flexible(true)
        .from_reader(reader);

    (first_line, reader)
}
