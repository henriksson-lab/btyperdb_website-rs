use crate::core_model::*;

use yew::{
    prelude::*,
};

impl Model {


    ////////////////////////////////////////////////////////////
    /// x
    pub fn get_strains(&self, inc: &IncludeData) -> Vec<String> {
    
        //list_strains.push("BTDB_2022-0001042.1".to_string());

        let mut list_strains = Vec::new();
        match inc {
            IncludeData::All => {
                if let Some(tabledata) = &self.tabledata {
                    for r in &tabledata.rows {
                        let id = r.get(0).expect("no col 0"); /////////////////// is this btyperid?
                        list_strains.push(id.clone());
                    }
                }
            },
            IncludeData::Selected => {
                for e in &self.selected_strains {
                    list_strains.push(e.clone());
                }
            }                    
        }
        list_strains
    }


    ////////////////////////////////////////////////////////////
    /// x
    pub fn view_table_row(&self, ctx: &Context<Self>, row: &Vec<String>, show_cols: &Vec<usize>) -> Html {
        let btyper_id = row.get(0).expect("Could not get first column of row to use as id");

        let is_selected=self.selected_strains.contains(btyper_id);

        let btyper_id_copy= btyper_id.clone();
        let onclick: Callback<MouseEvent> = ctx.link().callback(move |_e | {
            Msg::SetStrainSelected(btyper_id_copy.clone(), !is_selected)
        });

        let max_text_len = 40;

        html! {
            <tr key={btyper_id.clone()}>
                <td>
                    <input type="checkbox" key="check" onclick={onclick} checked={is_selected}/>  /////////// checked="checked"
                </td>
                {
                    show_cols.iter().map(|i| {

                        let txt = row.get(*i).expect("no such column").clone();

                        if txt.len() > max_text_len {
                            html!{<td key={*i}>  <span title={txt.clone()}> {txt[0..max_text_len].to_string()} {"..."}</span> </td>}
                        } else {
                            html!{<td key={*i}> {txt} </td>}
                        }
                    }).collect::<Html>()
                }
            </tr>
        }        
    }


    ////////////////////////////////////////////////////////////
    /// x
    pub fn view_table(&self, ctx: &Context<Self>) -> Html {

        if let Some(dt) = &self.tabledata {

            if dt.rows.len()==0 {
                html! {"(Table is empty)"}
            } else {


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
                                    
                                    let onclick = ctx.link().callback(move |_e | {
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

                ///// Decide on columns to show
                let mut show_cols = Vec::new();
                for (i,colname) in dt.columns.iter().enumerate() {
                    if !colname.starts_with("matchcol_") {
                        show_cols.push(i);
                    }
                }


                ///// Generate the header
                let html_header = html! {
                    <tr>
                        <td>
                            //// space for checkboxes
                        </td>
                        {
                            show_cols.iter().map(|i| {
                                let txt = dt.columns.get(*i).expect("Could not get column");
                                html!{<th key={*i}> {txt} </th>}
                            }).collect::<Html>()
                        }
                    </tr>
                };




                ///// Put it all together
                html! {
                    <div>
                        { div_gotopage }
                        <table class="divtable2">
                            ///// The table header
                            { html_header }
                            ///// All rows in the table
                            {
                                show_rows.into_iter().map(|i| { 
                                    html!{  self.view_table_row(&ctx, &dt.rows.get(i).expect("could not find row"), &show_cols)  }
                                }).collect::<Html>()
                            }
                        </table>
                    </div>        
                } 
            }
           
        } else {
            html! {""}
        }        
    }



}
