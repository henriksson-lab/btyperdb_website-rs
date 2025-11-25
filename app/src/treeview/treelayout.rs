
use std::collections::{HashMap, HashSet};

use phylo::prelude::*;

use crate::treeview::Rectangle2D;

////////////////////////////////////////////////////////////
/// x
#[derive(Debug)]
pub struct TreeLayout {

    pub tree: SimpleRootedTree<String, f32, f32>,

    pub map_name_to_id: HashMap<String, usize>,

    pub list_x: Vec<f32>,
    pub list_y: Vec<f32>,

    pub max_x: f32,
    pub max_y: f32,

    pub min_x: f32,
    pub min_y: f32,

    pub vec_owner: Vec<usize>,
    pub vec_vertex: Vec<f32>,
    pub gl_num_lines: u32,
}
impl TreeLayout {


    ////////////////////////////////////////////////////////////
    /// x
    pub fn new(newick_str: &str) -> TreeLayout {

//        let newick_str = "((A:0.1,B:0.2)F:0.6,(C:0.3,D:0.4)E:0.5)G;";
//        let newick_str = include_str!("parsnp2_noC_snps.fna.treefile"); 
//        let newick_str = include_str!("All-Species_rooted.nwk"); 

        log::debug!("reading phylo tree; how can it be so slow??");
        let tree: SimpleRootedTree<String, f32, f32> = PhyloTree::from_newick(newick_str.as_bytes()).expect("Could not parse tree");

        let root = tree.get_root_id();

        log::debug!("Num nodes: {}",tree.num_nodes());

        //Create default set of y-positions (all to be overwritten)
        let mut list_y = Vec::with_capacity(tree.num_nodes());
        for _i in 0..tree.num_nodes() {
            list_y.push(0.0);
        }

        //Store name of all nodes
        let mut map_name_to_id:HashMap<String, usize> = HashMap::new();
        for node in tree.get_nodes() {
            let name = node.get_taxa();
            if let Some(name) = name {
                map_name_to_id.insert(name.clone(), node.get_id());
            } 
        }

        //Figure out y-position of all entries
        let mut next_y=0.0;
        for node_id in tree.postord_ids(root) {
            if tree.is_leaf(node_id) {
                //Set y-coordinate to new position
                let this_y = list_y.get_mut(node_id).expect("foo");
                *this_y = next_y;
                next_y += 1.0;
            } else {
                //Set y-coordinate to average of children
                let mut sum_y = 0.0;
                let mut num_children = 0;
                
                for child_id in tree.get_node_children_ids(node_id) {
                    sum_y += list_y.get(child_id).expect("Could not get child");
                    num_children += 1;
                }
                let this_y = list_y.get_mut(node_id).expect("foo");
                *this_y = sum_y / (num_children as f32);
            }
        }
        //log::debug!("final y {}", next_y);



        //Figure out x-position of all entries        
        let list_x = TreeLayout::calc_x(&tree);

        //log::debug!("x: {:?}", list_x);
        //log::debug!("y: {:?}", list_y);

        //Figure out extent of diagram, for camera
        let max_y=next_y;
        let min_x=0.0;
        let min_y=0.0;

        let mut max_x=0.0;
        for v in &list_x {
            if *v > max_x {
                max_x=*v;
            }
        }

        //Lines, set up coordinates from-to. Too few vertices to make it worth sharing data
        let mut vec_owner: Vec<usize> = Vec::new();
        let mut vec_vertex: Vec<f32> = Vec::new();
        vec_vertex.reserve(tree.num_nodes()*2);
        let mut gl_num_lines=0;        
        for node_id in tree.postord_ids(root) {

            let node_x = *list_x.get(node_id).unwrap();
            //let node_y = *list_y.get(node_id).unwrap();

            let mut child_min_y = f32::MAX;
            let mut child_max_y = f32::MIN;
            let mut has_children= false;

            for child_id in tree.get_node_children_ids(node_id) {

                let child_x = *list_x.get(child_id).unwrap();
                let child_y = *list_y.get(child_id).unwrap();
                
                //Could we look at first and last instead? todo
                if child_max_y < child_y {
                    child_max_y = child_y;
                }
                if child_min_y > child_y {
                    child_min_y = child_y;
                }

                //log::debug!("parent node {}", node_id);
                vec_vertex.push(node_x);
                vec_vertex.push(child_y);
                vec_vertex.push(child_x);
                vec_vertex.push(child_y);
                vec_owner.push(child_id);
                //log::debug!("pvertex {:?}", vec_vertex);

                gl_num_lines += 1;   

                has_children = true;
            }

            //Vertical line at parent
            if has_children {
                vec_vertex.push(node_x);
                vec_vertex.push(child_min_y);
                vec_vertex.push(node_x);
                vec_vertex.push(child_max_y);
                vec_owner.push(node_id);
                gl_num_lines += 1;   
            }


        }
        //log::debug!("num lines {}", gl_num_lines);
        //log::debug!("vertex {:?}", vec_vertex);


        TreeLayout {
            tree,
            map_name_to_id,

            list_x,
            list_y,
            max_x,
            min_x,
            max_y,
            min_y,

            vec_owner,
            vec_vertex,
            gl_num_lines,
        }
        
    }


