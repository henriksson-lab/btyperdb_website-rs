use std::collections::HashSet;
use std::io::Cursor;

use my_web_app::ComparisonType;
use my_web_app::DatabaseMetadata;
use my_web_app::StrainRequest;
use my_web_app::TableData;
use my_web_app::SearchSettings;
use my_web_app::SearchCriteria;

use geojson::GeoJson;


use web_sys::window;
use yew::prelude::*;

////////////////////////////////////////////////////////////
/// Which page is currently being shown?
#[derive(Debug)]
#[derive(PartialEq)]
pub enum CurrentPage {
    Home,
    Search,
    Statistics,
    Help,
    About,
}


////////////////////////////////////////////////////////////
/// 
#[derive(Debug)]
pub enum IncludeData {
    All,
    Selected
}

////////////////////////////////////////////////////////////
/// Message sent to the event system for updating the page
#[derive(Debug)]
pub enum Msg {

    OpenPage(CurrentPage),
    StartQuery,
    SetQuery(Option<TableData>),
    SetSearchControlVisibility(bool),
    AddSearchFilter,
    DeleteSearchFilter(usize),
    SetDatabaseMetadata(DatabaseMetadata),
    FetchDatabaseMetadata,

    ChangedSearchFieldType(usize, String),
    ChangedSearchFieldFrom(usize, String),
    ChangedSearchFieldTo(usize, String),
    ChangedSearchFieldLike(usize, String),

    SetTableFrom(usize),
    DownloadFASTA(IncludeData),
    DownloadMetadata(IncludeData),
    DownloadFASTAgot(Vec<u8>),

    SetStrainSelected(String, bool),

    HideColumn(String),
    ShowColumn(String),
}



////////////////////////////////////////////////////////////
/// State of the page
pub struct Model {
    pub current_page: CurrentPage,
    pub tabledata: Option<TableData>,
    pub tabledata_from: usize,
        
    pub show_search_controls: bool,
    pub search_settings: SearchSettings,
    pub db_metadata: Option<DatabaseMetadata>,

    pub geojson: GeoJson,

    pub selected_strains: HashSet<String>,

    pub show_columns: HashSet<String>,
}

impl Component for Model {
    type Message = Msg;

    type Properties = ();


    ////////////////////////////////////////////////////////////
    /// Create a new component
    fn create(ctx: &Context<Self>) -> Self {

        let geojson = GeoJson::from_reader(Cursor::new(include_bytes!("custom.geo.json"))).unwrap();

        // For testing
        //let tabledata:TableData = serde_json::from_reader(Cursor::new(include_bytes!("testdata.json"))).unwrap();

        //Get metadata about database right away
        //ex from https://github.com/yewstack/yew/blob/master/examples/async_clock/src/main.rs
        ctx.link().send_message(Msg::FetchDatabaseMetadata);

        Self {
            current_page: CurrentPage::Home,
            tabledata: None, 
            tabledata_from: 0,
            
            show_search_controls: true,
            search_settings: SearchSettings::new(),
            db_metadata: None,
            geojson: geojson,

            selected_strains: HashSet::new(),

            show_columns: HashSet::new(),

          //  country: "asdasd".to_string()
        }
    }




