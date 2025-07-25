use std::sync::Mutex;

use actix_files::Files;
use actix_web::{get, web, web::Data, App, HttpResponse, HttpServer, Responder};
use log::info;

//use my_web_app::MyTestStruct;
use my_web_app::StrainTableEntries;
use my_web_app::DatabaseIncludeRow;
use my_web_app::StrainColumns;

use rusqlite::types::ValueRef;

use std::io::Read;
use std::io::Cursor;

use rusqlite::OpenFlags;
extern crate rusqlite;

use rusqlite::{Connection, Result};

pub struct ServerData {
//    num: i32,
    conn: Connection,
    strain_columns: StrainColumns
}


/*
#[get("/hello")]
async fn hello() -> impl Responder {
    info!("Sending a String.");
    "Hallo Welt"
}
*/



/*
#[get("/json-data")]
async fn jsondata(counter: Data<Mutex<ServerData>>) -> impl Responder {
    let mut v = counter.lock().unwrap();
//    *v.num += 1;

    v.num += 1;

//    let data = MyTestStruct::from(*v.num);
    let data = MyTestStruct::from(v.num);
    info!("Data: {:?}", data);
//    info!("Sending: {:?}", counter.num);
    serde_json::to_string(&data)
}
 */




#[get("/straindata")]
async fn straindata(server_data: Data<Mutex<ServerData>>) -> impl Responder {
    let data = sql(&server_data).expect("could not read database");
    info!("Data: {:?}", data);
    serde_json::to_string(&data)
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let strain_columns = read_btyper_database_include(
            Cursor::new(include_bytes!("/Users/mahogny/Desktop/rust/2_actix-yew-template/minimal_testing/meta/btyperdb_include.tsv"))
        );


    let path = "/Users/mahogny/Desktop/rust/2_actix-yew-template/minimal_testing/meta/data.sqlite";
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).expect("Could not open database");



    let data = Data::new(Mutex::new(
        ServerData {
            //num: 0,
            conn: conn,
            strain_columns: strain_columns
        }
    ));


//    let table = sql();
//    println!("{:?}", table);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            //.service(hello)
            .service(straindata)
            .service(Files::new("/", "./dist/").index_file("index.html"))
            .default_service(
                web::route().to(|| HttpResponse::Found().header("Location", "/").finish()),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}













pub fn read_btyper_database_include (
    src: impl Read
) -> StrainColumns { 

    let mut outlist = Vec::new();

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(src);
    for result in reader.deserialize() {
        let record: DatabaseIncludeRow = result.unwrap();
        outlist.push(record);
    }

    StrainColumns {
        columns: outlist
    }
}














fn sql(server_data: &Data<Mutex<ServerData>>) -> Result<StrainTableEntries> {

    let server_data =server_data.lock().unwrap();

    let mut stmt = server_data.conn.prepare("SELECT * FROM straindata limit 5")?;

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

    Ok(StrainTableEntries {
        columns: cn,
        rows: ok_rows,
    })
}