    ////////////////////////////////////////////////////////////
    /// Get nodeIDs from list of strain names.
    /// Ignore missing strain names
    pub fn get_ids_from_names(&self, list_strains: Vec<String>) -> Vec<usize> {
        let mut list_ids:Vec<usize> = Vec::new();
        for s in &list_strains {
            let id = self.map_name_to_id.get(s);
            if let Some(id)=id {
                list_ids.push(*id);
            }
        }
        list_ids
    }


    ////////////////////////////////////////////////////////////
    /// The equivalent of R groupOTU
    pub fn select_common_ancestors(&self, list_branches: Vec<usize>) -> HashSet<usize> {

        let mut list_sel:HashSet<usize> = HashSet::new();
        for v in &list_branches {
            list_sel.insert(*v);
        }

        //Scan bottom-up to fill in common ancestors
        let root = self.tree.get_root_id();
        for node_id in self.tree.postord_ids(root) {

            //Check if all children are selected
            let mut all_children_in_sel = true;
            for child_id in self.tree.get_node_children_ids(node_id) {
                if !list_sel.contains(&child_id) {
                    all_children_in_sel = false;
                    break;
                }
            }

            if all_children_in_sel {
                list_sel.insert(node_id);
            }
        }

        list_sel
    }




    ////////////////////////////////////////////////////////////
    /// x
    pub fn get_bounding_rect(&self) -> Rectangle2D {
        Rectangle2D {
            x1: self.min_x,
            x2: self.max_x,

            y1: self.min_y,
            y2: self.max_y,
        }
    }



    ////////////////////////////////////////////////////////////
    /// x
    fn calc_x<T,W,Z>(tree: &SimpleRootedTree<T,W,Z>) -> Vec<W> where 
            T: NodeTaxa,
            W: EdgeWeight,
            Z: NodeWeight {

        let mut list_x = Vec::with_capacity(tree.num_nodes());
        for _i in 0..tree.num_nodes() {
            list_x.push(W::zero());
        }

        let cur_node = tree.get_root_id();
        let next_w = W::zero();
        TreeLayout::calc_x_recursive(tree, &mut list_x, cur_node, next_w);

        list_x
    }


        
    ////////////////////////////////////////////////////////////
    /// x
    fn calc_x_recursive<T,W,Z>(tree: &SimpleRootedTree<T,W,Z>, list_x: &mut Vec<W>, cur_node: usize, cur_x: W) where 
            T: NodeTaxa,
            W: EdgeWeight,
            Z: NodeWeight {

        for child_id in tree.get_node_children_ids(cur_node) {
                let w= tree.get_edge_weight(cur_node, child_id).expect("no edge weight");
                let next_w = cur_x + w;
                let this_x = list_x.get_mut(child_id).expect("foo");
                *this_x = next_w;
                TreeLayout::calc_x_recursive(tree, list_x, child_id, next_w);
            }

        }


}