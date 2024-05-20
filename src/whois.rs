use crate::{db, Result, MONGODB_COLLECTION, MONGODB_DB, MONGO_CLIENT};
use log::info;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use tokio::spawn;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct WhoIsRecord {
    pub num: u32,
    pub domain_name: String,
    pub domain_keyword: String,
    pub domain_tld: String,
    pub query_time: String,
    pub create_date: Option<String>,
    pub update_date: Option<String>,
    pub expiry_date: Option<String>,
    pub registrar_iana: Option<String>,
    pub registrar_name: Option<String>,
    pub registrar_website: Option<String>,
    pub registrant_name: Option<String>,
    pub registrant_company: Option<String>,
    pub registrant_address: Option<String>,
    pub registrant_city: Option<String>,
    pub registrant_state: Option<String>,
    pub registrant_zip: Option<String>,
    pub registrant_country: Option<String>,
    pub registrant_phone: Option<String>,
    pub registrant_fax: Option<String>,
    pub registrant_email: Option<String>,
    pub name_servers: Option<String>,
}

impl WhoIsRecord {
    /// Read the csv file and return a vector of `WhoIsRecord`
    pub fn from_file(path: &str) -> Result<Vec<Self>> {
        Ok(csv_de(&read_to_string(path)?)?)
    }

    /// Read the csv buffer and return a vector of `WhoIsRecord`
    pub fn from_buffer(buffer: &str) -> Result<Vec<Self>> {
        Ok(csv_de(buffer)?)
    }

    pub async fn save(records: Vec<WhoIsRecord>) {
        //! Save the records to the database.
        let chunked_records = records.chunks(5000).map(|x| x.to_vec()).collect::<Vec<_>>();
        let mut handles = Vec::new();

        info!(
            "Saving {} records to the database. This may take a while...",
            records.len()
        );
        for records in chunked_records {
            let mongo_client_ref = MONGO_CLIENT.clone();
            handles.push(spawn(async move {
                let mongo_client_ref = mongo_client_ref.clone();
                // save the records
                db::upsert(
                    mongo_client_ref.clone(),
                    MONGODB_DB.as_str(),
                    MONGODB_COLLECTION.as_str(),
                    records.to_vec(),
                )
                .await
                .unwrap();
                info!("Saved {} records to the database", records.len());
            }));
        }

        futures::future::join_all(handles).await;
    }
}

/// deserialize the csv text into a vector of `WhoIsRecord`
fn csv_de(csv_text: &str) -> std::result::Result<Vec<WhoIsRecord>, csv::Error> {
    csv::Reader::from_reader(csv_text.as_bytes())
        .deserialize()
        .collect()
}
