use crate::core_model::*;

use yew::{
    prelude::*,
};


impl Model {

    ////////////////////////////////////////////////////////////
    /// Page: Help
    pub fn view_help_pane(&self) -> Html {
        html! {
            <div>
            <div class="App-divider">{"Help"}</div>
            <div class="landingdiv">
                <h1>
                    {"Sorry for the delay! We're under construction 🚧"}
                </h1>
                <p>
                    {"Help will be written here in the near future!"}
                </p>
                {"&nbsp;"}
                
            </div>
            <br />
            </div>
        }        
    }


}
