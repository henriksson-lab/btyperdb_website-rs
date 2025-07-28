use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::io::BufReader;

use actix_files::Files;
use actix_web::{web, web::Data, App, HttpResponse, HttpServer};

use my_web_app::DatabaseMetadata;

extern crate rusqlite;
use rusqlite::OpenFlags;
use rusqlite::{Connection};

pub mod zip;
use serde::Deserialize;
use serde::Serialize;
use zip::*;

pub mod stats;
//use stats::*;

pub mod escaping;
//use escaping::*;

pub mod straintable;
use straintable::*;

////////////////////////////////////////////////////////////
/// Backend state
pub struct ServerData {
    conn: Connection,
    db_metadata: DatabaseMetadata,
    path_store: PathBuf,
}


#[derive(Debug, Deserialize, Serialize)]
struct ConfigFile {
    store: String,
    bind: String,
}






////////////////////////////////////////////////////////////
/// Backend entry point
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    // Read the config file
    let f_meta = File::open("config.json").expect("Could not open config.json");
    let config_reader = BufReader::new(f_meta);
    let config_file:ConfigFile = serde_json::from_reader(config_reader).expect("Could not open config file");

    

    // Open SQL database
    let path_store = Path::new(&config_file.store);
    let path_sql = path_store.join(Path::new("meta/data.sqlite"));
    //let path = "/Users/mahogny/Desktop/rust/2_actix-yew-template/minimal_testing/meta/data.sqlite";
    let conn = Connection::open_with_flags(&path_sql, OpenFlags::SQLITE_OPEN_READ_ONLY).expect("Could not open SQL database");

    let path_meta = path_store.join(Path::new("meta/btyperdb_include.tsv"));
    let f_meta = File::open(path_meta).expect("Could not open btyperdb_include");
    let reader = BufReader::new(f_meta);
    let db_metadata = read_database_metadata(
        reader,//Cursor::new(include_bytes!("/Users/mahogny/Desktop/rust/2_actix-yew-template/minimal_testing/meta/btyperdb_include.tsv")),
        &conn
    );

    /* 
    let ser:SearchSettings = serde_json::from_reader(
        Cursor::new(include_bytes!("/Users/mahogny/Desktop/rust/2_actix-yew-template/app/src/testsearch.json"))
    ).expect("asdasd");

    let q = build_straindb_search(&ser);
    println!("search {:?}",ser);
    println!("search {}",q);*/

    let data = Data::new(Mutex::new(
        ServerData {
            conn: conn,
            db_metadata: db_metadata,
            path_store: path_store.into()
        }
    ));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(actix_web::middleware::Logger::default())  //for debugging
            .service(straindata)
            .service(strainmeta)
            .service(strainfasta)
            .service(Files::new("/", "./dist/").index_file("index.html"))
            .default_service(
                web::route().to(|| HttpResponse::NotFound()),  //header("Location", "/").finish()
            )
    })
    .bind(config_file.bind)? /////////////// for dev, "127.0.0.1:8080"  ; 127.0.0.1:5199 for beagle deployment
    .run()
    .await
}




