pub mod core_model;
pub mod model_landing;
pub mod model_about;
pub mod model_search;
pub mod model_stats;
pub mod model_table;
pub mod model_help;
pub mod component_map;

use crate::core_model::*;

////////////////////////////////////////////////////////////
/// x
fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}
