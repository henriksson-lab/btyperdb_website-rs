use std::io::Cursor;

use anyhow::Result;

use my_web_app::DatabaseMetadata;
use my_web_app::TableData;
use my_web_app::SearchSettings;
use my_web_app::SearchCriteria;

use yew::web_sys;
use yew::{
    format::{Json, Nothing},
    //worker::Context,
    prelude::*,
    services::{
        fetch::{FetchTask, Request, Response},
        FetchService,
    },
};


////////////////////////////////////////////////////////////
/// Which page is currently being shown?
#[derive(Debug)]
#[derive(PartialEq)]
enum CurrentPage {
    Home,
    Search,
    Statistics,
    Help,
    About,
}

////////////////////////////////////////////////////////////
/// Message sent to the event system for updating the page
#[derive(Debug)]
enum Msg {

    OpenPage(CurrentPage),
    StartQuery,
    SetQuery(Option<TableData>),
    SetSearchControlVisibility(bool),
    AddSearchFilter,
    DeleteSearchFilter(usize),
    SetDatabaseMetadata(DatabaseMetadata),
    FetchDatabaseMetadata,

    ChangedSearchField(usize, String),
}

////////////////////////////////////////////////////////////
/// State of the page
struct Model {
    link: ComponentLink<Self>,
    current_page: CurrentPage,
    tabledata: Option<TableData>,
    tabledata_from: usize,
    task: Option<FetchTask>, ///////////// why do we keep this?
    show_search_controls: bool,
    search_settings: SearchSettings,
    db_metadata: Option<DatabaseMetadata>
}

impl Component for Model {
    type Message = Msg;

    type Properties = ();

    ////////////////////////////////////////////////////////////
    /// Create a new component
    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {


        // For testing
        let tabledata:TableData = serde_json::from_reader(Cursor::new(include_bytes!("testdata.json"))).unwrap();

        // For testing
        //let tablemeta:DatabaseMetadata = serde_json::from_reader(Cursor::new(include_bytes!("testmeta.json"))).unwrap();

        let mut comp = Self {
            link,
            current_page: CurrentPage::Home,
            tabledata: Some(tabledata), //None,
            tabledata_from: 0,
            task: None,
            show_search_controls: true,
            search_settings: SearchSettings::new(),
            db_metadata: None
        };

        //Get metadata about database right away
        comp.update(Msg::FetchDatabaseMetadata);

        comp
    }

