use crate::readers::{get_csv_reader, get_scs_reader};
use chrono::Local;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use regex::{Regex};
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use csv::{QuoteStyle, WriterBuilder};

pub fn update_csv(ids: Vec<String>, csv:PathBuf, out_dir:PathBuf, pattern: Regex, column: String) -> Result<(), Box<dyn Error>> {
    let (first_line, mut reader) = get_csv_reader(csv);

    let headers = reader.headers()?.clone();

    let mut raw_writer = BufWriter::new(File::create(out_dir.display().to_string() + "/EXTF_Import-Buchungsstapel.csv")?);
    raw_writer.write(first_line.as_bytes())?; // add the metadata line to the new file
    let mut wtr = WriterBuilder::new()
        .delimiter(b';')
        .quote_style(QuoteStyle::NonNumeric)
        .from_writer(raw_writer);
    wtr.write_record(&headers)?;

    for result in reader.records() {
        match result {
            Ok(record) => {
                let mut new_record = record.clone();

                // get Buchungstext
                let buchungs_text_index = headers.iter().position(|h| h == column).expect("Konnte keinen Buchungstext header finden");
                let text : String = record.get(buchungs_text_index).expect("Kein Buchungstext vorhanden").to_string();

                let id = pattern
                    .find(text.as_str())
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| {
                        println!("Can't find match in {}", text);
                        text.clone()
                    })
                    .to_lowercase();

                // if the id corresponds to a PDF, add the PDF link
                if ids.contains(&(id.clone())) {
                    println!("found: {} {}", pattern, id);

                    let link_index = headers.iter().position(|h| h == "Beleglink").unwrap();
                    let t = format!("{}.pdf", id);
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


pub fn update_scs(ids: Vec<String>, csv:PathBuf, out_dir:PathBuf, pattern: Regex) -> Result<(), Box<dyn Error>> {
    let (first_line, mut reader) = get_scs_reader(csv);

    let headers = reader.headers()?.clone();

    let mut raw_writer = BufWriter::new(File::create(out_dir.display().to_string() + "/Import-Buchungsstapel.scs")?);
    raw_writer.write(first_line.as_bytes())?; // add the metadata line to the new file
    let mut wtr = WriterBuilder::new()
        .delimiter(b';')
        .quote_style(QuoteStyle::Never)
        .from_writer(raw_writer);
    wtr.write_record(&headers)?;

    println!("{:?}", ids);
    for result in reader.records() {
        match result {
            Ok(record) => {
                let mut new_record = record.clone();

                // get Buchungstext
                let mut id = [13, 12]
                    .iter()
                    .filter_map(|&col| record.get(col))
                    .find_map(|val| pattern.find(val).map(|m| m.as_str().to_lowercase()))
                    .unwrap_or("".to_string());
                id = id.replace("_", " ");

                println!("{}", id);
                // if the id corresponds to a PDF, add the PDF link
                if ids.contains(&(id.clone())) {
                    println!("found: {} {}", pattern, id);

                    let link_index = 28;
                    let t = format!("{}.pdf", id);
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


pub fn generate_xml(ids: Vec<String>, pdf_path: PathBuf, pattern: Regex) -> Result<(), Box<dyn Error>> {
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
        println!("{}", id);
        let id_extensionless = pattern.find(&*id)
            .map(|m| m.as_str().to_lowercase())
            .unwrap_or_else(|| id.to_lowercase());

        let mut document = BytesStart::new("document");
        document.push_attribute(("guid", id_extensionless.as_str()));
        writer.write_event(Event::Start(document))?;


        let name = format!("{id_extensionless}.pdf");
        writer.create_element("extension")
            .with_attribute(("xsi:type", "File"))
            .with_attribute(("name", name.as_str()))
            .write_empty()?;

        writer.write_event(Event::End(BytesEnd::new("document")))?;

    }
    writer.write_event(Event::End(BytesEnd::new("content")))?;

    //close archive
    writer.write_event(Event::End(BytesEnd::new("archive")))?;
    writer.into_inner().flush()?;
    Ok(())
}