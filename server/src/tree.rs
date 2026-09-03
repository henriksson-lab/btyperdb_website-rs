use actix_web::{get, web::Data, Responder};
use std::sync::Mutex;

use crate::ServerData;

////////////////////////////////////////////////////////////
/// REST entry point
#[get("/treedata")]
async fn treedata(server_data: Data<Mutex<ServerData>>) -> impl Responder {
    let server_data = server_data.lock().unwrap();
    let data = &server_data.tree;
    serde_json::to_string(&data)
}
