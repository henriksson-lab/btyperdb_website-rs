use crate::core_model::*;


use yew::{
    prelude::*,
};

use crate::component_map::GeoMapView;


impl Model {




    ////////////////////////////////////////////////////////////
    /// Page: Statistics about the database
    pub fn view_statistics_pane(&self) -> Html {

        let html_stats = if let Some(db_metadata) = &self.db_metadata {





            html! { 
                <p>
                    <div>
                        {"Number of genomes per country"}

                        { GeoMapView::draw_geojson(&self.geojson, &db_metadata.hist_country) }



                    </div>
                    <div>
                        {"GTDB species"}
                        { svg_horizontal_bar_fractions(&db_metadata.hist_gtdb_species) }
                    </div>
                    <div>
                        {"Isolation source (Source 1)"}
                        { svg_horizontal_bar_fractions(&db_metadata.hist_source1) }
                    </div>
                    <div>
                        {"Human illness"}
                        { svg_horizontal_bar_fractions(&db_metadata.hist_humanillness) }
                    </div>
                    <div>
                        {"BTyper3 adjusted panC group"}
                        { svg_horizontal_bar_fractions(&db_metadata.hist_pancgroup) }
                    </div>

                </p>
            }

        } else {
            html! { {""} }
        };

        html! {
            <div>
            <div class="App-divider">{"Statistics"}</div>
            <div class="landingdiv">
                {html_stats}
                {""}                
            </div>
            <br />
            </div>        
        }
        
    }




}



////////////////////////////////////////////////////////////
/// x
pub type DatabaseHistogram = Vec<(String,i32)>;


////////////////////////////////////////////////////////////
/// x
pub fn test_svg_horizontal_bar_fractions() -> Html {
    let mut list_entry: DatabaseHistogram = Vec::new();
    list_entry.push(("foo".to_string(),100 as i32));
    list_entry.push(("bar".to_string(),50 as i32));
    svg_horizontal_bar_fractions(&list_entry)
}


////////////////////////////////////////////////////////////
/// x
pub fn svg_horizontal_bar_fractions(list_entry: &DatabaseHistogram) -> Html {

    let list_colors= vec!["red","blue","green"];


    let mut total_cnt: i32 =0;
    for (_n,cnt) in list_entry.iter() {
        total_cnt += cnt;
    }

    let scale_x = 800 as f64 / total_cnt as f64;

    let mut cur_x = 0;
    let mut outp = Vec::new();
    for (i, (name, cnt)) in list_entry.iter().enumerate() {

        let use_color = list_colors.get(i % list_colors.len()).unwrap().to_string();
        
        outp.push(html! { 
            <rect x={format!("{}",cur_x as f64 * scale_x)} y="0" width={format!("{}",*cnt as f64 * scale_x)} height="10" style={format!{"fill:{};stroke-width:.5;stroke-linecap:round;stroke-miterlimit:2", use_color}} /> 
        });

        outp.push(html! {
            <text style="font-style:normal;font-weight:400;font-size:10.5833px;line-height:1.25;font-family:sans-serif;fill:#000;stroke:none;stroke-width:.26"  transform={format!("translate({},20) rotate(45)",(cur_x+cnt/2) as f64 * scale_x)}>
                {name.clone()}
            </text>
        });

        cur_x += cnt;
    }
    html! {
        <svg viewBox={format!("0 0 1000 150")} xmlns="http://www.w3.org/2000/svg">
            {
                outp
            }
        </svg>    
    }

}

