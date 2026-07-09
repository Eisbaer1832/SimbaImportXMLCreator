use std::fs;
use reqwest;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use base64::{engine::general_purpose, Engine as _};
use smb::{Client, ClientConfig, Connection, FileAccessMask, FileCreateArgs, UncPath, WriteAt};
use smb::binrw_util::boolean::Boolean;

pub async fn instant_import(mandant: String, path: &Path) {
    upload_file(path.clone().into()).await.expect("failed to upload file");
    simba_start_import(mandant,path).await;
}

async fn upload_file(path: PathBuf) -> Result<(), Boolean> {
    let client = Client::new(ClientConfig::default());

    // Connect to a share
    let target_path = UncPath::from_str(r"\\192.168.2.8\sbcache\").unwrap();
    client.share_connect(&target_path, "kltogo@kanzleilotz.de", "KLTG2019#".to_string()).await.expect("can't connect to smb share");


    // create empty file
    let file_to_open = target_path.with_path("TEMP/EXTF_Import-Buchungsstapel.csv.zip");
    let file_open_args = FileCreateArgs::make_open_existing(FileAccessMask::new().with_generic_read(true));
    let file = client.create_file(&file_to_open, &file_open_args).await.unwrap().unwrap_file();

    let bytes = fs::read(path).unwrap();
    file.write_at(&bytes, 0).await;
    Ok(())
}

async fn simba_start_import(mandant: String, path: &Path) {
    println!("Importing {mandant} with {path:?}");

    let url = "http://192.168.2.8:8972/csp/simba/Datenverwaltung.BuchungsdatenImportieren.CLS";

    let filename = path.file_name().unwrap().to_str().unwrap();

    let soap_body = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
        <soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/"
                       xmlns:imp="http://www.simba.de/import">
          <soapenv:Body>
            <imp:importDATEV>
              <imp:filename>{}</imp:filename>
              <imp:MDNR>{}</imp:MDNR>
            </imp:importDATEV>
          </soapenv:Body>
        </soapenv:Envelope>"#,
        filename,
        mandant
    );


    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header("Content-Type", "text/xml; charset=utf-8")
        .header(
            "SOAPAction",
            "\"http://www.simba.de/import/Datenverwaltung.BuchungsdatenImportieren.importDATEV\"",
        )
        .body(soap_body)
        .send()
        .await
        .unwrap();

    let status = response.status();
    let text = response.text().await.unwrap();

    println!("Status: {status}");
    println!("Response: {text}");
}