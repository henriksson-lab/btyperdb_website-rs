use crate::core_model::*;

use yew::prelude::*;

impl Model {

    ////////////////////////////////////////////////////////////
    /// x
    pub fn view_about_pane(&self, _ctx: &Context<Self>) -> Html {

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



}
