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

        let onchange_field = ctx.link().callback(move |e: Event | {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlSelectElement>().ok()).expect("wrong type");
            MsgCore::ChangedSearchFieldType(i, input.value())
        });

        let oninput_from = ctx.link().callback(move |e: Event | {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok()).expect("wrong type");
            MsgCore::ChangedSearchFieldFrom(i, input.value())
        });

        let oninput_to = ctx.link().callback(move |e: Event | {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok()).expect("wrong type");
            MsgCore::ChangedSearchFieldTo(i, input.value())
        });

        let oninput_like = ctx.link().callback(move |e: Event | {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok()).expect("wrong type");
            MsgCore::ChangedSearchFieldLike(i, input.value())
        });

        //Generate different controls depending on what type of comparison will be made
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

                if let Some(list_dropdown) = metadata.column_dropdown.get(&crit.field) {
                    //This ite has a suggested list of value via a dropdown
                    html! {
                        <label>
                            {" Is: "}
                            { elem_input }
                            <datalist>
                            {
                                list_dropdown.iter().map(|val| { 
                                    html!{
                                        <option>
                                            {val.clone()}
                                        </option>  
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

        //Figure out which fields we can search
        let mut list_select_options = Vec::new();
        for (colname,colmeta) in &metadata.columns {
            if colmeta.display {
                list_select_options.push(
                    html!{
                        <option value={colname.clone()} selected={*colname == crit.field}>
                            { colname.replace("_", " ") }
                        </option>
                    }
                );
            }
        }

        //HTML: all elements together
        html! {
			<div class="divSearchField">
				<button name="bDelete" class="buttonspacer" onclick={ctx.link().callback(move |_| MsgCore::DeleteSearchFilter(i))}>
                    {"X"}
                </button>
				<select class="columndrop" name="selectfield" onchange={onchange_field}>
                    {list_select_options}
				</select>
                { html_values }
			</div>
        }
        
    }



    ////////////////////////////////////////////////////////////
    /// Page: Search
    pub fn view_search_pane(&self, ctx: &Context<Self>) -> Html {

        let search_controls = if let Some(metadata) = &self.db_metadata {


            //Callback: Add a column to show
            let onchange_addcolumn = ctx.link().callback(move |e: Event | {
                let target: Option<EventTarget> = e.target();
                let input = target.and_then(|t: EventTarget| t.dyn_into::<HtmlSelectElement>().ok()).expect("wrong type");
                MsgCore::ShowColumn(input.value())
            });                        

            //Get list of columns
            let mut list_colstoadd = Vec::new();
            list_colstoadd.push(html! {
                <option selected={true}>{""}</option>
            });
            if let Some(db_metadata) = &self.db_metadata {
                for (colname,colmeta) in &db_metadata.columns {
                    if colmeta.display && !self.show_columns.contains(colname){
                        list_colstoadd.push(html! {
                            <option value={colname.clone()}> // id={colname.clone()} 
                                {colname.replace("_"," ")}
                            </option>
                        });
                    }
                }
            }

            //// Generate HTML: all buttons below the filters
            html! {
                <div>
                    <div class="withspacer"> /////////likely need to fix divs here
                    </div>
                    <div>
                        {
                            //List of filters
                            (0..self.search_settings.criteria.len()).into_iter().map(|i| { 
                                html!{  self.view_search_line(&ctx, metadata, i)  }
                            }).collect::<Html>()
                        }
                        <div>                        
                            <button class="buttonspacer" onclick={ctx.link().callback(|_| MsgCore::AddSearchFilter)}>
                                {"Add filter"}
                            </button>
                            <button class="buttonspacer" onclick={ctx.link().callback(|_| MsgCore::StartQuery)}>
                                {"Search"}
                            </button>
                            
                            <span class="commontext">
                                {"Add column to show: "}
                            </span>
                            <select class="columndrop" onchange={onchange_addcolumn}>
                                {list_colstoadd}
                            </select>                            
                        </div>
                    </div>
                </div>
            }

        } else {
            html! {{""}}
        };

        

        let visibility=self.show_search_controls;

        


        //Get list of selected strains
        let list_strains_withcomma = self.selected_strains.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",");

        //// Generate HTML: total search pane
        html! {
            <div>
                <div class="App-divider">
                    {"Search for genomes"}                
                    <button class="toggleview" onclick={ctx.link().callback(move |_| MsgCore::SetSearchControlVisibility(!visibility))}>
                        {
                            if self.show_search_controls { 
                                html!{"Hide panel"} 
                            } else { 
                                html!{"Show panel"} 
                            } 
                        }  
                    </button>
                </div>

                { if self.show_search_controls { search_controls } else { html!{""} } }  

                { self.view_table(&ctx) }

                <div class="downloadnav">
                    <button class="buttonspacer" onclick={ctx.link().callback(move |_e | {MsgCore::DownloadFASTA(IncludeData::Selected)})}>
                        {"FASTA: Download selected"}
                    </button>
                    <button class="buttonspacer" onclick={ctx.link().callback(move |_e | {MsgCore::DownloadFASTA(IncludeData::All)})}>
                        {"FASTA: Download displayed"}
                    </button>

                    <button class="buttonspacer" onclick={ctx.link().callback(move |_e | {MsgCore::DownloadMetadata(IncludeData::Selected)})}>
                        {"Metadata: Download selected"}
                    </button>
                    <button class="buttonspacer" onclick={ctx.link().callback(move |_e | {MsgCore::DownloadMetadata(IncludeData::All)})}>
                        {"Metadata: Download displayed"}
                    </button>

                    
                    <form target="_blank" method="get" action="https://nextstrain.org/community/vigzy77/BTracker/Bacillus-cereus-group/All-Species">
                        <input type="hidden" name="s" value={list_strains_withcomma.clone()}/>
                        <button class="buttonspacer" disabled={list_strains_withcomma.is_empty()}>
                            {"Open selected in BTracker"}
                        </button>
                    </form>
                </div>

            </div>
        }
        
    }



}