    ////////////////////////////////////////////////////////////
    /// Handle an update message
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {

            Msg::OpenPage(page) => {
                self.current_page = page;
                true
            }


            Msg::StartQuery => {
                let request = Request::get("/straindata")
                    .body(Nothing)
                    .expect("Could not build request");
                let callback =
                    self.link
                        .callback(|response: Response<Json<Result<TableData>>>| {
                            //log::debug!("{:?}", response);
                            let Json(data) = response.into_body();
                            Msg::SetQuery(data.ok())
                        });
                let task = FetchService::fetch(request, callback).expect("Failed to start request");
                self.task = Some(task);
                false
            }





            Msg::FetchDatabaseMetadata => {
                //Ask the server for metadata
                let request = Request::get("/strainmeta")
                        .body(Nothing)
                        .expect("Could not build request");
                        
                let callback =
                    self.link
                        .callback(|response: Response<Json<Result<DatabaseMetadata>>>| {
                            //log::debug!("got metadata {:?}", response);
                            let Json(data) = response.into_body();
                            Msg::SetDatabaseMetadata(data.ok().expect("metadata fail"))
                        });
                let task = FetchService::fetch(request, callback).expect("Failed to start request");
                self.task = Some(task);
                false
            }



            Msg::SetQuery(data) => {
                //log::trace!("SetQuery: {:?}", data);
                self.tabledata = data;
                self.tabledata_from = 0;
                true
            }



            Msg::SetDatabaseMetadata(data) => {
                //log::trace!("SetDatabaseMetadata: {:?}", data);
                self.db_metadata = Some(data);

                //TODO: populate search box

                true
            }


            Msg::SetSearchControlVisibility(data) => {
                //log::trace!("SetSearchControlVisibility: {:?}", data);
                self.show_search_controls = data;
                true
            }

            Msg::AddSearchFilter => {
                //log::trace!("AddSearchFilter: {:?}", data);
                let c = SearchCriteria::new();
                self.search_settings.criteria.push(c);
                true
            },

            Msg::DeleteSearchFilter(i) => {
                //log::trace!("DeleteSearchFilter: {:?}", data);
                self.search_settings.criteria.remove(i);
                true
            }


            Msg::ChangedSearchField(i, val) => {

                self.search_settings.criteria.get_mut(i).expect("boho").from=val;

/*
                self.db_metadata.iter_mut().map(|metadata| {

                    metadata.columns.get_mut(i).unwrap().

                });
 */

                false
            }
            


        }
    }



    ////////////////////////////////////////////////////////////
    /// x
    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }


    ////////////////////////////////////////////////////////////
    /// Top renderer of the page
    fn view(&self) -> Html {

        let current_page = match self.current_page { 
            CurrentPage::Home => self.view_landing_page(),
            CurrentPage::Search => self.view_search_pane(),
            CurrentPage::Statistics => self.view_statistics_pane(),
            CurrentPage::Help => self.view_help_pane(),
            CurrentPage::About => self.view_about_pane()
        };

        let html_top_buttons = html! {
            <header class="App-header">
                <div id="topmenu" class="topnav">
                    <div class="topnav-right">
                        <a class=active_if(self.current_page==CurrentPage::Home)       onclick=self.link.callback(|_| Msg::OpenPage(CurrentPage::Home))>{"Home"}</a>
                        <a class=active_if(self.current_page==CurrentPage::Search)     onclick=self.link.callback(|_| Msg::OpenPage(CurrentPage::Search))>{"Search"}</a>
                        <a class=active_if(self.current_page==CurrentPage::Statistics) onclick=self.link.callback(|_| Msg::OpenPage(CurrentPage::Statistics))>{"Statistics"}</a>
                        <a class=active_if(self.current_page==CurrentPage::Help)       onclick=self.link.callback(|_| Msg::OpenPage(CurrentPage::Help))>{"Help"}</a>
                        <a class=active_if(self.current_page==CurrentPage::About)      onclick=self.link.callback(|_| Msg::OpenPage(CurrentPage::About))>{"About"}</a>
                    </div>
                </div>
            </header>        
        };

        html! {
            <div>
                { html_top_buttons }
                { current_page }
            </div>
        }
    }
}


////////////////////////////////////////////////////////////
/// If condition is met, return "active", otherwise "". For CSS styling of which control is active
pub fn active_if(cond: bool) -> String {
    if cond {
        "active".to_string()
    } else {
        "".to_string()
    }
}




////////////////////////////////////////////////////////////
/// If condition is met, return "selected", otherwise "". For OPTION
pub fn selected_if(cond: bool) -> String {
    if cond {
        "selected".to_string()
    } else {
        "".to_string()
    }
}   // can do true false https://yew.rs/docs/concepts/html 



impl Model {

