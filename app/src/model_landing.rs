use crate::core_model::*;

use yew::{
    prelude::*,
};


////////////////////////////////////////////////////////////
/// If condition is met, return "selected", otherwise "". For OPTION
pub fn selected_if(cond: bool) -> String {
    if cond {
        "selected".to_string()
    } else {
        "".to_string()
    }
}   // can do true false https://yew.rs/docs/concepts/html 



impl Model {


    ////////////////////////////////////////////////////////////
    /// x
    pub fn view_landing_page(&self) -> Html {

        let num_strain = if let Some(metadata) = &self.db_metadata {
            format!("{}", metadata.num_strain)
        } else {
            "___".to_string()
        };

        html! {

            <div class="landingdiv">

                <img src="assets/Btyperdb_logo.svg" alt="rust image"/> 
                <p style="color: rgb(0, 150, 255);">
                    {"A community curated, global atlas of Bacillus cereus group genomes"}
                </p>

                <p style="color: rgb(0, 150, 255);">
                    {"Database version v1"}
                </p>

                <p style="color: rgb(0, 150, 255);">
                    {num_strain} {" total B. cereus group genomes with curated metadata"}
                </p>

                <button class="toolbutton" onclick=self.link.callback(|_| Msg::OpenPage(CurrentPage::Search))>
                    {"Search BTyperDB"}
                </button>

/*
                <button class="toolbutton" onclick=self.link.callback(|_| Msg::FetchDatabaseMetadata)>
                    {"debug BTyperDB"}
                </button>
 */


            </div>
        }
    }



}
