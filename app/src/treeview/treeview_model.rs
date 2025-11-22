use crate::{core_model::*};

use yew::{prelude::*};

use super::ReductionView;


impl Model {


    ////////////////////////////////////////////////////////////
    /// x
    pub fn view_tree_pane(&self, ctx: &Context<Self>) -> Html {
/*
        //Callback: Hovering a certain cell
        let on_cell_hovered = Callback::from(move |_name: Option<usize>| {
        });

        //Callback: Clicked on a cell
        let on_cell_clicked = Callback::from(move |_name: Vec<usize>| {
        });
 */

        //Callback: send message to component above
        let on_propagate= ctx.link().callback(move |sig: MsgCore| {
            log::debug!("propagate {:?}", sig);
            sig
        });
         
        html! {
            <div>
                <div> 
                    <ReductionView 
//                        on_cell_hovered={on_cell_hovered} 
//                        on_cell_clicked={on_cell_clicked} 
                        treedata={self.treedata.clone()}
                        on_propagate={on_propagate}
                        last_component_size={self.last_component_size.clone()}
                    />
                </div>
            </div>
        }
    }


}