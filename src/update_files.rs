use crate::utils::get_csv_reader;
use chrono::Local;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

pub fn update_csv(ids: Vec<String>, csv:PathBuf, out_dir:PathBuf, pattern: String) -> Result<(), Box<dyn Error>> {
    let (first_line, mut reader) = get_csv_reader(csv);

    let headers = reader.headers()?.clone();

    let mut raw_writer = BufWriter::new(File::create(out_dir.display().to_string() + "/Buchungsstapel.csv")?);
    raw_writer.write(first_line.as_bytes())?; // add the metadata line to the new file
    let mut wtr = csv::Writer::from_writer(raw_writer);
    wtr.write_record(&headers)?;

    for result in reader.records() {
        match result {
            Ok(record) => {
                let mut new_record = record.clone();

                // get Buchungstext
                let buchungs_text_index = headers.iter().position(|h| h == "Buchungstext").expect("Konnte keinen Buchungstext header finden");
                let text : String = record.get(buchungs_text_index).expect("Kein Buchungstext vorhanden").to_string();

                let re = Regex::new(&*pattern)?;
                let id = re.find(text.as_str())
                    .expect("cant get string").as_str().to_string();


                // if the id corresponds to a PDF, add the PDF link
                if ids.contains(&(id.clone())) {
                    println!("found: {} {}", pattern, id);

                    let link_index = headers.iter().position(|h| h == "Beleglink").unwrap();
                    let t = format!("BEDI \"{}\"", id);
                    new_record = record
                        .iter()
                        .enumerate()
                        .map(|(i,val)| if i == link_index {t.clone()} else { val.to_string() })
                        .collect();
                }
                wtr.write_record(&new_record)?;
            },
            Err(err) => println!("{}", err)
        }
    }
    wtr.flush()?;
    Ok(())
}


pub fn generate_xml(ids: Vec<String>, pdf_path: PathBuf, pattern: String) -> Result<(), Box<dyn Error>> {
    let file = File::create(pdf_path.display().to_string() + "/document.xml")?;
    let writer = BufWriter::new(file);

    let mut writer = Writer::new_with_indent(writer, b' ', 2);

    // open archive
    let mut elem = BytesStart::new("archive");
    elem.push_attribute(("xmlns", "http://xml.datev.de/bedi/tps/document/v05.0"));
    elem.push_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"));
    elem.push_attribute(("version", "5.0"));
    elem.push_attribute(("generatingSystem", "lexware.de"));
    elem.push_attribute(("xsi:schemaLocation", "http://xml.datev.de/bedi/tps/document/v05.0 Document_v050.xsd"));
    writer.write_event(Event::Start(elem))?;

    // set date and time info
    let time = Local::now();
    writer.write_event(Event::Start(BytesStart::new("header")))?;
    writer.create_element("date")
        .write_text_content(BytesText::new(&*time.format("%Y-%d-%mT%H:%M:%S").to_string()))?;
    writer.write_event(Event::End(BytesEnd::new("header")))?;


    // add actual PDF links
    writer.write_event(Event::Start(BytesStart::new("content")))?;
    for id in ids {
        let id_extensionless =  Regex::new(&*pattern)?.find(&*id).unwrap().as_str();

        let mut document = BytesStart::new("document");
        document.push_attribute(("guid", id_extensionless));
        writer.write_event(Event::Start(document))?;

        writer.create_element("extension")
            .with_attribute(("xsi:type", "File"))
            .with_attribute(("name", id.as_str()))
            .write_empty()?;

        writer.write_event(Event::End(BytesEnd::new("document")))?;

    }
    writer.write_event(Event::End(BytesEnd::new("content")))?;


    //close archive
    writer.write_event(Event::End(BytesEnd::new("archive")))?;
    writer.into_inner().flush()?;
    Ok(())
}