    ////////////////////////////////////////////////////////////
    /// One of the search fields
    fn view_search_line(&self, metadata: &DatabaseMetadata, i: usize) -> Html {

        let crit = self.search_settings.criteria.get(i).unwrap();
        


/*
        //meah https://yew.rs/docs/concepts/html/events
    let on_cautious_change = {
        let input_value_handle = input_value_handle.clone();

        Callback::from(move |e: Event| {
            // When events are created the target is undefined, it's only
            // when dispatched does the target get added.
            let target: Option<EventTarget> = e.target();
            // Events can bubble so this listener might catch events from child
            // elements which are not of type HtmlInputElement
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

            if let Some(input) = input {
                input_value_handle.set(input.value());
            }
        })
    };


    // check https://docs.rs/yew-components/latest/src/yew_components/select.rs.html


 */

        html! {
			<div class="divSearchField">
				<button name="bDelete" class="buttonspacer" onclick=self.link.callback(move |_| Msg::DeleteSearchFilter(i))>
                    {"X"}
                </button>
				<select class="columndrop" name="selectfield">
                    {
                        metadata.columns.clone().into_iter().map(|col| { /////////////////////////////////////////////// check why so much cloning needed
                            html!{
                                <option value={col.column_id.clone()}>  //////  selected="selected"  if the one
                                    { col.column_id.clone() }
                                </option>
                            }
                        }).collect::<Html>()
                    }
				</select>
				<label>
                    {" From: "}
					<input class="textbox" type="text" name="value" value={crit.from.clone()} 
             //       oninput=self.link.callback(move |e: web_sys::InputEvent| Msg::ChangedSearchField(i, e.value))
                    /> ////////////// if edited, should update underlying model 666
                    {" To: "}
					<input class="textbox" type="text" name="value2" value={crit.to.clone()}/>
				</label>				
			</div>
        }
        
    }



