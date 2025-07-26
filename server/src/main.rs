use std::num::NonZero;
use std::sync::Mutex;

use actix_files::Files;
use actix_web::{get, web, web::Data, App, HttpResponse, HttpServer, Responder};
use log::info;

use my_web_app::TableData;
use my_web_app::DatabaseColumn;
use my_web_app::DatabaseMetadata;

use rusqlite::types::ValueRef;

use std::io::Read;
use std::io::Cursor;

use rusqlite::OpenFlags;
extern crate rusqlite;

use rusqlite::{Connection, Result};

////////////////////////////////////////////////////////////
/// Backend state
pub struct ServerData {
    conn: Connection,
    db_metadata: DatabaseMetadata,
}


////////////////////////////////////////////////////////////
/// REST entry point
#[get("/straindata")]
async fn straindata(server_data: Data<Mutex<ServerData>>) -> impl Responder {
    let data = query_straintable(&server_data).expect("could not read database");
    info!("Data: {:?}", data);
    serde_json::to_string(&data)
}



////////////////////////////////////////////////////////////
/// REST entry point
#[get("/strainmeta")]
async fn strainmeta(server_data: Data<Mutex<ServerData>>) -> impl Responder {
    let server_data =server_data.lock().unwrap();
    info!("metadata: {:?}", &server_data.db_metadata);
    serde_json::to_string(&server_data.db_metadata)
}



////////////////////////////////////////////////////////////
/// Backend entry point
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let path = "/Users/mahogny/Desktop/rust/2_actix-yew-template/minimal_testing/meta/data.sqlite";
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).expect("Could not open database");

    let db_metadata = read_database_metadata(
        Cursor::new(include_bytes!("/Users/mahogny/Desktop/rust/2_actix-yew-template/minimal_testing/meta/btyperdb_include.tsv")),
        &conn
    );

    let data = Data::new(Mutex::new(
        ServerData {
            conn: conn,
            db_metadata: db_metadata,
        }
    ));



    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            //.service(hello)
            .service(straindata)
            .service(strainmeta)
            .service(Files::new("/", "./dist/").index_file("index.html"))
            .default_service(
                web::route().to(|| HttpResponse::Found().header("Location", "/").finish()),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}






////////////////////////////////////////////////////////////
/// Get metadata about the database
pub fn read_database_metadata (
    src: impl Read,
    conn: &Connection
) -> DatabaseMetadata { 


    let num_strain = query_get_strain_count(&conn).expect("Could not get SQL strain count");

    /////////// Other metadata from CSV-file

    let mut outlist = Vec::new();

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(src);
    for result in reader.deserialize() {
        let record: DatabaseColumn = result.unwrap();
        outlist.push(record);
    }

    DatabaseMetadata {
        columns: outlist,
        num_strain: num_strain
    }
}




////////////////////////////////////////////////////////////
/// Get entries from the strain table given search criteria
fn query_straintable(
    server_data: &Data<Mutex<ServerData>>
) -> Result<TableData> {

    let server_data =server_data.lock().unwrap();

    let mut stmt = server_data.conn.prepare("SELECT * FROM straindata limit 50000")?;

    let cn = stmt.column_names().iter().map(|x| x.to_string()).collect();
    let numcol = stmt.column_count();

    let rows = stmt.query_map([], |row| {
        let mut out:Vec<String> = Vec::new();
        for i in 0..numcol {

            let value = match row.get_ref_unwrap(i) {
                ValueRef::Null => "".to_string(),
                ValueRef::Integer(i) => format!("{}",i).to_string(),
                ValueRef::Real(f) => format!("{}",f).to_string(),
                ValueRef::Text(t) => String::from_utf8_lossy(t).to_string(),
                ValueRef::Blob(_b) => "(blob)".to_string()
            }.to_string();

            out.push(value); //////// need to cast to the right type...

            //out.push(row.get(i)?); //////// need to cast to the right type...
        }
        Ok(out)
    })?;

    let mut ok_rows = Vec::new();
    for row in rows {
        match row {
            Ok(row) => {
                ok_rows.push(row);
                //println!("ID: {:?}", row)
            },
            Err(e) => {
                eprintln!("Error: {e:?}")
            }
        }
    }

    Ok(TableData {
        columns: cn,
        rows: ok_rows,
    })
}










////////////////////////////////////////////////////////////
/// 
pub fn query_get_strain_count(
    conn: &Connection
) -> Result<i32> {

    let mut stmt = conn.prepare("SELECT count(*) as cnt FROM straindata")?;

    let cnts = stmt.query_map([], |row| {
        let val = row.get(0)?;
        Ok(val)
    })?;

    let mut ret_cnt: i32 = -1;
    for cnt in cnts {
        if let Ok(cnt) = cnt {
            ret_cnt = cnt;
        }
    }

    Ok(ret_cnt)
}