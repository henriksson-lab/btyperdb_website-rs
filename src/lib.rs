use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MyTestStruct {
    v1: usize,
    v2: Option<String>,
}

impl MyTestStruct {
    pub fn new() -> Self {
        Self {
            v1: rand::thread_rng().gen_range(0..1000),
            v2: None,
        }
    }

    pub fn from(v1: i32) -> Self {
        Self {
            v1: v1 as usize,
            v2: None,
        }
    }
}








#[derive(Debug, Deserialize, Serialize)]
pub struct TableData {
    column_names: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl TableData {
    pub fn new() -> Self {
        Self {
            column_names: vec![],
            rows: vec![]
        }
    }

}


//////////////////////////////////////
/// New table to offer

#[derive(Debug, Deserialize, Serialize)]
pub struct StrainTableEntries {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}



#[derive(Debug, Deserialize, Serialize)]
pub struct StrainColumns {
    pub columns: Vec<DatabaseIncludeRow>
}



#[derive(Debug, Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct DatabaseIncludeRow {
    pub column_id: String,
    pub column_type: String,	
    pub default_v1: String,	
    pub default_v2: String,	
    pub default_show_column: String,
    pub display: String,
    pub search: String,
    pub print: String,
    pub notes: String,
}
