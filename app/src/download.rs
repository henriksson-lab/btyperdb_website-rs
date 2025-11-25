use std::collections::HashSet;

use wasm_bindgen::JsCast;
use web_sys::js_sys::{Array, Uint8Array};
use web_sys::{wasm_bindgen::JsValue, Blob, BlobPropertyBag};
use web_sys::{HtmlElement};
use web_sys::window;

use crate::appstate::AsyncData;
use crate::core_model::Model;


impl Model {

    ////////////////////////////////////////////////////////////
    /// generate CSV from the table data we got already
    pub fn make_metadata_csv(&self, list_strains: &Vec<String>) -> String {
        let mut csv = String::new();

        if let Some(db_metadata) = &self.db_metadata {

            let mut set_strains = HashSet::new();
            for e in list_strains {
                set_strains.insert(e);
            }

            if let AsyncData::Loaded(tabledata) = &self.tabledata {

                //Decide which columns to include
                let mut pick_col_id = Vec::new();
                for (i,colname) in tabledata.columns.iter().enumerate() {
                    let colmeta = db_metadata.columns.get(colname).expect("could not get column");
                    if colmeta.print {
//                    if colmeta.print=="1" {
                        pick_col_id.push(i);
                    }
                }

                for col_i in &pick_col_id {
                    csv.push_str(tabledata.columns.get(*col_i).expect("expected column"));
                    csv.push_str("\t");
                }
                csv.push_str("\n");


                for r in &tabledata.rows {
                    let id = r.get(0).expect("empty row");
                    if set_strains.contains(id) {

                        for col_i in &pick_col_id {
                            csv.push_str(r.get(*col_i).expect("expected column"));
                            csv.push_str("\t");
                        }
                        csv.push_str("\n");
                    }
                }
            } else {
                log::debug!("tabledata is missing");
            }

        }


        csv
    }


    ////////////////////////////////////////////////////////////
    /// Download table data
    pub fn download_metadata(&self, list_strains: &Vec<String>){

        let window = window().expect("no window");
        let document = window.document().expect("should have a document on window");
        //let body = document.body().expect("document should have a body");

        let csv_data = self.make_metadata_csv(list_strains);

        // Creating a Blob for having a csv file format and passing the data with type
        // https://docs.rs/web-sys/latest/web_sys/struct.Blob.html
        // const blob = new Blob([data], { type: 'text/csv' }); 
        let blob_properties = BlobPropertyBag::new();
        blob_properties.set_type("text/csv");  // application/zip     application/json

        let blob_parts = Array::new();
        blob_parts.push(&JsValue::from_str(csv_data.as_str()));
        let blob = Blob::new_with_buffer_source_sequence_and_options(&blob_parts, &blob_properties).unwrap();

        // Creating an object for downloading url
        // const url = window.URL.createObjectURL(blob)
        let url = web_sys::Url::create_object_url_with_blob(&blob).expect("Could not create url");

        // Creating an anchor(a) tag of HTML
        // const a = document.createElement('a')
        let a:HtmlElement = document.create_element("a").expect("could not create a").dyn_into().unwrap();

        // Passing the blob downloading url 
        // a.setAttribute('href', url)
        a.set_attribute("href", &url).expect("Could not set attribute");

        // Setting the anchor tag attribute for downloading
        // and passing the download file name
        // a.setAttribute('download', 'download.csv');    
    //    a.set_attribute("download", "fastq.zip").expect("Could not set attribute");
        a.set_attribute("download", "metadata.txt").expect("Could not set attribute");

        // Performing a download with click
        // a.click()
        a.click();
    }








    ////////////////////////////////////////////////////////////
    /// Download table data
    /// 
    /// not ideal to download into rust memory first, then send to JS space, then save. fix in future
    pub fn download_fasta(&self, data: &Vec<u8>){

        //https://docs.rs/js-sys/latest/js_sys/struct.Uint8Array.html
        let arr = Uint8Array::new_with_length(data.len() as u32);
        arr.copy_from(&data.as_slice());

        let window = window().expect("no window");
        let document = window.document().expect("should have a document on window");
        //let body = document.body().expect("document should have a body");

        // Creating a Blob for having a csv file format and passing the data with type
        // https://docs.rs/web-sys/latest/web_sys/struct.Blob.html
        // const blob = new Blob([data], { type: 'text/csv' }); 
        let blob_properties = BlobPropertyBag::new();
        blob_properties.set_type("application/zip");

        let blob_parts = Array::new();
        blob_parts.push(&arr);  
        let blob = Blob::new_with_buffer_source_sequence_and_options(&blob_parts, &blob_properties).unwrap();

        // Creating an object for downloading url
        // const url = window.URL.createObjectURL(blob)
        let url = web_sys::Url::create_object_url_with_blob(&blob).expect("Could not create url");

        // Creating an anchor(a) tag of HTML
        // const a = document.createElement('a')
        let a:HtmlElement = document.create_element("a").expect("could not create a").dyn_into().unwrap();

        // Passing the blob downloading url 
        // a.setAttribute('href', url)
        a.set_attribute("href", &url).expect("Could not set attribute");

        // Setting the anchor tag attribute for downloading
        // and passing the download file name
        // a.setAttribute('download', 'download.csv');    
    //    a.set_attribute("download", "fastq.zip").expect("Could not set attribute");
        a.set_attribute("download", "btyper_fasta.zip").expect("Could not set attribute");

        // Performing a download with click
        // a.click()
        a.click();
    }





    
}





  /*
   * if we want to hack javascript, this is more scalable
   * 
   * from view-source:https://jimmywarting.github.io/StreamSaver.js/examples/fetch.html
   */
  /*
  downloadFasta(listFasta){
    const fileStream = this.streamSaver.createWriteStream('fasta.zip');
    var query=listFasta;
    fetch(
        'rest/getfasta',{
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify(query)
    }).then(res => {
          const readableStream = res.body
          if (window.WritableStream && readableStream.pipeTo) {
            return readableStream.pipeTo(fileStream)
              .then(() => console.log('done writing'))
          }
          window.writer = fileStream.getWriter()
          const reader = res.body.getReader()
          const pump = () => reader.read()
            .then(res => res.done
              ? window.writer.close()
              : window.writer.write(res.value).then(pump))
          pump()
    })
  }

   */
 
