use crate::core_model::*;
use wasm_bindgen::JsCast;

use my_web_app::{ComparisonType, DatabaseMetadata};
use web_sys::{EventTarget, HtmlInputElement};
use yew::{
    prelude::*,
};

impl Model {

    ////////////////////////////////////////////////////////////
    /// One of the search fields
    pub fn view_search_line(&self, ctx: &Context<Self>, metadata: &DatabaseMetadata, i: usize) -> Html {

        let crit = self.search_settings.criteria.get(i).unwrap();

        //meah https://yew.rs/docs/concepts/html/events
        // check https://docs.rs/yew-components/latest/src/yew_components/select.rs.html

        let onchange_field = Callback::from(move |e: Event | {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            if let Some(input) = input {
                Msg::ChangedSearchFieldType(i, input.value());
            }
        });
        let oninput_from = Callback::from(move |e: Event | {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            if let Some(input) = input {
                Msg::ChangedSearchFieldFrom(i, input.value());
            }
        });
        let oninput_to = Callback::from(move |e: Event | {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            if let Some(input) = input {
                Msg::ChangedSearchFieldTo(i, input.value());
            }
        });
        let oninput_like = Callback::from(move |e: Event | {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            if let Some(input) = input {
                Msg::ChangedSearchFieldLike(i, input.value());
            }
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
                        <input class="textbox" type="text" name="value1" value={from.clone()} onchange={oninput_from}/> 
                        {" To: "}
                        <input class="textbox" type="text" name="value2" value={to.clone()} onchange={oninput_to}/>
                    </label>				
                }
            },
            ComparisonType::Like(v) => {
                html! {
                    <label>
                        {" Is: "}
                        <input class="textbox" type="text" name="value" value={v.clone()} onchange={oninput_like}/> 
                    </label>				
                }
            }

            
        };


        html! {
			<div class="divSearchField">
				<button name="bDelete" class="buttonspacer" onclick={ctx.link().callback(move |_| Msg::DeleteSearchFilter(i))}>
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
    pub fn view_search_pane(&self, ctx: &Context<Self>) -> Html {

        let search_controls = if let Some(metadata) = &self.db_metadata {

            html! {
                <div>
                    <div class="withspacer"> /////////likely need to fix divs here
                    </div>
                    <div>
                        {
                            (0..self.search_settings.criteria.len()).into_iter().map(|i| { 
                                html!{  self.view_search_line(&ctx, metadata, i)  }
                            }).collect::<Html>()
                        }
                        <div>                        
                            <button class="buttonspacer" onclick={ctx.link().callback(|_| Msg::AddSearchFilter)}>
                                {"Add filter"}
                            </button>
                            <button class="buttonspacer" onclick={ctx.link().callback(|_| Msg::StartQuery)}>
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
                    <button class="toggleview" onclick={ctx.link().callback(move |_| Msg::SetSearchControlVisibility(!visibility))}>
                        { if self.show_search_controls { html!{"Hide panel"} } else { html!{"Show panel"} } }  
                    </button>
                </div>

                { if self.show_search_controls { search_controls } else { html!{""} } }  

                { self.view_table(&ctx) }

                <div class="downloadnav">
                    <button class="buttonspacer" onclick={ctx.link().callback(move |_e | {Msg::DownloadFASTA(IncludeData::Selected)})}>
                        {"FASTA: Download selected"}
                    </button>
                    <button class="buttonspacer" onclick={ctx.link().callback(move |_e | {Msg::DownloadFASTA(IncludeData::All)})}>
                        {"FASTA: Download all"}
                    </button>

                    <button class="buttonspacer" onclick={ctx.link().callback(move |_e | {Msg::DownloadMetadata(IncludeData::Selected)})}>
                        {"Metadata: Download selected"}
                    </button>
                    <button class="buttonspacer" onclick={ctx.link().callback(move |_e | {Msg::DownloadMetadata(IncludeData::All)})}>
                        {"Metadata: Download all"}
                    </button>
                </div>

            </div>
        }
        
    }



}
