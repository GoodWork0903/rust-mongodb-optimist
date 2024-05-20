use crate::{whois::WhoIsRecord, Result};
use log::info;
use std::io::{Cursor, Read};
use zip::read::ZipArchive;

/// Download the zip file and extract the contents and
/// save the records to the database.
pub async fn fetch(zip_url: &str) -> Result<Vec<WhoIsRecord>> {
    // Download the zip file
    info!("Downloading from: {}", zip_url);
    let zip_file = reqwest::get(zip_url)
        .await
        .expect("Failed to download the zip file");
    let zip_file_contents = zip_file.bytes().await.expect("Failed to read the zip file");
    // Extract the downloaded zip file
    info!("Downloaded ZIP file, extracting...");
    let mut zip_archive = ZipArchive::new(Cursor::new(zip_file_contents))?;

    info!("Number of files in the archive: {}", zip_archive.len());
    let mut records: Vec<WhoIsRecord> = Vec::new();
    for i in 0..zip_archive.len() {
        let mut file = zip_archive.by_index(i)?;
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents)?;

        records.append(&mut WhoIsRecord::from_buffer(&file_contents)?);
    }

    Ok(records)
}
