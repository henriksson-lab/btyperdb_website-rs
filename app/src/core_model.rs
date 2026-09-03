use std::collections::HashSet;
use std::io::Cursor;

use my_web_app::ComparisonType;
use my_web_app::DatabaseMetadata;
use my_web_app::SearchCriteria;
use my_web_app::SearchSettings;
use my_web_app::StrainRequest;
use my_web_app::TableData;

use geojson::GeoJson;

use my_web_app::TreeData;
use web_sys::window;
use yew::prelude::*;

use crate::appstate::AsyncData;
use crate::resize::ComponentSize;
use crate::resize::ComponentSizeObserver;
use crate::treeview::treelayout::TreeLayout;

////////////////////////////////////////////////////////////
/// Send a request and read the response as json, describing any failure
/// instead of panicking on it.
///
/// The backend answers an invalid search with a plain-text 400 body, so
/// calling .json() on whatever came back used to take down the whole app --
/// clearing a numeric search box was enough to do it. Errors reach the user as
/// an AsyncData::Failed message now.
async fn fetch_json<T: serde::de::DeserializeOwned>(
    req: reqwest::RequestBuilder,
    what: &str,
) -> Result<T, String> {
    let res = req
        .send()
        .await
        .map_err(|e| format!("Could not reach the server while loading {what}. ({e})"))?;

    let status = res.status();
    if !status.is_success() {
        // An error response carries a plain-text explanation, not json
        let body = res.text().await.unwrap_or_default();
        let body = body.trim();
        return if body.is_empty() {
            Err(format!("The server rejected the request for {what} ({status})."))
        } else {
            Err(format!("The server rejected the request for {what}: {body}"))
        };
    }

    res.json::<T>()
        .await
        .map_err(|e| format!("Could not read the {what} sent back by the server. ({e})"))
}

////////////////////////////////////////////////////////////
/// Which page is currently being shown?
#[derive(Debug, PartialEq)]
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
    Selected,
}

////////////////////////////////////////////////////////////
/// Message sent to the event system for updating the page
#[derive(Debug)]
pub enum MsgCore {
    WindowResize(ComponentSize),

    OpenPage(CurrentPage),
    StartQuery,
    SetQuery(AsyncData<TableData>),
    SetSearchControlVisibility(bool),
    AddSearchFilter,
    DeleteSearchFilter(usize),

    FetchDatabaseMetadata,
    SetDatabaseMetadata(Result<DatabaseMetadata, String>),

    FetchTreeData,
    SetTreeData(AsyncData<TreeLayout>),

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

    OpenBTracker,
}

////////////////////////////////////////////////////////////
/// State of the page
pub struct Model {
    pub current_page: CurrentPage,
    pub tabledata: AsyncData<TableData>,
    pub tabledata_from: usize,

    pub show_search_controls: bool,
    pub search_settings: SearchSettings,
    pub db_metadata: Option<DatabaseMetadata>,
    /// Set when /strainmeta could not be loaded. Nothing else works without
    /// it, so the message is shown in place of the page content.
    pub metadata_error: Option<String>,

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
        //ctx.link().send_message(MsgCore::FetchTreeData);

