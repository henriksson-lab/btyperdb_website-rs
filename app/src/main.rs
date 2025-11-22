pub mod core_model;
pub mod model_landing;
pub mod model_about;
pub mod model_search;
pub mod model_stats;
pub mod model_table;
pub mod model_help;
pub mod component_map;
pub mod download;
pub mod resize;
pub mod treeview;
pub mod appstate;

use crate::core_model::*;

////////////////////////////////////////////////////////////
/// x
fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<Model>::new().render();
}