    ////////////////////////////////////////////////////////////
    /// Page: Search
    fn view_search_pane(&self) -> Html {

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


    ////////////////////////////////////////////////////////////
    /// Page: Help
    fn view_help_pane(&self) -> Html {
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



    ////////////////////////////////////////////////////////////
    /// Page: Statistics about the database
    fn view_statistics_pane(&self) -> Html {

        html! {
            <div>
            <div class="App-divider">{"Statistics"}</div>
            <div class="landingdiv">
                <p>
                    {"map num genomes per country"}
                    //pie charts below
                    {"GTDB species"}
                    {"Isolation source (Source 1)"}
                    {"Human illness"}
                    {"BTyper3 adjusted panC group"}
                </p>
                {""}
                
            </div>
            <br />
            </div>        
        }
        
    }



    ////////////////////////////////////////////////////////////
    /// x
    fn view_about_pane(&self) -> Html {

        html! {
            <div>
                <div class="App-divider">
                    {"About BTyperDB"}
                </div>
                <div class="landingdiv">
                    <h1>
                        {"What is BTyperDB?"}
                    </h1>
                    <p>
                        <a href="https://www.biorxiv.org/content/10.1101/2023.12.20.572685v1">
                            {"BTyperDB"}
                        </a>
                        {" is our community-curated, global atlas of "}
                        <a href="https://www.tandfonline.com/doi/full/10.1080/10408398.2021.1916735">
                            <i>
                                {"Bacillus cereus"}
                            </i> 
                            {" group genomes"}
                        </a>
                        {". We developed BTyperDB because we noticed that existing pathogen genome databases were inadequate–and "}
                        <a href="https://wwwnc.cdc.gov/eid/article/28/9/22-0293_article">
                            {"potentially dangerous!"}
                        </a>
                        {"–when used for "}
                        <i>
                            {"B. cereus"}
                        </i>
                        {" group surveillance."}
                    </p>
                    <p>
                        {"We decided to construct our own database by assembling THOUSANDS of novel "}
                        <i>
                            {"B. cereus"}
                        </i>
                        {"group genomes; this allowed us to nearly double the number of publicly available "}
                        <i>
                            {"B. cereus"}
                        </i>
                        {"group genomes! For version 1 (v1) of the database, our curation team spent thousands of hours manually curating and standardizing metadata for every single genome (nearly 6k genomes total)."} 
                    </p>
                    <p>
                        {"For version 2 (v2) of the database, we wanted to give "}
                        <i>
                            {"B. cereus"}
                        </i>
                        {"group (meta)data generators the opportunity to participate by validating and/or correcting their genomic metadata. To that end, we’ve recently reached out to hundreds of "}
                        <i>
                            {"B. cereus"}
                        </i>
                        {"group (meta)data generators, and dozens have already contrbuted by validating, correcting, and/or contributing novel metadata. Some even dontated novel genomes, and so far, we've received hundreds of novel, unpublished "} 
                        <i>
                            {"B. cereus"}
                        </i>
                        {"group genomes from our generous contributors!"}
                    </p>
                    <p>
                        {"So stay tuned for exciting updates in v2!"}
                    </p>
                    <h1>{"Citing BTyperDB"}</h1>
                    <p>
                        {"If you find BTyperDB useful, please cite our "}
                        <a href="https://www.biorxiv.org/content/10.1101/2023.12.20.572685v1">
                            {"preprint"}
                        </a>
                        {":"}
                    </p>
                    <p>
                        {"Ramnath, et al. 2023. A community-curated, global atlas of "}
                        <i>
                            {"Bacillus cereus sensu lato"}
                        </i> 
                        {" genomes for epidemiological surveillance. "}
                        <i>
                            {"bioRxiv"}
                        </i>
                        {" 2023.12.20.572685; doi: "}
                        <a href="https://www.biorxiv.org/content/10.1101/2023.12.20.572685v1">
                            {"https://doi.org/10.1101/2023.12.20.572685."}
                        </a>
                    </p>
                    <h1>{"License of BTyperDB"}</h1>
                    <p>
                        {"All BTyperDB metadata is available under the "}
                        <a href="https://en.wikipedia.org/wiki/CC0">
                            {"CC0 license "}
                        </a>
                        {", which is as close to public domain as material can get. This means that you can do almost anything you want with the metadata. The "}                    
                        <a href="https://github.com/henriksson-lab/btyper_website/tree/main">
                            {"code for the website"}
                        </a>                    
                        {" will be under a yet-to-be-decided open source license."}
                    </p>
                    <h1>{"The BTyperDB team"}</h1>
                    <p>
                        <a href="https://www.biorxiv.org/content/10.1101/2023.12.20.572685v1">
                            {"BTyperDB is a community-driven project, with contributors and curators from around the world."}
                        </a>
                        
                        {" The project is led by "}
                        <a href="https://www.microbe.dev/">{"Laura Carroll"}</a>
                        {" and "}
                        <a href="http://www.henlab.org">{"Johan Henriksson"}</a>
                        {"."}
                    </p>
                </div>
                <br />
            </div>
        }        
    }



    ////////////////////////////////////////////////////////////
    /// x
    fn view_landing_page(&self) -> Html {

        let num_strain = if let Some(metadata) = &self.db_metadata {
            format!("{}", metadata.num_strain)
        } else {
            "___".to_string()
        };

        html! {

            <div class="landingdiv">

                <img src="assets/Btyperdb_logo.svg" alt="rust image"/> 
                <p style="color: rgb(0, 150, 255);">
                    {"A community curated, global atlas of Bacillus cereus group genomes"}
                </p>

                <p style="color: rgb(0, 150, 255);">
                    {"Database version v1"}
                </p>

                <p style="color: rgb(0, 150, 255);">
                    {num_strain} {" total B. cereus group genomes with curated metadata"}
                </p>

                <button class="toolbutton" onclick=self.link.callback(|_| Msg::OpenPage(CurrentPage::Search))>
                    {"Search BTyperDB"}
                </button>

                <button class="toolbutton" onclick=self.link.callback(|_| Msg::FetchDatabaseMetadata)>
                    {"debug BTyperDB"}
                </button>


            </div>
        }
    }



    ////////////////////////////////////////////////////////////
    /// x
    fn view_table_row(&self, row: &Vec<String>) -> Html {

        //pub column_names: Vec<String>,
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
    fn view_table(&self) -> Html {

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

                let from_row = 0;
                let mut to_row = from_row + entries_per_page;
                if to_row > dt.rows.len() {
                    to_row = dt.rows.len();
                }

                let show_rows = from_row..to_row;

                log::debug!("showrows {:?}", show_rows);

                //// Generate all pages
                let possible_pages = 0..(1+(dt.rows.len()/entries_per_page));
                let div_gotopage = html! {
                    <div>
                        {"Show page: "}
                        {
                            possible_pages.into_iter().map(|p| 
                                html!{ format!("{} ",p+1)}
                            ).collect::<Html>()
                        }
                    </div>
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


fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}
