use std::collections::HashSet;
use std::io::Cursor;

use my_web_app::ComparisonType;
use my_web_app::DatabaseMetadata;
use my_web_app::StrainRequest;
use my_web_app::TableData;
use my_web_app::SearchSettings;
use my_web_app::SearchCriteria;

use geojson::GeoJson;


use my_web_app::TreeData;
use web_sys::window;
use yew::prelude::*;

use crate::appstate::AsyncData;
use crate::resize::ComponentSize;
use crate::resize::ComponentSizeObserver;
use crate::treeview::treelayout::TreeLayout;

////////////////////////////////////////////////////////////
/// Which page is currently being shown?
#[derive(Debug)]
#[derive(PartialEq)]
pub enum CurrentPage {
    Home,
    Search,
    Tree,
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
pub enum MsgCore {

    WindowResize(ComponentSize),

    OpenPage(CurrentPage),
    StartQuery,
    SetQuery(Option<TableData>),
    SetSearchControlVisibility(bool),
    AddSearchFilter,
    DeleteSearchFilter(usize),

    FetchDatabaseMetadata,
    SetDatabaseMetadata(DatabaseMetadata),

    FetchTreeData,
    SetTreeData(TreeLayout),

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

    pub last_component_size: ComponentSize,

    pub treedata: AsyncData<TreeLayout>,
}

impl Component for Model {
    type Message = MsgCore;

    type Properties = ();


    ////////////////////////////////////////////////////////////
    /// Create a new component
    fn create(ctx: &Context<Self>) -> Self {

        let geojson = GeoJson::from_reader(Cursor::new(include_bytes!("custom.geo.json"))).unwrap();

        //Get metadata about database right away
        ctx.link().send_message(MsgCore::FetchDatabaseMetadata);

        //Get tree right away (or wait until tab open?)
        ctx.link().send_message(MsgCore::FetchTreeData);

//        let treedata = AsyncData::new(TreeLayout::new());
        let treedata = AsyncData::NotLoaded;
        
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

            last_component_size: ComponentSize { width: 100.0, height: 100.0 },
            
            treedata: treedata,

        }
    }




