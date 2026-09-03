use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod derive;

type DatabaseHistogram = Vec<(String, i32)>;

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

        let mut list_fields = Vec::new();
        for v in &list_default {
            let col = self.columns.get(v).expect("could not find column");
            let c = SearchCriteria::default_search(col);
            list_fields.push(c);
        }

        SearchSettings {
            criteria: list_fields,
        }
    }
}

////////////////////////////////////////////////////////////
/// How a column is searched. Anything else in the metadata file is a
/// load-time error rather than a silent fallback to "matches nothing".
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ColumnType {
    Text,
    Float,
    Integer,
}

impl ColumnType {
    pub fn is_numeric(&self) -> bool {
        matches!(self, ColumnType::Float | ColumnType::Integer)
    }
}

////////////////////////////////////////////////////////////
/// Metadata about one column in the database, as read from
/// meta/btyperdb_include.json.
///
/// Every flag defaults to false and every string to empty, so the file only
/// has to state what is true of a column. Unknown fields are ignored, which
/// is what lets the file carry "notes" for whoever curates it without that
/// text being shipped to every client on every request.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct DatabaseColumn {
    #[serde(rename = "id")]
    pub column_id: String,

    #[serde(rename = "type")]
    pub column_type: ColumnType,

    /// Value a new text filter starts with, or the lower bound of a new
    /// range filter. Empty means no constraint.
    #[serde(default)]
    pub default_from: String,
    /// Upper bound of a new range filter. Empty means no constraint.
    #[serde(default)]
    pub default_to: String,

    /// Show this column in the table without the user asking
    #[serde(default)]
    pub show_by_default: bool,
    /// Offer the distinct values of this column as an autocomplete list
    #[serde(default)]
    pub dropdown: bool,
    /// May be shown as a column in the results table
    #[serde(default)]
    pub display: bool,
    /// May be used as a search filter
    #[serde(default)]
    pub search: bool,
    /// Included in the downloaded metadata file
    #[serde(default)]
    pub print: bool,
}

////////////////////////////////////////////////////////////
///
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct StrainRequest {
    pub list: Vec<String>,
}

////////////////////////////////////////////////////////////
///
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct SearchSettings {
    pub criteria: Vec<SearchCriteria>,
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

        let mut list_fields = Vec::new();
        for v in list_default {
            let mut c = SearchCriteria::new();
            c.field = "BTyperDB_ID".to_string();
            c.comparison = ComparisonType::Like(v);
            list_fields.push(c);
        }

        SearchSettings {
            criteria: list_fields,
        }
    }
}

////////////////////////////////////////////////////////////
/// One search criterion, e.g. a field should be <>= or like some value
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct SearchCriteria {
    pub field: String,
    pub comparison: ComparisonType,
}
impl SearchCriteria {
    ////////////////////////////////////////////////////////////
    /// Constructor
    pub fn new() -> SearchCriteria {
        SearchCriteria {
            field: "".to_string(),
            comparison: ComparisonType::Like("".to_string()),
        }
    }

    ////////////////////////////////////////////////////////////
    /// Genereate the default search criterion
    pub fn default_search(col: &DatabaseColumn) -> SearchCriteria {
        let comp = ComparisonType::default_comparison(col);
        SearchCriteria {
            field: col.column_id.clone(),
            comparison: comp,
        }
    }
}

////////////////////////////////////////////////////////////
/// A type of comparison for a field
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum ComparisonType {
    Like(String),
    FromTo(String, String),
}
impl ComparisonType {
    ////////////////////////////////////////////////////////////
    /// Generate a comparison with default fields
    pub fn default_comparison(db: &DatabaseColumn) -> ComparisonType {
        if db.column_type.is_numeric() {
            ComparisonType::FromTo(db.default_from.clone(), db.default_to.clone())
        } else {
            ComparisonType::Like(db.default_from.clone())
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
