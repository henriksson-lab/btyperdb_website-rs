use std::sync::Mutex;

use actix_web::http::header::ContentDisposition;
use actix_web::web::Json;
use actix_web::{HttpResponse};
use actix_web::{post, web, web::Data};
use archflow::compress::tokio::archive::ZipArchive;
use archflow::compress::FileOptions;
use archflow::compression::CompressionMethod;
use archflow::types::FileDateTime;
use tokio::fs::File;
use tokio::io::duplex;
use tokio_util::io::ReaderStream;

use my_web_app::StrainRequest;

use crate::ServerData;
use crate::escaping::*;


////////////////////////////////////////////////////////////
/// x
#[post("/strainfasta")]
pub async fn strainfasta(server_data: Data<Mutex<ServerData>>, req_body: web::Json<StrainRequest>) -> HttpResponse {
    println!("{:?}",req_body); 
    let Json(req) = req_body;

    println!("{:?}",req);

    let (w, r) = duplex(4096);
    let options = FileOptions::default()
        .last_modified_time(FileDateTime::Now)
        .compression_method(CompressionMethod::Deflate());

    //let list_files = vec!["BTDB_2022-0001042.1".to_string()];

    let path_store = {
    let server_data =server_data.lock().unwrap();
        server_data.path_store.clone()
    };
    let path_fna = path_store.join("fna");


    tokio::spawn(async move {
        let mut archive = ZipArchive::new_streamable(w);

        for f in req.list {
            let f = clean_btyper_id(&f);
            println!("sending {}",f);

            let fname_outer = format!("{}.fna", f);
            let file_path = path_fna.join(&fname_outer);

            //Future option: if each file already zipped, could directly concatenate their contents
            let mut file = File::open(file_path).await.unwrap();
            archive
                .append(&format!("fastq/{}.fasta", f), &options, &mut file)
                .await
                .unwrap();
        }
        
        println!("finalizing zip to send");
        archive.finalize().await.unwrap();
    });

    HttpResponse::Ok()
        .insert_header(("Content-Type", "application/zip"))
        .insert_header(ContentDisposition::attachment("btyper_fastq.zip"))
        .streaming(ReaderStream::new(r))
 }






// curl --header "Content-Type: application/json" --request POST  -d '{"list":["BTDB_2022-0001042.1"]}' 127.0.0.1:8080/strainfastq -v -o test.zip