    ////////////////////////////////////////////////////////////
    /// Handle an update message
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {

            ////////////////////////////////////////////////////////////
            // Message: Window is resized
            MsgCore::WindowResize(size) => {
                log::debug!("window resize");
                self.last_component_size = size;
                true
            },

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::OpenPage(page) => {
                self.current_page = page;
                true
            }


            ////////////////////////////////////////////////////////////
            // x
            MsgCore::StartQuery => {
                let json = serde_json::to_string(&self.search_settings).expect("Failed to generate json");
                //log::debug!("sending {}", json);
                async fn get_data(json: String) -> MsgCore {
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
                    MsgCore::SetQuery(Some(res))
                }

                ctx.link().send_future(get_data(json));
                false
            }





            ////////////////////////////////////////////////////////////
            // x
            MsgCore::FetchDatabaseMetadata => {
                async fn get_data() -> MsgCore {
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
                    MsgCore::SetDatabaseMetadata(res)
                }

                ctx.link().send_future(get_data());
                false
            }


            ////////////////////////////////////////////////////////////
            // x
            MsgCore::FetchTreeData => {
                async fn get_data() -> MsgCore {
                    let client = reqwest::Client::new();
                    let url=format!("{}/treedata",get_host_url());
                    //log::debug!("wtf -{}-",url);
                    log::debug!("getting tree");
                    let res: TreeData = client.get(url)  
                        .header("Content-Type", "application/json")
                        .body("")
                        //no body
                        .send()
                        .await
                        .expect("Failed to send request")
                        .json()
                        .await
                        .expect("Failed to get treedata");
                    log::debug!("making layout");
                    let lay = TreeLayout::new(&res.tree_str);
                    log::debug!("setting layout");
                    MsgCore::SetTreeData(lay)
                }

                ctx.link().send_future(get_data());
                false
            }




            ////////////////////////////////////////////////////////////
            // x
            MsgCore::SetTreeData(lay) => {

                //log::trace!("SetDatabaseMetadata: {:?}", data);
                //let lay = TreeLayout::new(&data.tree_str);
                self.treedata = AsyncData::new(lay);

                true
            }


            ////////////////////////////////////////////////////////////
            // x
            MsgCore::SetQuery(data) => {
                //log::trace!("SetQuery: {:?}", data);
                self.tabledata = data;
                self.tabledata_from = 0;
                self.selected_strains.clear();
                true
            }



            ////////////////////////////////////////////////////////////
            // x
            MsgCore::SetDatabaseMetadata(data) => {

                //Set columns to show
                self.show_columns.clear();
                for (colname, colmeta) in &data.columns {
                    if colmeta.default_show_column=="1" {
                        self.show_columns.insert(colname.clone());
                    }
                }

                //Populate search box
                self.search_settings = data.make_default_search();

                //log::trace!("SetDatabaseMetadata: {:?}", data);
                self.db_metadata = Some(data);

                true
            }


            ////////////////////////////////////////////////////////////
            // x
            MsgCore::SetSearchControlVisibility(data) => {
                //log::trace!("SetSearchControlVisibility: {:?}", data);
                self.show_search_controls = data;
                true
            }

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::AddSearchFilter => {
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
            MsgCore::DeleteSearchFilter(i) => {
                //log::trace!("DeleteSearchFilter: {:?}", data);
                self.search_settings.criteria.remove(i);
                true
            }


            ////////////////////////////////////////////////////////////
            // x
            MsgCore::ChangedSearchFieldType(i, val) => {
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
            MsgCore::ChangedSearchFieldFrom(i, val) => {
                let field = self.search_settings.criteria.get_mut(i).expect("Could not get field");
                if let ComparisonType::FromTo(from,_to) = &mut field.comparison {
                    *from = val;
                }
                //log::debug!("got f {:?}", field);
                false
            }
            
            ////////////////////////////////////////////////////////////
            // x
            MsgCore::ChangedSearchFieldTo(i, val) => {
                let field = self.search_settings.criteria.get_mut(i).expect("Could not get field");
                if let ComparisonType::FromTo(_from,to) = &mut field.comparison {
                    *to = val;
                }
                //log::debug!("got f {:?}", field);
                false
            }

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::ChangedSearchFieldLike(i, val) => {
                let field = self.search_settings.criteria.get_mut(i).expect("Could not get field");
                if let ComparisonType::Like(v) = &mut field.comparison {
                    *v = val;
                }
                false
            }



            ////////////////////////////////////////////////////////////
            // x
            MsgCore::SetTableFrom(from) => {
                self.tabledata_from = from;
                true
            },

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::DownloadFASTAgot(data) => {
                log::debug!("DownloadFASTAgot");
                self.download_fasta(&data);
                false
            },


            ////////////////////////////////////////////////////////////
            // x
            MsgCore::DownloadFASTA(inc) => {
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
                    async fn get_data(json: String) -> MsgCore {
                        let client = reqwest::Client::new();
                        let res = client.post(format!("{}/strainfasta",get_host_url()))
                            .header("Content-Type", "application/json")
                            .body(json)
                            .send()
                            .await
                            .expect("Failed to send request")
                            .bytes()
                            .await
                            .expect("Failed to get table data");

                        MsgCore::DownloadFASTAgot(res.to_vec())
                    }
                    ctx.link().send_future(get_data(json));
                }        
                false        
            },


            ////////////////////////////////////////////////////////////
            // x
            MsgCore::DownloadMetadata(inc) => {
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
            MsgCore::SetStrainSelected(id, tosel) => {
                if tosel {
                    self.selected_strains.insert(id);
                } else {
                    self.selected_strains.remove(&id);
                }
                false
            },
            
            ////////////////////////////////////////////////////////////
            // Hide a column specified by name
            MsgCore::HideColumn(col) => {
                self.show_columns.retain(|s| s != &col);
                true
            },


            ////////////////////////////////////////////////////////////
            // Show a column specified by name
            MsgCore::ShowColumn(col) => {
                if col != "" {
                    //log::debug!("Adding new column to show {}", col);
                    self.show_columns.insert(col);
                    //log::debug!("now cols: {:?}", self.show_columns);
                }
                true
            },            
        }
    }



    ////////////////////////////////////////////////////////////
    /// Top renderer of the page
    fn view(&self, ctx: &Context<Self>) -> Html {

        let onsize = ctx.link().callback(|size: ComponentSize| {
            MsgCore::WindowResize(size)
        });

        let current_page = match self.current_page { 
            CurrentPage::Home => self.view_landing_page(&ctx),
            CurrentPage::Search => self.view_search_pane(&ctx),
            CurrentPage::Tree => self.view_tree_pane(&ctx),
            CurrentPage::Statistics => self.view_statistics_pane(&ctx),
            CurrentPage::Help => self.view_help_pane(&ctx),
            CurrentPage::About => self.view_about_pane(&ctx)
        };

        let html_top_buttons = html! {

            <div style="position: relative;"> // added; does this mess anything up?
                <ComponentSizeObserver onsize={onsize} />

                <header class="App-header">
                    <div id="topmenu" class="topnav">
                        <div class="topnav-right">
                            <a class={active_if(self.current_page==CurrentPage::Home)}       onclick={ctx.link().callback(|_| MsgCore::OpenPage(CurrentPage::Home))}>{"Home"}</a> 
                            <a class={active_if(self.current_page==CurrentPage::Search)}     onclick={ctx.link().callback(|_| MsgCore::OpenPage(CurrentPage::Search))}>{"Search"}</a>
                            <a class={active_if(self.current_page==CurrentPage::Tree)}       onclick={ctx.link().callback(|_| MsgCore::OpenPage(CurrentPage::Tree))}>{"Tree"}</a>
                            <a class={active_if(self.current_page==CurrentPage::Statistics)} onclick={ctx.link().callback(|_| MsgCore::OpenPage(CurrentPage::Statistics))}>{"Statistics"}</a>
                            <a class={active_if(self.current_page==CurrentPage::Help)}       onclick={ctx.link().callback(|_| MsgCore::OpenPage(CurrentPage::Help))}>{"Help"}</a>
                            <a class={active_if(self.current_page==CurrentPage::About)}      onclick={ctx.link().callback(|_| MsgCore::OpenPage(CurrentPage::About))}>{"About"}</a>
                        </div>
                    </div>
                </header>      

            </div>  
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

////////////////////////////////////////////////////////////
/// 
pub fn get_host_url() -> String {
    let document = window().expect("no window").document().expect("no document on window");
    let location = document.location().expect("no location");
    let protocol = location.protocol().expect("no protocol");
    let host = location.host().expect("no host");

    let url = format!("{}//{}", protocol, host);
    //log::debug!("{}",url);
    url
}

