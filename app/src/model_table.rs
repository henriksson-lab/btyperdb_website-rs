use crate::core_model::*;

use yew::{
    prelude::*,
};

impl Model {


    ////////////////////////////////////////////////////////////
    /// x
    pub fn view_table_row(&self, row: &Vec<String>) -> Html {
        let btyper_id = row.get(0).expect("Could not get first column of row to use as id");
        html! {
            <tr key={btyper_id.clone()}>
                {
                    row.clone().iter().enumerate().map(|(i, val)| {  /////////////////////////////////////////////// check why so much cloning needed
                        html!{<td key={i}>{ val.clone() }</td>}
                    }).collect::<Html>()
                }
            </tr>
        }        
    }


    ////////////////////////////////////////////////////////////
    /// x
    pub fn view_table(&self) -> Html {

        if let Some(dt) = &self.tabledata {

            if dt.rows.len()==0 {
                html! {"table has no data"}
            } else {
                let html_header = html! {
                    <tr>
                        {
                            dt.columns.clone().into_iter().map(|name| { /////////////////////////////////////////////// check why so much cloning needed
                                html!{<th key={name.clone()}>{ name.clone() }</th>}
                            }).collect::<Html>()
                        }
                    </tr>
                };

                let entries_per_page = 100;

                let from_row = self.tabledata_from;
                let mut to_row = from_row + entries_per_page;
                if to_row > dt.rows.len() {
                    to_row = dt.rows.len();
                }

                let show_rows = from_row..to_row;

                //log::debug!("showrows {:?}", show_rows);
                //// Generate all pages
                let possible_pages = 0..(1+(dt.rows.len()/entries_per_page));
                let div_gotopage = if possible_pages.len()>1 { 

                    html! {
                        <div>
                            {"Go to page: "}
                            {
                                possible_pages.into_iter().map(move |p| {
                                    
                                    let onclick = self.link.callback(move |_e | {
                                        Msg::SetTableFrom(p*entries_per_page)
                                    });

                                    html! { 
                                        <label onclick={onclick}> 
                                            {format!("{} ",p+1)}   /////// possible to highlight current page here
                                        </label>
                                    }
                                }).collect::<Html>()
                            }
                        </div>
                    }

                } else {
                    html!{ {""}}
                };


                html! {
                    <div>
                        { div_gotopage }
                        <table class="divtable2">
                            ///// The table header
                            { html_header }
                            ///// All rows in the table
                            {
                                show_rows.into_iter().map(|i| { 
                                    html!{  self.view_table_row(&dt.rows.get(i).expect("could not find row"))  }
                                }).collect::<Html>()
                            }
                        </table>
                    </div>        
                } 
            }
           
        } else {
            html! {"dt = null"}
        }        
    }



}
