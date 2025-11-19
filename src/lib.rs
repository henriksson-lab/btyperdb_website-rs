use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

type DatabaseHistogram = Vec<(String,i32)>;


////////////////////////////////////////////////////////////
/// Strain table data
#[derive(Debug, Deserialize, Serialize)]
pub struct TableData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}



////////////////////////////////////////////////////////////
/// Metadata about strain columns
#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseMetadata {
    pub num_strain: i32,
    pub columns: BTreeMap<String, DatabaseColumn>,
    pub column_dropdown: BTreeMap<String, Vec<String>>,

    pub hist_humanillness: DatabaseHistogram,
    pub hist_source1: DatabaseHistogram,
    pub hist_pancgroup: DatabaseHistogram,
    pub hist_gtdb_species: DatabaseHistogram,
    pub hist_country: DatabaseHistogram,

    
}
impl DatabaseMetadata {
    pub fn new() -> DatabaseMetadata {
        DatabaseMetadata {
            num_strain: -1,
            columns: BTreeMap::new(),
            column_dropdown: BTreeMap::new(),

            hist_humanillness: Vec::new(),
            hist_source1: Vec::new(),
            hist_pancgroup: Vec::new(),
            hist_gtdb_species: Vec::new(),
            hist_country: Vec::new(),

        }
    }
}


////////////////////////////////////////////////////////////
/// Metadata about one column in the database
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct DatabaseColumn {
    pub column_id: String,
    pub column_type: String,	
    pub default_v1: String,	
    pub default_v2: String,	
    pub default_show_column: String,
    pub dropdown: String, /// should we do bools instead?
    pub display: String,
    pub search: String,
    pub print: String,
    pub notes: String,
}






#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct StrainRequest {
    pub list: Vec<String>
}




#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct SearchSettings {
    pub criteria: Vec<SearchCriteria>
}
impl SearchSettings {
    pub fn new() -> SearchSettings {

        let mut c= SearchCriteria::new();
        c.field = "BTyperDB_ID".to_string();
        c.comparison = ComparisonType::Like("BTDB_2022-0000001.1".to_string());// "".to_string();

        SearchSettings {
            criteria: vec![c]
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct SearchCriteria {
    pub field: String,
    pub comparison: ComparisonType,
}
impl SearchCriteria {
    pub fn new() -> SearchCriteria {
        SearchCriteria {
            field: "".to_string(),
            comparison: ComparisonType::Like("".to_string())
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum ComparisonType {
    Like(String),
    FromTo(String,String),
}
impl ComparisonType {


    pub fn default_comparison(db: &DatabaseColumn) -> ComparisonType {
        if db.column_type == "text" {
            ComparisonType::Like(db.default_v1.clone()) 
        } else if db.column_type == "float" || db.column_type == "integer" {
            ComparisonType::FromTo(
                db.default_v1.clone(),
                db.default_v2.clone(),
            ) 
        } else {
            println!("!!!!!!!!!!!!!!!!!!!!!!!!!! unexpected type of data {}", db.column_type);
            ComparisonType::Like("".to_string()) //TODO
        }        
    }


}