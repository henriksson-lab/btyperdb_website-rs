use serde::{Deserialize, Serialize};

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
    pub columns: Vec<DatabaseColumn>
}
impl DatabaseMetadata {
    pub fn new() -> DatabaseMetadata {
        DatabaseMetadata {
            num_strain: -1,
            columns: vec![]
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
    pub display: String,
    pub search: String,
    pub print: String,
    pub notes: String,
}













pub struct SearchSettings {
    pub criteria: Vec<SearchCriteria>
}
impl SearchSettings {
    pub fn new() -> SearchSettings {

        let mut c= SearchCriteria::new();
        c.field = "BTyperDB_ID".to_string();
        c.is = "BTDB_2022-0000001.1".to_string();

        SearchSettings {
            criteria: vec![c]
        }
    }
}


pub struct SearchCriteria {
    pub field: String,
    pub from: String,
    pub to: String,
    pub is: String,
}
impl SearchCriteria {
    pub fn new() -> SearchCriteria {
        SearchCriteria {
            field: "".to_string(),
            from: "".to_string(),
            to: "".to_string(),
            is: "".to_string()
        }
    }
}


