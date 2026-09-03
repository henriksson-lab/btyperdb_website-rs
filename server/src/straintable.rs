use std::collections::BTreeMap;
use std::io::Read;
use std::sync::Mutex;

use actix_web::web::Json;
use actix_web::{get, post, web, web::Data, HttpResponse, Responder};

use my_web_app::ComparisonType;
use my_web_app::DatabaseColumn;
use my_web_app::DatabaseMetadata;
use my_web_app::SearchSettings;
use my_web_app::TableData;

use rusqlite::params_from_iter;
use rusqlite::types::{Value, ValueRef};
use rusqlite::{Connection, Result as SqlResult};

use crate::stats::*;
use crate::ServerData;

// Possible to test straindata search this way:
// curl --header "Content-Type: application/json" --request POST  -d '{"criteria":[]}' 127.0.0.1:8080/straindata -v

////////////////////////////////////////////////////////////
/// REST entry point
#[post("/straindata")]
async fn straindata(
    server_data: Data<Mutex<ServerData>>,
    req_body: web::Json<SearchSettings>,
) -> HttpResponse {
    //println!("{:?}",req_body);
    let Json(search_settings) = req_body;
    match query_straintable(&server_data, search_settings) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(QueryError::BadRequest(msg)) => HttpResponse::BadRequest().body(msg),
        Err(QueryError::Database(err)) => {
            eprintln!("Database query failed: {err:?}");
            HttpResponse::InternalServerError().body("could not read database")
        }
    }
}

////////////////////////////////////////////////////////////
/// REST entry point
#[get("/strainmeta")]
async fn strainmeta(server_data: Data<Mutex<ServerData>>) -> impl Responder {
    let server_data = server_data.lock().unwrap();
    //info!("metadata: {:?}", &server_data.db_metadata);
    serde_json::to_string(&server_data.db_metadata)
}

////////////////////////////////////////////////////////////
#[derive(Debug)]
enum QueryError {
    BadRequest(String),
    Database(rusqlite::Error),
}

impl From<rusqlite::Error> for QueryError {
    fn from(err: rusqlite::Error) -> Self {
        QueryError::Database(err)
    }
}

struct QueryPlan {
    sql: String,
    params: Vec<Value>,
}

////////////////////////////////////////////////////////////
/// Build a validated, parameterized SQL query for strain search.
fn build_straindb_search(
    search: &SearchSettings,
    metadata: &DatabaseMetadata,
) -> Result<QueryPlan, QueryError> {
    let mut query = "SELECT * FROM straindata ".to_string();
    let mut params = Vec::new();

    if search.criteria.len() > 0 {
        query.push_str(" WHERE ");

        let mut list_formatted_crit: Vec<String> = Vec::new();
        for crit in search.criteria.iter() {
            let col = metadata.columns.get(&crit.field).ok_or_else(|| {
                QueryError::BadRequest(format!("unknown search field: {}", crit.field))
            })?;
            let colname = sql_quote_identifier(&col.column_id);

            match &crit.comparison {
                ComparisonType::FromTo(from, to) => {
                    let from = parse_search_number(from, &crit.field)?;
                    let to = parse_search_number(to, &crit.field)?;
                    list_formatted_crit.push(format!("{} >= ?", colname));
                    params.push(Value::Real(from));
                    list_formatted_crit.push(format!("{} <= ?", colname));
                    params.push(Value::Real(to));
                }
                ComparisonType::Like(v) => {
                    list_formatted_crit.push(format!("{} LIKE ?", colname));
                    params.push(Value::Text(v.clone()));
                }
            };
        }
        //println!("{:?}",query);
        query.push_str(list_formatted_crit.join(" AND ").as_str());
    }
    query.push_str(" limit 100000");

    println!("search {}", query);
    Ok(QueryPlan { sql: query, params })
}

fn sql_quote_identifier(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

fn parse_search_number(s: &str, field: &str) -> Result<f64, QueryError> {
    let value = s
        .parse::<f64>()
        .map_err(|_| QueryError::BadRequest(format!("invalid numeric value for {field}: {s}")))?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(QueryError::BadRequest(format!(
            "invalid numeric value for {field}: {s}"
        )))
    }
}

////////////////////////////////////////////////////////////
/// Get metadata about the database
pub fn read_database_metadata(src: impl Read, conn: &Connection) -> SqlResult<DatabaseMetadata> {
    let mut list_dropdown = BTreeMap::new();
    let mut list_hist = Vec::new();

    /////////// Gather statistics to show
    list_hist.push(make_stats(
        &conn,
        &"BTyper3 Species".to_string(),
        &"matchcol_BTyper3_species".to_string(),
    )?);

    list_hist.push(make_stats(
        &conn,
        &"GTDB Species".to_string(),
        &"GTDB_Species".to_string(),
    )?);

    list_hist.push(make_stats(
        &conn,
        &"Isolation source (Source 1)".to_string(),
        &"Source_1".to_string(),
    )?);

    list_hist.push(make_stats(
        &conn,
        &"Human Illness".to_string(),
        &"Human_Illness".to_string(),
    )?);

    list_hist.push(make_stats(
        &conn,
        &"BTyper3 adjusted panC group".to_string(),
        &"BTyper3_Adjusted_panC_Group(predicted_species)".to_string(),
    )?);

    let hist_country = query_histogram(&conn, &"Country(Code)".to_string())?;

    let num_strain = query_get_strain_count(&conn).expect("Could not get SQL strain count");

    /////////// Other metadata from CSV-file
    let mut outlist = BTreeMap::new();
    let mut reader = csv::ReaderBuilder::new().delimiter(b'\t').from_reader(src);
    for result in reader.deserialize() {
        let record: DatabaseColumn = result.unwrap();

        /////////// Drop-down values for relevant fields  --- detect from metadata file?
        if record.dropdown {
            list_dropdown.insert(
                record.column_id.clone(),
                query_dropdown(conn, &record.column_id).expect("Failed to create dropdown"),
            );
        }

        outlist.insert(record.column_id.clone(), record);
    }

    //    println!("{:?}",list_dropdown);

    Ok(DatabaseMetadata {
        columns: outlist,
        num_strain: num_strain,
        column_dropdown: list_dropdown,
        list_hist: list_hist,
        hist_country: hist_country,
    })
}

////////////////////////////////////////////////////////////
/// Get entries from the strain table given search criteria
fn query_straintable(
    server_data: &Data<Mutex<ServerData>>,
    search: SearchSettings,
) -> Result<TableData, QueryError> {
    //println!("Query database using: {}",q);

    let server_data = server_data.lock().unwrap();
    let query_plan = build_straindb_search(&search, &server_data.db_metadata)?;

    let mut stmt = server_data.conn.prepare(query_plan.sql.as_str())?;

    let cn = stmt.column_names().iter().map(|x| x.to_string()).collect();
    let numcol = stmt.column_count();

    let rows = stmt.query_map(params_from_iter(query_plan.params.iter()), |row| {
        let mut out: Vec<String> = Vec::new();
        for i in 0..numcol {
            let value = match row.get_ref_unwrap(i) {
                ValueRef::Null => "".to_string(),
                ValueRef::Integer(i) => format!("{}", i).to_string(),
                ValueRef::Real(f) => format!("{}", f).to_string(),
                ValueRef::Text(t) => String::from_utf8_lossy(t).to_string(),
                ValueRef::Blob(_b) => "(blob)".to_string(),
            }
            .to_string();

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
            }
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
