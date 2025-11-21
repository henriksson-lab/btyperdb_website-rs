use std::collections::BTreeMap;
use serde::{Deserialize, Serialize, Serializer, de};

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
pub struct OneStats {
    pub name: String,
    pub hist: DatabaseHistogram,
}


////////////////////////////////////////////////////////////
/// Metadata about strain columns
#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseMetadata {
    pub num_strain: i32,
    pub columns: BTreeMap<String, DatabaseColumn>,
    pub column_dropdown: BTreeMap<String, Vec<String>>,

    pub list_hist: Vec<OneStats>,
    pub hist_country: DatabaseHistogram,   
}
impl DatabaseMetadata {

    ////////////////////////////////////////////////////////////
    /// Construct empty database
    pub fn new() -> DatabaseMetadata {
        DatabaseMetadata {
            num_strain: -1,
            columns: BTreeMap::new(),
            column_dropdown: BTreeMap::new(),
            list_hist: Vec::new(),
            hist_country: Vec::new(),
        }
    }


    ////////////////////////////////////////////////////////////
    /// Set up default search criteria
    pub fn make_default_search(&self) -> SearchSettings {
        
        let mut list_default = Vec::new();
        list_default.push("CheckM_Completeness".to_string());
        list_default.push("CheckM_Contamination".to_string());
        list_default.push("Quast_N50".to_string());
        list_default.push("Kraken_Phylum(Bacillota)".to_string());

        let mut list_fields=Vec::new();
        for v in &list_default {
            let col = self.columns.get(v).expect("could not find column");
            let c = SearchCriteria::default_search(col);
            list_fields.push(c);
        }

        SearchSettings {
            criteria: list_fields
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

    #[serde(deserialize_with = "deserialize_01bool", serialize_with = "serialize_01bool")]
    pub dropdown: bool, 
    #[serde(deserialize_with = "deserialize_01bool", serialize_with = "serialize_01bool")]
    pub display: bool,
    #[serde(deserialize_with = "deserialize_01bool", serialize_with = "serialize_01bool")]
    pub search: bool,
    #[serde(deserialize_with = "deserialize_01bool", serialize_with = "serialize_01bool")]
    pub print: bool,
    
    pub notes: String,
}


////////////////////////////////////////////////////////////
/// 1/0 => bool
fn deserialize_01bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;

    match s {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => Err(de::Error::unknown_variant(s, &["1", "0"])),
    }
}


////////////////////////////////////////////////////////////
/// bool => 1/0
fn serialize_01bool<S>(x: &bool, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if *x {
        s.serialize_str("1")
    } else {
        s.serialize_str("0")
    }
}




////////////////////////////////////////////////////////////
/// 
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct StrainRequest {
    pub list: Vec<String>
}




////////////////////////////////////////////////////////////
/// 
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct SearchSettings {
    pub criteria: Vec<SearchCriteria>
}
impl SearchSettings {
    pub fn new() -> SearchSettings {

        /*
        let mut c= SearchCriteria::new();
        c.field = "BTyperDB_ID".to_string();
        c.comparison = ComparisonType::Like("BTDB_2022-0000001.1".to_string());// "".to_string();
 */
        
        let mut list_default = Vec::new();
        list_default.push("CheckM_Completeness".to_string());
        list_default.push("CheckM_Contamination".to_string());
        list_default.push("Quast_N50".to_string());
        list_default.push("Kraken_Phylum(Bacillota)".to_string());

        let mut list_fields=Vec::new();
        for v in list_default {
            let mut c= SearchCriteria::new();
            c.field = "BTyperDB_ID".to_string();
            c.comparison = ComparisonType::Like(v);
            list_fields.push(c);
        }


        SearchSettings {
            criteria: list_fields
        }
    }
}


////////////////////////////////////////////////////////////
/// 
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct SearchCriteria {
    pub field: String,
    pub comparison: ComparisonType,
}
impl SearchCriteria {


    ////////////////////////////////////////////////////////////
    /// 
    pub fn new() -> SearchCriteria {
        SearchCriteria {
            field: "".to_string(),
            comparison: ComparisonType::Like("".to_string())
        }
    }


    ////////////////////////////////////////////////////////////
    /// 
    pub fn default_search(col: &DatabaseColumn) -> SearchCriteria {
        let comp = ComparisonType::default_comparison(col);
        SearchCriteria {
            field: col.column_id.clone(),
            comparison: comp
        }
    }
}


////////////////////////////////////////////////////////////
/// 
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum ComparisonType {
    Like(String),
    FromTo(String,String),
}
impl ComparisonType {


    ////////////////////////////////////////////////////////////
    /// Generate a comparison with default fields
    pub fn default_comparison(db: &DatabaseColumn) -> ComparisonType {
        if db.column_type == "text" {
            ComparisonType::Like(db.default_v1.clone()) 
        } else if db.column_type == "float" || db.column_type == "integer" {
            ComparisonType::FromTo(
                db.default_v1.clone(),
                db.default_v2.clone(),
            ) 
        } else {
            println!("!!!! unexpected type of data {}", db.column_type);
            ComparisonType::Like("".to_string()) //TODO
        }        
    }


}