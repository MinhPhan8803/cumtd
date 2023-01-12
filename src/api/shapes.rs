use crate::Error;
use serde::Deserialize;
use reqwest::Client;

#[derive(Deserialize, Debug, Clone)]
pub struct Shape {
    shape_dist_traveled: f64,
    shape_pt_lat: f64, 
    shape_pt_lon: f64, 
    shape_pt_sequence: i32,
    #[serde(default)]
    stop_id: Option<String>,
}

impl Shape {
    pub fn dist_traveled(&self) -> f64 {
        self.shape_dist_traveled
    }
    pub fn point_lat(&self) -> f64 {
        self.shape_pt_lat
    }
    pub fn point_lon(&self) -> f64 {
        self.shape_pt_lon
    }
    pub fn point_sequence(&self) -> i32 {
        self.shape_pt_sequence
    }
    pub fn stop_id(&self) -> &Option<String> {
        &self.stop_id
    }
}

#[derive(Deserialize, Debug)]
struct GetShapesResponse {
    shapes: Vec<Shape>,
}

pub enum ShapesQueryType {
    FullShape(String),
    BetweenStops(ShapeSpecifier),
}

pub async fn shapes_query_api(key: &str, query_type: &ShapesQueryType) -> Result<Vec<Shape>, Error> {
    let client = Client::builder().build()?;
    let request = match query_type {
        ShapesQueryType::FullShape(id) => {
            client
            .get("https://developer.cumtd.com/api/v2.2/json/getshape")
            .query(&[("key", key), ("shape_id", &id)])
            .send()
            .await
        },
        ShapesQueryType::BetweenStops(specs) => {
            client
            .get("https://developer.cumtd.com/api/v2.2/json/getshapebetweenstops")
            .query(&[("key", key), ("begin_stop_id", &specs.begin_id), ("end_stop_id", &specs.end_id), ("shape_id", &specs.shape_id)])
            .send()
            .await
        },
    };
    let json= request?.json::<GetShapesResponse>().await?;
    Ok(json.shapes)
}

pub struct ShapeSpecifier {
    begin_id: String,
    end_id: String,
    shape_id: String,
}

impl ShapeSpecifier {
    pub fn new(begin_stop_id: &str, end_stop_id: &str, shape_id: &str) -> ShapeSpecifier {
        ShapeSpecifier { 
            begin_id: begin_stop_id.to_owned(), 
            end_id: end_stop_id.to_owned(), 
            shape_id: shape_id.to_owned() 
        }
    }
}