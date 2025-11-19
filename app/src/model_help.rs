use crate::core_model::*;

use yew::prelude::*;


impl Model {

    ////////////////////////////////////////////////////////////
    /// Page: Help
    pub fn view_help_pane(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div>
            <div class="App-divider">{"Help"}</div>
            <div class="landingdiv">
                <div class="commontext">
                    <h1>
                        {"Sorry for the delay! We're under construction 🚧"}
                    </h1>
                    <p>
                        {"Help will be written here in the near future!"}
                    </p>
                    {""}
                </div>
            </div>
            <br />
            </div>
        }        
    }


}
