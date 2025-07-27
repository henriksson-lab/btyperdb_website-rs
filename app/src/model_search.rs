use crate::core_model::*;

use my_web_app::{ComparisonType, DatabaseMetadata};
use yew::{
    prelude::*,
};

impl Model {

    ////////////////////////////////////////////////////////////
    /// One of the search fields
    pub fn view_search_line(&self, metadata: &DatabaseMetadata, i: usize) -> Html {

        let crit = self.search_settings.criteria.get(i).unwrap();
        
        //meah https://yew.rs/docs/concepts/html/events
        // check https://docs.rs/yew-components/latest/src/yew_components/select.rs.html

        let onchange_field = self.link.callback(move |e: ChangeData | {
            Msg::ChangedSearchFieldType(i, e)
        });
        // batch_callback  use instead
        let oninput_from = self.link.callback(move |e: InputData | {
            Msg::ChangedSearchFieldFrom(i, e.value)
        });
        let oninput_to = self.link.callback(move |e: InputData | {
            Msg::ChangedSearchFieldTo(i, e.value)
        });
        let oninput_like = self.link.callback(move |e: InputData | {
            Msg::ChangedSearchFieldLike(i, e.value)
        });
        
        //log::debug!("render {:?}",crit);


        //let coltype = metadata.columns.get(&crit.field.clone()).unwrap().column_type.clone();
        //let is_ranged_type = coltype=="integer" || coltype=="float";
        //possible values: text  integer float   

        let html_values = match &crit.comparison {
            ComparisonType::FromTo(from,to) => {
                html! {
                    <label>
                        {" From: "}
                        <input class="textbox" type="text" name="value1" value={from.clone()} oninput={oninput_from}/> 
                        {" To: "}
                        <input class="textbox" type="text" name="value2" value={to.clone()} oninput={oninput_to}/>
                    </label>				
                }
            },
            ComparisonType::Like(v) => {
                html! {
                    <label>
                        {" Is: "}
                        <input class="textbox" type="text" name="value" value={v.clone()} oninput={oninput_like}/> 
                    </label>				
                }
            }

            
        };


        html! {
			<div class="divSearchField">
				<button name="bDelete" class="buttonspacer" onclick=self.link.callback(move |_| Msg::DeleteSearchFilter(i))>
                    {"X"}
                </button>
				<select class="columndrop" name="selectfield" onchange={onchange_field}>
                    {
                        metadata.columns.keys().clone().into_iter().map(|col| { /////////////////////////////////////////////// check why so much cloning needed
                            html!{
                                <option value={col.clone()} selected={*col == crit.field}>  //////  selected="selected"  if the one
                                    { col.clone() }
                                </option>
                            }
                        }).collect::<Html>()
                    }
				</select>
                { html_values }
			</div>
        }
        
    }



    ////////////////////////////////////////////////////////////
    /// Page: Search
    pub fn view_search_pane(&self) -> Html {

        let search_controls = if let Some(metadata) = &self.db_metadata {

            html! {
                <div>
                    <div class="withspacer"> /////////likely need to fix divs here
                    </div>
                    <div>
                        {
                            (0..self.search_settings.criteria.len()).into_iter().map(|i| { 
                                html!{  self.view_search_line(metadata, i)  }
                            }).collect::<Html>()
                        }
                        <div>                        
                            <button class="buttonspacer" onclick=self.link.callback(|_| Msg::AddSearchFilter)>
                                {"Add filter"}
                            </button>
                            <button class="buttonspacer" onclick=self.link.callback(|_| Msg::StartQuery)>
                                {"Search"}
                            </button>
                        </div>
                    </div>
                </div>
            }

        } else {
            html! {{""}}
        };

        

        let visibility=self.show_search_controls;

        html! {
            <div>
                <div class="App-divider">
                    {"Search for genomes"}                
                    <button class="toggleview" onclick=self.link.callback(move |_| Msg::SetSearchControlVisibility(!visibility))>
                        { if self.show_search_controls { html!{"Hide panel"} } else { html!{"Show panel"} } }  
                    </button>
                </div>

                { if self.show_search_controls { search_controls } else { html!{""} } }  

                { self.view_table() }

            </div>
        }
        
    }



}