    ////////////////////////////////////////////////////////////
    /// Handle an update message
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {

            ////////////////////////////////////////////////////////////
            // x
            Msg::OpenPage(page) => {
                self.current_page = page;
                true
            }


            ////////////////////////////////////////////////////////////
            // x
            Msg::StartQuery => {
                let json = serde_json::to_string(&self.search_settings).expect("Failed to generate json");
                //log::debug!("sending {}", json);
                async fn get_data(json: String) -> Msg {
                    let client = reqwest::Client::new();
                    let res: TableData = client.post(format!("{}/straindata",get_host_url()))
                        .header("Content-Type", "application/json")
                        .body(json)
                        .send()
                        .await
                        .expect("Failed to send request")
                        .json()
                        .await
                        .expect("Failed to get table data");
                    Msg::SetQuery(Some(res))
                }

                ctx.link().send_future(get_data(json));
                false
            }





            ////////////////////////////////////////////////////////////
            // x
            Msg::FetchDatabaseMetadata => {
                async fn get_data() -> Msg {
                    let client = reqwest::Client::new();
                    let url=format!("{}/strainmeta",get_host_url());
                    //log::debug!("wtf -{}-",url);
                    let res: DatabaseMetadata = client.get(url)  
                        .header("Content-Type", "application/json")
                        .body("")
                        //no body
                        .send()
                        .await
                        .expect("Failed to send request")
                        .json()
                        .await
                        .expect("Failed to get metadata");
                    Msg::SetDatabaseMetadata(res)
                }

                ctx.link().send_future(get_data());
                false
            }



            ////////////////////////////////////////////////////////////
            // x
            Msg::SetQuery(data) => {
                //log::trace!("SetQuery: {:?}", data);
                self.tabledata = data;
                self.tabledata_from = 0;
                self.selected_strains.clear();
                true
            }



            ////////////////////////////////////////////////////////////
            // x
            Msg::SetDatabaseMetadata(data) => {

                //Set columns to show
                self.show_columns.clear();
                for (colname, colmeta) in &data.columns {
                    if colmeta.default_show_column=="1" {
                        self.show_columns.insert(colname.clone());
                    }
                }

                //log::trace!("SetDatabaseMetadata: {:?}", data);
                self.db_metadata = Some(data);

                //TODO: populate search box

                true
            }


            ////////////////////////////////////////////////////////////
            // x
            Msg::SetSearchControlVisibility(data) => {
                //log::trace!("SetSearchControlVisibility: {:?}", data);
                self.show_search_controls = data;
                true
            }

            ////////////////////////////////////////////////////////////
            // x
            Msg::AddSearchFilter => {
                if let Some(metadata) = &self.db_metadata {

                    let col = metadata.columns.get("BTyperDB_ID").expect("no BTyperDB_ID column");

                    //let all_columns: Vec<String> = metadata.columns.iter().map(|x| x.column_id.clone()).collect();
                    //let default_element = all_columns.get(0).expect("empty list");

                    //log::trace!("AddSearchFilter: {:?}", data);
                    let mut c = SearchCriteria::new();
                    c.field=col.column_id.clone();
                    c.comparison = ComparisonType::default_comparison(&col);
                    self.search_settings.criteria.push(c);
                }
                true
            },

            ////////////////////////////////////////////////////////////
            // x
            Msg::DeleteSearchFilter(i) => {
                //log::trace!("DeleteSearchFilter: {:?}", data);
                self.search_settings.criteria.remove(i);
                true
            }


            ////////////////////////////////////////////////////////////
            // x
            Msg::ChangedSearchFieldType(i, val) => {
                let crit = self.search_settings.criteria.get_mut(i).expect("Could not get field");
                crit.field = val;

                if let Some(db_metadata) = &self.db_metadata {
                    let column_metadata = db_metadata.columns.get(&crit.field).expect("no column");
                    crit.comparison = ComparisonType::default_comparison(column_metadata);
                    log::debug!("{:?}",crit.comparison);
                } else {
                    log::debug!("Missing db metadata");
                }
                true
            }

            ////////////////////////////////////////////////////////////
            // x
            Msg::ChangedSearchFieldFrom(i, val) => {
                let field = self.search_settings.criteria.get_mut(i).expect("Could not get field");
                if let ComparisonType::FromTo(from,_to) = &mut field.comparison {
                    *from = val;
                }
                //log::debug!("got f {:?}", field);
                false
            }
            
            ////////////////////////////////////////////////////////////
            // x
            Msg::ChangedSearchFieldTo(i, val) => {
                let field = self.search_settings.criteria.get_mut(i).expect("Could not get field");
                if let ComparisonType::FromTo(_from,to) = &mut field.comparison {
                    *to = val;
                }
                //log::debug!("got f {:?}", field);
                false
            }

            ////////////////////////////////////////////////////////////
            // x
            Msg::ChangedSearchFieldLike(i, val) => {
                let field = self.search_settings.criteria.get_mut(i).expect("Could not get field");
                if let ComparisonType::Like(v) = &mut field.comparison {
                    *v = val;
                }
                false
            }



            ////////////////////////////////////////////////////////////
            // x
            Msg::SetTableFrom(from) => {
                self.tabledata_from = from;
                true
            },

            ////////////////////////////////////////////////////////////
            // x
            Msg::DownloadFASTAgot(data) => {
                log::debug!("DownloadFASTAgot");
                self.download_fasta(&data);
                false
            },


            ////////////////////////////////////////////////////////////
            // x
            Msg::DownloadFASTA(inc) => {
                log::debug!("trying to download");

                let list_strains = self.get_strains(&inc);
                log::debug!("Asking to download {:?}", list_strains);

                if list_strains.is_empty() {
                    alert("No strains to download");
                } else {
                    let req = StrainRequest {
                        list: list_strains
                    };

                    let json = serde_json::to_string(&req).expect("Failed to generate json");
                    //log::debug!("sending {}", json);

                    //log::debug!("sending {}", json);
                    async fn get_data(json: String) -> Msg {
                        let client = reqwest::Client::new();
                        let res = client.post(format!("{}/strainfasta",get_host_url()))
                            .header("Content-Type", "application/json")
                            .body(json)
                            .send()
                            .await
                            .expect("Failed to send request")
                            .bytes()
//                            .json()
                            .await
                            .expect("Failed to get table data");

                        Msg::DownloadFASTAgot(res.to_vec())
                    }
                    ctx.link().send_future(get_data(json));
                }        
                false        
            },


            ////////////////////////////////////////////////////////////
            // x
            Msg::DownloadMetadata(inc) => {
                log::debug!("trying to download");

                let list_strains = self.get_strains(&inc);
                log::debug!("Asking to download {:?}", list_strains);

                if list_strains.is_empty() {
                    alert("No strains to download");
                } else {
                    self.download_metadata(&list_strains);
                }
                false                
            },


            ////////////////////////////////////////////////////////////
            // x
            Msg::SetStrainSelected(id, tosel) => {
                if tosel {
                    self.selected_strains.insert(id);
                } else {
                    self.selected_strains.remove(&id);
                }
                false
            },
            
            ////////////////////////////////////////////////////////////
            // Hide a column specified by name
            Msg::HideColumn(col) => {
                self.show_columns.retain(|s| s != &col);
                true
            },


            ////////////////////////////////////////////////////////////
            // Show a column specified by name
            Msg::ShowColumn(col) => {
                if col != "" {
                    self.show_columns.insert(col);
                }
                true
            },            
        }
    }



    ////////////////////////////////////////////////////////////
    /// Top renderer of the page
    fn view(&self, ctx: &Context<Self>) -> Html {

        let current_page = match self.current_page { 
            CurrentPage::Home => self.view_landing_page(&ctx),
            CurrentPage::Search => self.view_search_pane(&ctx),
            CurrentPage::Statistics => self.view_statistics_pane(&ctx),
            CurrentPage::Help => self.view_help_pane(&ctx),
            CurrentPage::About => self.view_about_pane(&ctx)
        };

        let html_top_buttons = html! {
            <header class="App-header">
                <div id="topmenu" class="topnav">
                    <div class="topnav-right">
                        <a class={active_if(self.current_page==CurrentPage::Home)}       onclick={ctx.link().callback(|_| Msg::OpenPage(CurrentPage::Home))}>{"Home"}</a> 
                        <a class={active_if(self.current_page==CurrentPage::Search)}     onclick={ctx.link().callback(|_| Msg::OpenPage(CurrentPage::Search))}>{"Search"}</a>
                        <a class={active_if(self.current_page==CurrentPage::Statistics)} onclick={ctx.link().callback(|_| Msg::OpenPage(CurrentPage::Statistics))}>{"Statistics"}</a>
                        <a class={active_if(self.current_page==CurrentPage::Help)}       onclick={ctx.link().callback(|_| Msg::OpenPage(CurrentPage::Help))}>{"Help"}</a>
                        <a class={active_if(self.current_page==CurrentPage::About)}      onclick={ctx.link().callback(|_| Msg::OpenPage(CurrentPage::About))}>{"About"}</a>
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
/// Show an alert message
pub fn alert(s: &str) {
    let window = window().expect("no window");
    window.alert_with_message(s).unwrap();
}


pub fn get_host_url() -> String {
    let document = window().expect("no window").document().expect("no document on window");
    let location = document.location().expect("no location");
    let protocol = location.protocol().expect("no protocol");
    let host = location.host().expect("no host");

    let url = format!("{}//{}", protocol, host);
    //log::debug!("{}",url);
    url
}

// https://yew.rs/docs/next/advanced-topics/struct-components/hoc