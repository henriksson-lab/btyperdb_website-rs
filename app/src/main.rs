use anyhow::Result;
use my_web_app::MyTestStruct;
use yew::{
    format::{Json, Nothing, Text},
    prelude::*,
    services::{
        fetch::{FetchTask, Request, Response},
        FetchService,
    },
};

#[derive(Debug)]
#[derive(PartialEq)]
enum CurrentPage {
    Home,
    Search,
    Statistics,
    Help,
    About,
}


#[derive(Debug)]
enum Msg {

    OpenPage(CurrentPage),

/* 
    SetText(Option<String>),
    SetStruct(Option<MyTestStruct>),*/
}

struct Model {
    link: ComponentLink<Self>,
    current_page: CurrentPage,
    /* 
    value: i64,
    text: Option<String>,
    task: Option<FetchTask>,
    obj: Option<MyTestStruct>,*/
}
impl Component for Model {
    type Message = Msg;

    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            current_page: CurrentPage::Home,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {

            Msg::OpenPage(page) => {
                self.current_page = page;
                true
            }

            /*
            
            Msg::SetStruct(data) => {
                log::trace!("Update: {:?}", data);
                self.obj = data;
                true
            }


            Msg::Fetch => {
                let request = Request::get("/hello")
                    .body(Nothing)
                    .expect("Could not build request");
                let callback = self
                    .link
                    .callback(|response: Response<Text>| Msg::SetText(response.into_body().ok()));
                let task = FetchService::fetch(request, callback).expect("Failed to start request");
                self.task = Some(task);
                false
            }
            Msg::FetchStruct => {
                let request = Request::get("/json-data")
                    .body(Nothing)
                    .expect("Could not build request");
                let callback =
                    self.link
                        .callback(|response: Response<Json<Result<MyTestStruct>>>| {
                            log::debug!("{:?}", response);
                            let Json(data) = response.into_body();
                            Msg::SetStruct(data.ok())
                        });
                let task = FetchService::fetch(request, callback).expect("Failed to start request");
                self.task = Some(task);
                false
            }

 */
            

        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }


    




    fn view(&self) -> Html {

        let current_page = match self.current_page { 
            CurrentPage::Home => self.view_landing_page(),
            CurrentPage::Search => self.view_search_pane(),
            CurrentPage::Statistics => self.view_statistics_pane(),
            CurrentPage::Help => self.view_help_pane(),
            CurrentPage::About => self.view_about_pane()
        };

        

        html! {
            <div>

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

                { current_page }
                //{ page_search }

            </div>
        }
    }
}


fn active_if(cond: bool) -> String {
    if cond {
        "active".to_string()
    } else {
        "".to_string()
    }
}


impl Model {



    fn view_search_line(&self) -> Html {

        html! {
			<div class="divSearchField">
				<button name="bDelete" class="buttonspacer">
                    {"X"}
                </button>
				<select class="columndrop" name="selectfield">
					<option value="BTyperDB_ID">{"BTyperDB_ID"}</option>
					<option value="NCBI_BioSample">{"NCBI_BioSample"}</option>
				</select>
				<label>
                    {"&nbsp; From: "}
					<input class="textbox" type="text" name="value" value="20000"/>
                    {"&nbsp; To: "}
					<input class="textbox" type="text" name="value2" value="1000000000"/>
				</label>				
			</div>
        }
        
    }


    fn view_search_pane(&self) -> Html {

        html! {
            <div>
                <div class="App-divider">
                    {"Search for genomes"}                
                    <button class="toggleview">
                        {"Hide panel"}                    
                    </button>
                </div>

                <div class="withspacer"> /////////likely need to fix divs here
                </div>
                <div>
                    { self.view_search_line() }
                    <div>
                        
                        <button class="buttonspacer">
                            {"Add filter"}
                        </button>
                        <button class="buttonspacer">
                            {"Search"}
                        </button>

                    </div>
                </div>
            </div>
        }
        
    }



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
                {"&nbsp;"}
                
            </div>
            <br />
            </div>        
        }
        
    }




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


    fn view_landing_page(&self) -> Html {

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
                    {"xxx total B. cereus group genomes with curated metadata"}
                </p>

                <button class="toolbutton" onclick=self.link.callback(|_| Msg::OpenPage(CurrentPage::Search))>
                    {"Search BTyperDB"}
                </button>

            </div>
        }
    }

}


fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}
