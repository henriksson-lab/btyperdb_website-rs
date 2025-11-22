use crate::core_model::*;

use my_web_app::TableData;
use yew::prelude::*;

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
    /// Generate HTML for one row in the table
    pub fn view_table_row(&self, ctx: &Context<Self>, dt: &TableData, row: &Vec<String>, show_cols: &Vec<usize>) -> Html {
        let btyper_id = row.get(0).expect("Could not get first column of row to use as id");

        let is_selected=self.selected_strains.contains(btyper_id);

        let btyper_id_copy= btyper_id.clone();
        let onclick: Callback<MouseEvent> = ctx.link().callback(move |_e | {
            MsgCore::SetStrainSelected(btyper_id_copy.clone(), !is_selected)
        });

        let max_text_len = 40;

        html! {
            <tr key={btyper_id.clone()}>
                <td>
                    <input type="checkbox" key="check" onclick={onclick} checked={is_selected}/> 
                </td>
                {
                    show_cols.iter().map(|i| {

                        let cur_column = dt.columns.get(*i).expect("no such column");
                        let txt = row.get(*i).expect("no such column").clone();

                        //Figure out link to other page, if applicable
                        let ahref = if cur_column=="NCBI_BioProject" {
                            Some(format!("https://www.ncbi.nlm.nih.gov/bioproject/?term={}",txt).to_string())
                        } else if cur_column=="NCBI_BioSample" {
                            Some(format!("https://www.ncbi.nlm.nih.gov/biosample/?term={}",txt).to_string())
                        } else if cur_column=="NCBI_Assembly" {
                            Some(format!("https://www.ncbi.nlm.nih.gov/datasets/genome/{}",txt).to_string())
                        } else if cur_column=="NCBI_Experiment_Accession" || cur_column=="NCBI_Run_Accession" {
                            Some(format!("https://www.ncbi.nlm.nih.gov/sra/?term={}",txt).to_string())
                        } else {
                            None
                        };

                        /////////// if having links, need to split by , 


                        //Shorten column text if needed
                        let txt_html = if txt.len() > max_text_len {
                            html!{ <span title={txt.clone()}> {txt[0..max_text_len].to_string()} {"..."}</span> }  // NOTE: dangerous to have a class on span directly!!!!!!
                        } else {
                            html!{ {txt} }
                        };

                        //Put all html together
                        let txt_link = if let Some(url) = ahref {
                            if url != "" {
                                html!{
                                    <a href={url}>
                                        {txt_html}
                                    </a>
                                }
                            } else {
                                txt_html
                            }
                        } else {
                            txt_html
                        };
                        html!{<td key={*i} class="tablecontent"> {txt_link} </td>}
                    }).collect::<Html>()
                }
            </tr>
        }        
    }


    ////////////////////////////////////////////////////////////
    /// Generate HTML for the entire table
    pub fn view_table(&self, ctx: &Context<Self>) -> Html {

        if let Some(dt) = &self.tabledata {

            //Check if table empty
            if dt.rows.len()==0 {
                html! {"(Table is empty)"}
            } else {

                //Figure out range of table rows to display
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
                            <span class="commontext">
                                {"Go to page: "}
                            </span>
                            {
                                possible_pages.into_iter().map(move |p| {
                                    
                                    let onclick = ctx.link().callback(move |_e | {
                                        MsgCore::SetTableFrom(p*entries_per_page)
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
                    if self.show_columns.contains(colname) {
                        log::debug!("show col: {}",colname);
                        show_cols.push(i);
                    }
                }

                ///// Generate the header
                let html_header = html! {
                    <tr>
                        <td>
                            // column for checkboxes. empty header
                        </td>
                        {
                            show_cols.iter().map(|i| {
                                let txt = dt.columns.get(*i).expect("Could not get column");

                                //Callback: Removal of column
                                let copy_colname = txt.clone();
                                let remove_onclick = ctx.link().callback(move |_e: MouseEvent | {
                                    MsgCore::HideColumn(copy_colname.clone())
                                });

                                //Generate HTML for column header
                                let pretty_txt = str::replace(txt, "_", " ");
                                html!{
                                    <th key={*i} class="tableheader"> 
                                        {pretty_txt} 
                                        <button onclick={remove_onclick} class="hidecolumnbutton">{"X"}</button>
                                    </th>
                                }
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
                                    html!{  self.view_table_row(&ctx, &dt, &dt.rows.get(i).expect("could not find row"), &show_cols)  }
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
