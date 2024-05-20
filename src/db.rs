use crate::whois::WhoIsRecord;
use log::{debug, info};
use mongodb::{
    bson,
    bson::{doc, Document},
    options::{InsertManyOptions, UpdateOptions, DeleteOptions},
    sync::Client,
};
use tokio::task::spawn_blocking;

pub(crate) async fn upsert(
    client: Client,
    database: &str,
    collection: &str,
    records: Vec<WhoIsRecord>,
) -> crate::Result<()> {
    //! Save the `whois` records to a MongoDB database.
    //! This function basically inserts the records into the collection but
    //! if the record already exists in the collection, it updates the record.
    let db = client.database(database);
    let collection = db.collection::<Document>(collection);

    let mut documents = Vec::new();
    let mut domain_names_filter = Vec::new();
    for record in records {
        let document = doc! {
            "domain_name": &record.domain_name,
            "domain_keyword": &record.domain_keyword,
            "domain_tld": &record.domain_tld,
            "query_time": &record.query_time,
            "create_date": &record.create_date,
            "update_date": &record.update_date,
            "expiry_date": &record.expiry_date,
            "registrar_iana": &record.registrar_iana,
            "registrar_name": &record.registrar_name,
            "registrar_website": &record.registrar_website,
            "registrant_name": &record.registrant_name,
            "registrant_company": &record.registrant_company,
            "registrant_address": &record.registrant_address,
            "registrant_city": &record.registrant_city,
            "registrant_state": &record.registrant_state,
            "registrant_zip": &record.registrant_zip,
            "registrant_country": &record.registrant_country,
            "registrant_phone": &record.registrant_phone,
            "registrant_fax": &record.registrant_fax,
            "registrant_email": &record.registrant_email,
            "name_servers": &record.name_servers
        };
        domain_names_filter.push(record.domain_name.clone());
        documents.push(document);
    }

    let options = InsertManyOptions::builder().build();
    let delete_options = DeleteOptions::builder().build();

    let result = spawn_blocking(move || {
        collection.delete_many(doc! {"domain_name" : {
            "$in" : domain_names_filter
        }}, None).ok();
        collection.insert_many(documents, options).ok();
    }).await?;
    info!("Successfully saved  records to the database");
    Ok(())
}