        Self {
            current_page: CurrentPage::Home,
            tabledata: AsyncData::NotLoaded,
            tabledata_from: 0,

            show_search_controls: true,
            search_settings: SearchSettings::new(),
            db_metadata: None,
            metadata_error: None,
            geojson: geojson,

            selected_strains: HashSet::new(),

            show_columns: HashSet::new(),

            last_component_size: ComponentSize {
                width: 100.0,
                height: 100.0,
            },

            treedata: AsyncData::NotLoaded,
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
            }

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::OpenPage(page) => {
                self.current_page = page;
                true
            }

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::StartQuery => {
                //Set "loading" placeholder
                ctx.link()
                    .send_message(MsgCore::SetQuery(AsyncData::Loading));

                //Start query
                let json =
                    serde_json::to_string(&self.search_settings).expect("Failed to generate json");
                //log::debug!("sending {}", json);
                async fn get_data(json: String) -> MsgCore {
                    let client = reqwest::Client::new();
                    let req = client
                        .post(format!("{}/straindata", get_host_url()))
                        .header("Content-Type", "application/json")
                        .body(json);
                    match fetch_json::<TableData>(req, "the search results").await {
                        Ok(res) => MsgCore::SetQuery(AsyncData::new(res)),
                        Err(msg) => MsgCore::SetQuery(AsyncData::failed(msg)),
                    }
                }

                ctx.link().send_future(get_data(json));
                false
            }

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::FetchDatabaseMetadata => {
                async fn get_data() -> MsgCore {
                    let client = reqwest::Client::new();
                    let url = format!("{}/strainmeta", get_host_url());
                    let req = client.get(url).header("Content-Type", "application/json");
                    MsgCore::SetDatabaseMetadata(
                        fetch_json::<DatabaseMetadata>(req, "the database description").await,
                    )
                }

                ctx.link().send_future(get_data());
                false
            }

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::FetchTreeData => {
                async fn get_data() -> MsgCore {
                    let client = reqwest::Client::new();
                    let url = format!("{}/treedata", get_host_url());
                    log::debug!("getting tree");
                    let req = client.get(url).header("Content-Type", "application/json");
                    match fetch_json::<TreeData>(req, "the phylogenetic tree").await {
                        Ok(res) => {
                            log::debug!("making layout");
                            MsgCore::SetTreeData(AsyncData::new(TreeLayout::new(&res.tree_str)))
                        }
                        Err(msg) => MsgCore::SetTreeData(AsyncData::failed(msg)),
                    }
                }

                //Mark as in-flight before starting, or every re-render that
                //happens while the tree downloads starts another download
                self.treedata = AsyncData::Loading;
                ctx.link().send_future(get_data());
                true
            }

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::SetTreeData(lay) => {
                self.treedata = lay;

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
                let data = match data {
                    Ok(data) => data,
                    Err(msg) => {
                        //Without the metadata there is no search UI to show, so
                        //this one has to be visible on the page itself
                        log::error!("{}", msg);
                        self.metadata_error = Some(msg);
                        return true;
                    }
                };
                self.metadata_error = None;

                //Set columns to show
                self.show_columns.clear();
                for (colname, colmeta) in &data.columns {
                    if colmeta.show_by_default {
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
                    let col = metadata
                        .columns
                        .get("BTyperDB_ID")
                        .expect("no BTyperDB_ID column");

                    //let all_columns: Vec<String> = metadata.columns.iter().map(|x| x.column_id.clone()).collect();
                    //let default_element = all_columns.get(0).expect("empty list");

                    //log::trace!("AddSearchFilter: {:?}", data);
                    let mut c = SearchCriteria::new();
                    c.field = col.column_id.clone();
                    c.comparison = ComparisonType::default_comparison(&col);
                    self.search_settings.criteria.push(c);
                }
                true
            }

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
                let crit = self
                    .search_settings
                    .criteria
                    .get_mut(i)
                    .expect("Could not get field");
                crit.field = val;

                if let Some(db_metadata) = &self.db_metadata {
                    let column_metadata = db_metadata.columns.get(&crit.field).expect("no column");
                    crit.comparison = ComparisonType::default_comparison(column_metadata);
                    log::debug!("{:?}", crit.comparison);
                } else {
                    log::debug!("Missing db metadata");
                }
                true
            }

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::ChangedSearchFieldFrom(i, val) => {
                let field = self
                    .search_settings
                    .criteria
                    .get_mut(i)
                    .expect("Could not get field");
                if let ComparisonType::FromTo(from, _to) = &mut field.comparison {
                    *from = val;
                }
                //log::debug!("got f {:?}", field);
                false
            }

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::ChangedSearchFieldTo(i, val) => {
                let field = self
                    .search_settings
                    .criteria
                    .get_mut(i)
                    .expect("Could not get field");
                if let ComparisonType::FromTo(_from, to) = &mut field.comparison {
                    *to = val;
                }
                //log::debug!("got f {:?}", field);
                false
            }

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::ChangedSearchFieldLike(i, val) => {
                let field = self
                    .search_settings
                    .criteria
                    .get_mut(i)
                    .expect("Could not get field");
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
            }

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::DownloadFASTAgot(data) => {
                log::debug!("DownloadFASTAgot");
                if data.is_empty() {
                    alert("The sequences could not be downloaded. Please try again, or with fewer strains selected.");
                } else {
                    self.download_fasta(&data);
                }
                false
            }

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::DownloadFASTA(inc) => {
                log::debug!("trying to download");

                let list_strains = self.get_strains(&inc);
                log::debug!("Asking to download {:?}", list_strains);

                if list_strains.is_empty() {
                    alert("No strains to download");
                } else {
                    let req = StrainRequest { list: list_strains };

                    let json = serde_json::to_string(&req).expect("Failed to generate json");
                    //log::debug!("sending {}", json);

                    //log::debug!("sending {}", json);
                    async fn get_data(json: String) -> MsgCore {
                        let client = reqwest::Client::new();
                        let res = client
                            .post(format!("{}/strainfasta", get_host_url()))
                            .header("Content-Type", "application/json")
                            .body(json)
                            .send()
                            .await;

                        //An empty Vec means "it did not work"; the handler
                        //below turns that into an alert rather than saving a
                        //zero-byte zip
                        let bytes = match res {
                            Err(e) => {
                                log::error!("fasta download failed: {}", e);
                                Vec::new()
                            }
                            Ok(res) if !res.status().is_success() => {
                                log::error!("fasta download rejected: {}", res.status());
                                Vec::new()
                            }
                            Ok(res) => match res.bytes().await {
                                Ok(b) => b.to_vec(),
                                Err(e) => {
                                    log::error!("fasta download truncated: {}", e);
                                    Vec::new()
                                }
                            },
                        };

                        MsgCore::DownloadFASTAgot(bytes)
                    }
                    ctx.link().send_future(get_data(json));
                }
                false
            }

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
            }

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::SetStrainSelected(id, tosel) => {
                if tosel {
                    self.selected_strains.insert(id);
                } else {
                    self.selected_strains.remove(&id);
                }
                false
            }

            ////////////////////////////////////////////////////////////
            // Hide a column specified by name
            MsgCore::HideColumn(col) => {
                self.show_columns.retain(|s| s != &col);
                true
            }

            ////////////////////////////////////////////////////////////
            // Show a column specified by name
            MsgCore::ShowColumn(col) => {
                if col != "" {
                    //log::debug!("Adding new column to show {}", col);
                    self.show_columns.insert(col);
                    //log::debug!("now cols: {:?}", self.show_columns);
                }
                true
            }

            ////////////////////////////////////////////////////////////
            // x
            MsgCore::OpenBTracker => {
                let window = window().expect("no window");
                log::debug!("btracker with strains {:?}", self.selected_strains);
                if self.selected_strains.is_empty() {
                    window
                        .alert_with_message("No strains specified")
                        .expect("failed to alert");
                } else {
                    let list_strains_withcomma = self
                        .selected_strains
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    let url = format!("https://nextstrain.org/community/vigzy77/BTracker/Bacillus-cereus-group/All-Species(Mash-distance-NJ)?s={}",list_strains_withcomma);
                    window
                        .open_with_url_and_target(url.as_str(), "_blank")
                        .expect("Failed to open url");
                }
                false
            }
        }
    }

    ////////////////////////////////////////////////////////////
    /// Top renderer of the page
    fn view(&self, ctx: &Context<Self>) -> Html {
        let onsize = ctx
            .link()
            .callback(|size: ComponentSize| MsgCore::WindowResize(size));

        let current_page = match self.current_page {
            CurrentPage::Home => self.view_landing_page(&ctx),
            CurrentPage::Search => self.view_search_pane(&ctx),
            CurrentPage::Tree => self.view_tree_pane(&ctx),
            CurrentPage::Statistics => self.view_statistics_pane(&ctx),
            CurrentPage::Help => self.view_help_pane(&ctx),
            CurrentPage::About => self.view_about_pane(&ctx),
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

        //Nothing on any page works without the database description, so if it
        //failed to load, say so above whatever the page managed to render
        let html_metadata_error = match &self.metadata_error {
            Some(msg) => html! {
                <div class="errormessage">
                    <b>{"The database could not be contacted."}</b>
                    <br/>
                    {msg}
                    <br/>
                    {"Reloading the page will try again."}
                </div>
            },
            None => html! {},
        };

        html! {
            <div>
                { html_top_buttons }
                { html_metadata_error }
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
    let document = window()
        .expect("no window")
        .document()
        .expect("no document on window");
    let location = document.location().expect("no location");
    let protocol = location.protocol().expect("no protocol");
    let host = location.host().expect("no host");

    let url = format!("{}//{}", protocol, host);
    //log::debug!("{}",url);
    url
}
