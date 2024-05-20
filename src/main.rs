use chrono::{offset::Local, Duration};
use config::CliOpts;
use lazy_static::lazy_static;
use log::{error, info};
use mongodb::sync::Client;

use std::{process::exit, time::Instant};
use tokio::spawn;
use whois::WhoIsRecord;

#[cfg(not(target_os = "windows"))]
use jemallocator::Jemalloc;

#[cfg(not(target_os = "windows"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod config;
mod daily;
mod db;
mod error;
mod whois;

type Result<T> = std::result::Result<T, error::Error>;

lazy_static! {
    static ref CLI_OPTS: CliOpts = CliOpts::parse_cli();

    static ref MONGODB_URL: String = if CLI_OPTS.mongo_user.is_empty()
        && CLI_OPTS.mongo_password.is_empty()
    {
        format!(
            "mongodb://{}:{}",
            CLI_OPTS.mongo_host,
            CLI_OPTS.mongo_port
        )
    } else {
        format!(
            "mongodb://{}:{}@{}:{}",
            CLI_OPTS.mongo_user,
            CLI_OPTS.mongo_password,
            CLI_OPTS.mongo_host,
            CLI_OPTS.mongo_port
        )
    };
    static ref MONGODB_DB: String = CLI_OPTS.mongo_db.clone();
    static ref MONGODB_COLLECTION: String = CLI_OPTS.mongo_collection.clone();
    static ref DEBUG: bool = CLI_OPTS.debug;

    // Number of threads to use (defaults to 512)
    static ref THREADS: usize = CLI_OPTS.threads;

    // Read from CSV files
    static ref CSV_FILES_PATH: String = CLI_OPTS.csv_files_path.to_owned();

    // Get daily whois data if --daily flag is set
    static ref DAILY: bool = CLI_OPTS.daily;
    static ref DOWNLOAD_URL: String = format!(
                "https://bigdomaindata.s3.amazonaws.com/updates/{}_{}.zip",
                CLI_OPTS.download_code,
                (Local::now() - Duration::days(1)).format("%Y-%m-%d"));

    static ref MONGO_CLIENT: Client = Client::with_uri_str(MONGODB_URL.as_str()).unwrap();
}

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .max_blocking_threads(*THREADS)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let start = Instant::now();
        info!("Starting the program...");

        if DEBUG.to_owned() {
            simple_logger::init_with_level(log::Level::Debug).ok();
            info!("Debug mode enabled.");
        } else {
            simple_logger::init_with_level(log::Level::Info).ok();
        }

        if DAILY.to_owned() {
            info!("Fetching daily whois data...");
            if CLI_OPTS.download_code.is_empty() {
                error!("Please provide a download code.");
                exit(1);
            }
            let records = daily::fetch(&DOWNLOAD_URL.to_owned()).await.unwrap();
            info!("Fetched {} records.", records.len());
            WhoIsRecord::save(records).await;
        } else if let Err(e) = read_directory(&CSV_FILES_PATH.to_owned()).await {
            eprintln!("Error reading the directory: {:?}", e);
            exit(1);
        }
        info!("Reading from CSV and writing into Mongo: Success.");
        // time elapsed
        info!("Elapsed time: {:?} seconds", start.elapsed().as_secs());
    });
}

async fn read_directory(source_folder: &str) -> Result<()> {
    //! Read all the csv files in the directory and parse the csv content into whois records. The
    //! records are then saved to a MongoDB database.
    let paths = std::fs::read_dir(source_folder)?;
    for path in paths {
        // TODO: Check if the file is a csv file
        // Run in parallel
        spawn(async move {
            let path = path.unwrap().path();
            info!("Processing file: {:?}", path);
            if path.is_file() {
                let file_path = path.to_str().unwrap();
                let records = WhoIsRecord::from_file(file_path).unwrap();
                info!("Found {} records in the file {}", records.len(), file_path);
                // Save records into the DB
                WhoIsRecord::save(records).await;
            }
        })
        .await?;
    }
    Ok(())
}
