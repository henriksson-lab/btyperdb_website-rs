use std::sync::Mutex;

use std::collections::BTreeMap;


use actix_files::Files;
use actix_web::web::Json;
use actix_web::{get, post, web, web::Data, App, HttpResponse, HttpServer, Responder};
use log::info;

use my_web_app::ComparisonType;
use my_web_app::SearchSettings;
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


// Possible to test search this way:
// curl --header "Content-Type: application/json" --request POST  -d '{"criteria":[]}' 127.0.0.1:8080/straindata -v


////////////////////////////////////////////////////////////
/// REST entry point
#[post("/straindata")]
//#[get("/straindata")]
async fn straindata(server_data: Data<Mutex<ServerData>>, req_body: web::Json<SearchSettings>) -> impl Responder {

    println!("{:?}",req_body); 

    let Json(search_settings) = req_body;

//    let search: SearchSettings = serde_json::from_str(req_body.as_str()).expect("Failed to parse search settings");
//    println!("{:?}",search);

    let data = query_straintable(&server_data, search_settings).expect("could not read database");
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



pub fn sql_stringarg_to_num(s: &String) -> String {
    if s.parse::<f64>().is_ok() {
        s.clone()
    } else {
        panic!("bad value")
    }
}

pub fn sql_stringarg_escape(s: &String) -> String {
    let mut out =String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c.is_whitespace() || c=='-' || c=='_' || c=='.' || c==',' {
            out.push(c);
        } else {
            println!("!!!!!!!!!!! unhandled char {}",c);
        }
    }
    out
}


pub fn sql_check_name(s: &String) -> String {
    if s.len()==0 {
        panic!("invalid name as it is empty");
    }

    let valid_char="abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVQXYZ0123456789_".as_bytes();
    for b in s.as_bytes() {
        if !valid_char.contains(b) {
            panic!("invalid character in name {}", b);
        }
    }
    s.clone()
}




pub fn build_straindb_search(search: &SearchSettings) -> String {

    let mut query = "SELECT * ".to_string();

    query.push_str(" FROM straindata ");

    //let mut list_params:Vec<String> = Vec::new();
    //https://docs.rs/rusqlite/latest/rusqlite/struct.Statement.html
    //?1 ?2 etc

    if search.criteria.len()>0 {
        query.push_str(" WHERE ");

        let mut list_formatted_crit:Vec<String> = Vec::new();
        for crit in search.criteria.iter() {
            match &crit.comparison {
                ComparisonType::FromTo(from,to) => {
                    list_formatted_crit.push(format!("{} > {}",sql_check_name(&crit.field), sql_stringarg_to_num(&from))); /////////// can we produce a list of {} prep statement?
                    list_formatted_crit.push(format!("{} < {}",sql_check_name(&crit.field), sql_stringarg_to_num(&to)));
                },
                ComparisonType::Like(v) => {
                   list_formatted_crit.push(format!("{} LIKE \"{}\"",sql_check_name(&crit.field), sql_stringarg_escape(&v)));
                }
            };
        }
        //println!("{:?}",query);
        query.push_str(list_formatted_crit.join(" AND ").as_str());
    }
    query.push_str(" limit 6000");
    query
}

////////////////////////////////////////////////////////////
/// Backend entry point
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //std::env::set_var("RUST_LOG", "info");
    //std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let path = "/Users/mahogny/Desktop/rust/2_actix-yew-template/minimal_testing/meta/data.sqlite";
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).expect("Could not open database");

    let db_metadata = read_database_metadata(
        Cursor::new(include_bytes!("/Users/mahogny/Desktop/rust/2_actix-yew-template/minimal_testing/meta/btyperdb_include.tsv")),
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
        }
    ));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(actix_web::middleware::Logger::default())  //for debugging
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

    let mut outlist = BTreeMap::new();//::new();

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(src);
    for result in reader.deserialize() {
        let record: DatabaseColumn = result.unwrap();
        outlist.insert(record.column_id.clone(), record);
    }

    DatabaseMetadata {
        columns: outlist,
        num_strain: num_strain
    }
}




////////////////////////////////////////////////////////////
/// Get entries from the strain table given search criteria
fn query_straintable(
    server_data: &Data<Mutex<ServerData>>,
    search: SearchSettings
) -> Result<TableData> {

    let q = build_straindb_search(&search);
    println!("Query database using: {}",q);

    let server_data =server_data.lock().unwrap();

    let mut stmt = server_data.conn.prepare(q.as_str())?;
    //let mut stmt = server_data.conn.prepare("SELECT * FROM straindata limit 6000")?;

    
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