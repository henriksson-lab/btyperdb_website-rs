use std::collections::{BTreeMap};
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




////////////////////////////////////////////////////////////
/// 
#[derive(Debug, Deserialize, Serialize)]
pub struct TreeData {
    pub tree_str: String,
}


/*
#[derive(Debug, Deserialize, Serialize)]
pub struct Test {
    pub data: SimpleRootedTree<String, f32, f32>,

} */


/* 

////////////////////////////////////////////////////////////
/// 
#[derive(Debug, Deserialize, Serialize)]
pub struct SerializableTree {

    /// Root NodeID
    pub root: NodeID,
    /// Nodes of the tree
    pub nodes: Vec<Option<Node<String, f32, f32>>>,
    /// Index of nodes by taxa
    pub taxa_node_id_map: HashMap<String, NodeID>,

    /*    
    /// Field to hold precomputed euler tour for constant-time LCA queries
    pub precomputed_euler: Option<Vec<NodeID>>,
    /// Field to hold precomputed first-appearance for constant-time LCA queries
    pub precomputed_fai: Option<Vec<Option<usize>>>,
    /// Field to hold precomputed depth-array for constant-time LCA queries
    pub precomputed_da: Option<Vec<usize>>,

    // Field to hold precomputed range-minimum-query for constant-time LCA queries
    // pub precomputed_rmq: Option<BinaryRmq>,
    */
}
impl SerializableTree {


    ////////////////////////////////////////////////////////////
    /// 
    pub fn to_serialize(tree: SimpleRootedTree<String, f32, f32>) -> SerializableTree {

        SerializableTree {
            root: tree.root, 
            nodes: tree.nodes,
            taxa_node_id_map: tree.taxa_node_id_map,
        }
    }


    ////////////////////////////////////////////////////////////
    /// 
    pub fn from_serialize(tree: SimpleRootedTree<String, f32, f32>) -> SerializableTree {


        let ser_tree: SimpleRootedTree<String, f32, f32> = SimpleRootedTree {
            root: tree.root, 
            nodes: tree.nodes,
            taxa_node_id_map: tree.taxa_node_id_map,
            
            precomputed_euler: None,
            precomputed_fai: None,
            precomputed_da: None,
            precomputed_rmq: None,                
        };
    }



}



pub fn serialize_tree(tree: SimpleRootedTree<String, f32, f32>) {


    let ser_tree: SimpleRootedTree<String, f32, f32> = SimpleRootedTree {
        root: tree.root, 
        nodes: tree.nodes,
        taxa_node_id_map: tree.taxa_node_id_map,
        
        precomputed_euler: None,
        precomputed_fai: None,
        precomputed_da: None,
        precomputed_rmq: None,                
    };
}


pub fn unserialize_tree() {

//    let tree: SimpleRootedTree<String, f32, f32> = SimpleRootedTree::new(root_id);


}



#[derive(Clone, Deserialize, Serialize)]
pub struct SerializeNode 
{
    /// A unique identifier for a node
    id: NodeID,
    /// A link to the node parent (set to None for root)
    parent: Option<NodeID>,
    /// Children of node
    children: Vec<NodeID>,
    /// Taxa annotation of node
    taxa: Option<String>,
    /// Weight of edge ending in node
    weight: Option<f32>,
    /// Real number annotation of node (used by some algorithms)
    zeta: Option<f32>,
}
impl SerializeNode {


    pub fn from_serialize(n: Node<String, f32, f32>) -> SerializeNode {
        SerializeNode {
            id: n.get_id(),
            parent: n.get_parent(),
            children: n.get_children().collect(),
            taxa: n.get_taxa().cloned(),
            weight: n.get_weight(),
            zeta: n.get_zeta(),
        }
    }


    pub fn to_serialize(n: SerializeNode) -> Node<String, f32, f32> {
        Node {
            id: n.id,
            parent: n.parent,
            children: n.children,
            taxa: n.taxa,
            weight: n.weight,
            zeta: n.zeta,
        }
    }

    

}

*/