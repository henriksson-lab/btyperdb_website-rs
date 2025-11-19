use std::sync::Mutex;
use std::collections::BTreeMap;
use std::io::Read;

use actix_web::web::Json;
use actix_web::{get, post, web, web::Data, Responder};

use my_web_app::{ComparisonType};
use my_web_app::SearchSettings;
use my_web_app::TableData;
use my_web_app::DatabaseColumn;
use my_web_app::DatabaseMetadata;

use rusqlite::types::ValueRef;
use rusqlite::{Connection, Result};

use crate::ServerData;
use crate::escaping::*;
use crate::stats::*;


// Possible to test straindata search this way:
// curl --header "Content-Type: application/json" --request POST  -d '{"criteria":[]}' 127.0.0.1:8080/straindata -v


////////////////////////////////////////////////////////////
/// REST entry point
#[post("/straindata")]
async fn straindata(server_data: Data<Mutex<ServerData>>, req_body: web::Json<SearchSettings>) -> impl Responder {
    //println!("{:?}",req_body); 
    let Json(search_settings) = req_body;
    let data = query_straintable(&server_data, search_settings).expect("could not read database");
    //info!("Data: {:?}", data);
    serde_json::to_string(&data)
}



////////////////////////////////////////////////////////////
/// REST entry point
#[get("/strainmeta")]
async fn strainmeta(server_data: Data<Mutex<ServerData>>) -> impl Responder {
    let server_data =server_data.lock().unwrap();
    //info!("metadata: {:?}", &server_data.db_metadata);
    serde_json::to_string(&server_data.db_metadata)
}



////////////////////////////////////////////////////////////
/// x
pub fn build_straindb_search(search: &SearchSettings) -> String {
    let mut query = "SELECT * FROM straindata ".to_string();

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
/// Get metadata about the database
pub fn read_database_metadata (
    src: impl Read,
    conn: &Connection
) -> DatabaseMetadata { 

    let mut list_dropdown = BTreeMap::new();

    /////////// Gather statistics to show
    let hist_source1 = query_histogram(&conn, &"Source_1".to_string()).expect("Failed to make histogram");
    let hist_pancgroup = query_histogram(&conn, &"BTyper3_Adjusted_panC_Group(predicted_species)".to_string()).expect("Failed to make histogram");
    let hist_gtdb_species = query_histogram(&conn, &"GTDB_Species".to_string()).expect("Failed to make histogram");
    let hist_humanillness = query_histogram(&conn, &"Human_Illness".to_string()).expect("Failed to make histogram");
    let hist_country = query_histogram(&conn, &"Country(Code)".to_string()).expect("Failed to make histogram");    // was: Country_Code

    let num_strain = query_get_strain_count(&conn).expect("Could not get SQL strain count");



    /////////// Other metadata from CSV-file
    let mut outlist = BTreeMap::new();
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(src);
    for result in reader.deserialize() {
        let record: DatabaseColumn = result.unwrap();

        /////////// Drop-down values for relevant fields  --- detect from metadata file?
        if record.dropdown=="1" {
            //let col = record.column_id;//.to_string();
            list_dropdown.insert(record.column_id.clone(), query_dropdown(conn, &record.column_id).expect("Failed to create dropdown"));        
        }

        outlist.insert(record.column_id.clone(), record);
    }

    /////////// Drop-down values for relevant fields  --- detect from metadata file?
//    for col in &vec!["Country","Country(Code)","Continent","Region_Code","Source_1","Source_2","Source_3","Human_Illness","Human_Outbreak","GTDB_Species","BTyper3_Adjusted_panC_Group(predicted_species)"] {
  //      let col = col.to_string();
    //    list_dropdown.insert(col.clone(), query_dropdown(conn, &col).expect("Failed to create dropdown"));        
   // }


    
//    println!("{:?}",list_dropdown);

    DatabaseMetadata {
        columns: outlist,
        num_strain: num_strain,
        column_dropdown: list_dropdown,

        hist_humanillness: hist_humanillness,
        hist_source1: hist_source1,
        hist_pancgroup: hist_pancgroup,
        hist_gtdb_species: hist_gtdb_species,
        hist_country: hist_country
    }
}




////////////////////////////////////////////////////////////
/// Get entries from the strain table given search criteria
fn query_straintable(
    server_data: &Data<Mutex<ServerData>>,
    search: SearchSettings
) -> Result<TableData> {

    let q = build_straindb_search(&search);
    //println!("Query database using: {}",q);

    let server_data =server_data.lock().unwrap();

    let mut stmt = server_data.conn.prepare(q.as_str())?;
    
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

