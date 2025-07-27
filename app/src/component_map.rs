use std::collections::HashMap;

use geojson::GeoJson;
use geojson::{Geometry, Value};

use yew::{
    prelude::*,
};


////////////////////////////////////////////////////////////
/// x
#[derive(Debug)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
}
impl Color {
    pub fn as_string(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)           
    }
}

////////////////////////////////////////////////////////////
/// x
#[derive(Debug)]
pub struct GeoStats {
    minx: f64,
    miny: f64,
    maxx: f64,
    maxy: f64,
}

pub struct GeoMapView {
}
impl GeoMapView {

    ////////////////////////////////////////////////////////////
    /// x
    pub fn draw_geojson(geojson: &GeoJson, hist: &Vec<(String,i32)>) -> Html {

        //Place to store the extent of the map
        let mut geostat = GeoStats {
            minx:0 as f64,
            miny:0 as f64,
            maxx:0 as f64,
            maxy:0 as f64
        };

        //Function to scale the values
        fn transform_scale(v:i32) -> f64 {
            ((v + 1) as f64).log10()
        }

        //Figure out the largest value for normalization
        let max_cnt = *hist.iter().map(|(_,v)| v).max().unwrap_or(&666);
        let transformed_max_cnt = transform_scale(max_cnt);//((max_cnt + 1) as f64).log10();

        //Transform all values, store in map
        let mut map_hist = HashMap::new();
        for (k,v) in hist {
            map_hist.insert(k.clone(), transform_scale(*v) / transformed_max_cnt);
        }

        //Get all polygons
        let mut outpoly: Vec<Html> = Vec::new();
        Self::process_geojson(&geojson, &mut outpoly, &mut geostat, &"".to_string(), &map_hist);

        //log::debug!("geostats {:?}",geostat);

        html! {
            <svg viewBox={format!("{} {} {} {}", geostat.minx, geostat.miny,  geostat.maxx-geostat.minx, geostat.maxy-geostat.miny)}>
                { outpoly }
            </svg>
        }
    }



    ////////////////////////////////////////////////////////////
    /// Process top-level GeoJSON Object
    fn process_geojson(gj: &GeoJson, outpoly: &mut Vec<Html>, geostats: &mut GeoStats, current_country: &String, hist: &HashMap<String,f64>) {
        match *gj {
            GeoJson::FeatureCollection(ref ctn) => {

                for feature in &ctn.features {

                    let mut current_country = current_country;
                    if let Some(map) = &feature.properties {
                        let iso_a3=map.get("iso_a3");
                        if let Some(f) = iso_a3 {
                            if let serde_json::Value::String(s) = f {
                                current_country = s;
                            }
                        }
                    }
                    if let Some(ref geom) = feature.geometry {
                        Self::process_geometry(geom, outpoly, geostats, current_country, hist)
                    }
                }
            }
            GeoJson::Feature(ref feature) => {

                log::debug!("Feature {:?}",feature.properties);

                //Does not seem to ever happen
                if let Some(map) = &feature.properties {
                    let v:Vec<String> = map.keys().cloned().collect();
                    log::debug!("map {:?}",v);
                }

                if let Some(ref geom) = feature.geometry {
                    Self::process_geometry(geom, outpoly, geostats, current_country, hist)
                }
            }
            GeoJson::Geometry(ref geometry) => Self::process_geometry(geometry, outpoly, geostats, current_country, hist),
        }
    }


    ////////////////////////////////////////////////////////////
    /// Process GeoJSON geometries
    /// https://datatracker.ietf.org/doc/html/rfc7946#section-3.1.6
    fn process_geometry(geom: &Geometry, outpoly: &mut Vec<Html>, geostats: &mut GeoStats, current_country: &String, hist: &HashMap<String,f64>) {
        match &geom.value {
            Value::Polygon(polytype) => {
                for p in polytype {
                    Self::process_poly(p, outpoly, geostats, current_country, hist);
                }
            },
            Value::MultiPolygon(polytype) => {
                for pp in polytype {
                    for p in pp {
                        Self::process_poly(p, outpoly, geostats, current_country, hist);
                    }
                }
            },
            Value::GeometryCollection(ref gc) => {
                for geometry in gc {
                    Self::process_geometry(geometry, outpoly, geostats, current_country, hist)
                }
            }
            // Point, LineString, and their Multi– counterparts
            _ => log::debug!("Matched some other geometry"),
        }
    }


    ////////////////////////////////////////////////////////////
    /// x
    fn process_poly(points: &Vec<Vec<f64>>, outpoly: &mut Vec<Html>, geostats: &mut GeoStats, current_country: &String, hist: &HashMap<String,f64>) {

        let color = if let Some(cnt) = hist.get(current_country) {
            let cnt = (*cnt * 255_f64) as u8;
            let color = Color {red: cnt, blue: 0, green: 0};
            //log::debug!("{:?}", color);
            color.as_string()
        } else {
            "darkgray".to_string()
        };
        //log::debug!("--- {:?}", color);

        let mut outs = String::new();
        for p in points {
            let x=*p.get(0).unwrap();
            let y=- *p.get(1).unwrap();
            outs.push_str(format!("{},{} ", x, y).as_str());        

            if x < geostats.minx { // Can take out as a preprocessing operation
                geostats.minx = x;
            }
            if y < geostats.miny {
                geostats.miny = y;
            }

            if x > geostats.maxx {
                geostats.maxx = x;
            }
            if y > geostats.maxy {
                geostats.maxy = y;
            }
            //log::debug!("c {:?}",current_country);
        }
        outpoly.push(html! {
            <polygon points={outs} fill={color} stroke="black" /> 
        });
    }



}
