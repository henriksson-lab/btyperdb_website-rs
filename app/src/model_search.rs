use crate::core_model::*;
use wasm_bindgen::JsCast;

use my_web_app::{ComparisonType, DatabaseMetadata};
use web_sys::{EventTarget, HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

impl Model {

    ////////////////////////////////////////////////////////////
    /// One of the search fields
    pub fn view_search_line(&self, ctx: &Context<Self>, metadata: &DatabaseMetadata, i: usize) -> Html {

        let crit = self.search_settings.criteria.get(i).unwrap();

        //meah https://yew.rs/docs/concepts/html/events
        // check https://docs.rs/yew-components/latest/src/yew_components/select.rs.html

        let onchange_field = ctx.link().callback(move |e: Event | {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlSelectElement>().ok()).expect("wrong type");
            Msg::ChangedSearchFieldType(i, input.value())
        });

        let oninput_from = ctx.link().callback(move |e: Event | {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok()).expect("wrong type");
            Msg::ChangedSearchFieldFrom(i, input.value())
        });

        let oninput_to = ctx.link().callback(move |e: Event | {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok()).expect("wrong type");
            Msg::ChangedSearchFieldTo(i, input.value())
        });

        let oninput_like = ctx.link().callback(move |e: Event | {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok()).expect("wrong type");
            Msg::ChangedSearchFieldLike(i, input.value())
        });

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

                let elem_input = html! { <input class="textbox" type="text" name="value" value={v.clone()} onchange={oninput_like} list={crit.field.clone()}/> };

                let drop = metadata.column_dropdown.get(&crit.field);
                log::debug!("drop {:?}", drop);

                if let Some(list_dropdown) = metadata.column_dropdown.get(&crit.field) {

                    html! {
                        <label>
                            {" Is: "}
                            { elem_input }
                            <datalist id={crit.field.clone()}>
                            {
                                list_dropdown.iter().map(|val| { 
                                    html!{
                                        <option value={val.clone()} />  
                                    }
                                }).collect::<Html>()
                            }
                            </datalist>
                        </label>				
                    }

                } else {
                    html! {
                        <label>
                            {" Is: "}
                            { elem_input }
                        </label>				
                    }

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
                        metadata.columns.keys().clone().into_iter().filter(|col| !col.starts_with("mapcol_")).map(|col| { /////////////////////////////////////////////// check why so much cloning needed
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
