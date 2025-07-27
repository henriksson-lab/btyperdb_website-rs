use std::io::Cursor;

use anyhow::Result;

use my_web_app::ComparisonType;
use my_web_app::DatabaseMetadata;
use my_web_app::TableData;
use my_web_app::SearchSettings;
use my_web_app::SearchCriteria;

use geojson::GeoJson;


use yew::{
    format::{Json, Nothing},
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
pub enum CurrentPage {
    Home,
    Search,
    Statistics,
    Help,
    About,
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

    ChangedSearchFieldType(usize, ChangeData),
    ChangedSearchFieldFrom(usize, String),
    ChangedSearchFieldTo(usize, String),
    ChangedSearchFieldLike(usize, String),

    SetTableFrom(usize),
}

////////////////////////////////////////////////////////////
/// State of the page
pub struct Model {
    pub link: ComponentLink<Self>,
    pub current_page: CurrentPage,
    pub tabledata: Option<TableData>,
    pub tabledata_from: usize,
    pub task: Option<FetchTask>, ///////////// why do we keep this?
    pub show_search_controls: bool,
    pub search_settings: SearchSettings,
    pub db_metadata: Option<DatabaseMetadata>,

    pub geojson: GeoJson,
}

impl Component for Model {
    type Message = Msg;

    type Properties = ();

    ////////////////////////////////////////////////////////////
    /// Create a new component
    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {



        let geojson = GeoJson::from_reader(Cursor::new(include_bytes!("custom.geo.json"))).unwrap();

        //https://docs.rs/geojson/latest/geojson/

        


        //let geojson = geojson_str.parse::<GeoJson>().unwrap();


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
            db_metadata: None,
            geojson: geojson,
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
                let json = serde_json::to_string(&self.search_settings).expect("Failed to generate json");

                //log::debug!("sending {}", json);

                //let request = Request::get("/straindata") /////////// do post instead
                let request = Request::post("/straindata")
                    .header("Content-Type", "application/json")
//                    .body(Json(&json))  //do not use!! escapes the data resulting in error 400
                    .body(Ok(json))
                    .expect("Could not build request");
                let callback =
                    self.link.callback(|response: Response<Json<Result<TableData>>>| {
                        //log::debug!("{:?}", response);
                        let Json(data) = response.into_body();
                        Msg::SetQuery(data.ok())
                    });
                let task = FetchService::fetch(
                    request, 
                    callback).expect("Failed to start request");
                //store the task so it isn't canceled immediately
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

            Msg::DeleteSearchFilter(i) => {
                //log::trace!("DeleteSearchFilter: {:?}", data);
                self.search_settings.criteria.remove(i);
                true
            }


            Msg::ChangedSearchFieldType(i, val) => {
                if let ChangeData::Select(d) = val {
                    let val = d.value();

                    let crit = self.search_settings.criteria.get_mut(i).expect("Could not get field");
                    crit.field = val;
                    //log::debug!("set field {:?}", field);

                    if let Some(db_metadata) = &self.db_metadata {
                        let column_metadata = db_metadata.columns.get(&crit.field).expect("no column");
                        //log::debug!("field info {:?} {:?}", crit.field, column_metadata);                        
                        crit.comparison = ComparisonType::default_comparison(column_metadata);
                        //log::debug!("ChangedSearchFieldType: {:?}", crit);
                    }


                    //todo update from-to etc
                    //could be beneficial to do this in subcomponent; would it avoid rerendering everything?
                }
                true
            }

            Msg::ChangedSearchFieldFrom(i, val) => {
                let field = self.search_settings.criteria.get_mut(i).expect("Could not get field");
                if let ComparisonType::FromTo(from,_to) = &mut field.comparison {
                    *from = val;
                }
                //log::debug!("got f {:?}", field);
                false
            }
            
            Msg::ChangedSearchFieldTo(i, val) => {
                let field = self.search_settings.criteria.get_mut(i).expect("Could not get field");
                if let ComparisonType::FromTo(_from,to) = &mut field.comparison {
                    *to = val;
                }
                //log::debug!("got f {:?}", field);
                false
            }

            Msg::ChangedSearchFieldLike(i, val) => {
                let field = self.search_settings.criteria.get_mut(i).expect("Could not get field");
                if let ComparisonType::Like(v) = &mut field.comparison {
                    *v = val;
                }
//                field.to = val;
                //log::debug!("got f {:?}", field);
                false
            }



            Msg::SetTableFrom(from) => {
                self.tabledata_from = from;
                true
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





