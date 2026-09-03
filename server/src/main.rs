pub mod escaping;
pub mod stats;
pub mod straintable;
pub mod tree;
pub mod zip;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use actix_files::Files;
use actix_web::{web, web::Data, App, HttpResponse, HttpServer};
use rusqlite::Connection;
use rusqlite::OpenFlags;
use serde::Deserialize;
use serde::Serialize;

use my_web_app::{DatabaseMetadata, TreeData};
use straintable::*;
use tree::*;
use zip::*;

////////////////////////////////////////////////////////////
/// Backend state
pub struct ServerData {
    conn: Connection,
    db_metadata: DatabaseMetadata,
    /// Storage type of each column, from the sqlite schema. Range searches
    /// need it: the metadata file's "integer"/"float" says how a column should
    /// be searched, not how it is stored.
    column_storage: BTreeMap<String, String>,
    path_store: PathBuf,
    tree: TreeData,
}

////////////////////////////////////////////////////////////
/// Backend state
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
    let config_file: ConfigFile =
        serde_json::from_reader(config_reader).expect("Could not open config file");
    let path_store = Path::new(&config_file.store);

    //Read tree
    let tree_str = std::fs::read_to_string(path_store.join("tree.nwk"))?;
    let tree = TreeData { tree_str };

    // Open SQL database
    let path_sql = path_store.join(Path::new("meta/data.sqlite"));
    //let path = "/Users/mahogny/Desktop/rust/2_actix-yew-template/minimal_testing/meta/data.sqlite";
    let conn = Connection::open_with_flags(&path_sql, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("Could not open SQL database");

    let column_storage = read_column_storage(&conn).expect("Failed to read column storage types");

    let path_meta = path_store.join(Path::new("meta/btyperdb_include.json"));
    let f_meta = File::open(&path_meta)
        .unwrap_or_else(|e| panic!("Could not open {}: {}", path_meta.display(), e));
    let reader = BufReader::new(f_meta);
    let db_metadata = match read_database_metadata(reader, &conn, &column_storage) {
        Ok(m) => m,
        Err(e) => panic!("{}: {}", path_meta.display(), e),
    };

    let data = Data::new(Mutex::new(ServerData {
        conn: conn,
        db_metadata: db_metadata,
        column_storage: column_storage,
        tree: tree,
        path_store: path_store.into(),
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(actix_web::middleware::Logger::default()) //for debugging
            .service(straindata)
            .service(strainmeta)
            .service(strainfasta)
            .service(treedata)
            .service(Files::new("/", "./dist/").index_file("index.html"))
            .default_service(
                web::route().to(|| HttpResponse::NotFound()), //header("Location", "/").finish()
            )
    })
    .bind(config_file.bind)? /////////////// for dev, "127.0.0.1:8080"  ; 127.0.0.1:5199 for beagle deployment
    .run()
    .await
}
