use std::collections::HashMap;
use std::{fmt::Display, error::Error};
use std::path::Path;
use serde::Deserialize;
use reqwest::Client;
use csv::Reader;

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Stop {
    stop_id: String,
    stop_name: String,
    code: String,
    stop_points: Vec<StopPoint>,
}

impl Stop {
    pub fn id(&self) -> &str {
        &self.stop_id
    }
    pub fn name(&self) -> &str {
        &self.stop_name
    }
    pub fn code(&self) -> &str {
        &self.code
    }
    pub fn stop_points(&self) -> &[StopPoint] {
        &self.stop_points
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct StopPoint {
    #[serde(alias = "stop_code")]
    code: String,
    stop_id: String,
    stop_lat: f64,
    stop_lon: f64,
    stop_name: String,
}

impl StopPoint {
    pub fn code(&self) -> &str {
        &self.code
    }
    pub fn id(&self) -> &str {
        &self.stop_id
    }
    pub fn name(&self) -> &str {
        &self.stop_name
    }
    pub fn lat(&self) -> f64 {
        self.stop_lat
    }
    pub fn lon(&self) -> f64 {
        self.stop_lon
    }
}

impl Eq for StopPoint {}

impl PartialEq for StopPoint {
    fn eq(&self, other: &Self) -> bool {
        self.stop_id == other.stop_id
    }
}

#[derive(Deserialize, Debug)]
struct GetStopsResponse {
    stops: Vec<Stop>,
}

pub async fn stops_query_api(key: String, query_type: &StopsQueryType) -> Result<Vec<Stop>, Box<dyn Error>> {
    let client = Client::builder().build()?;
    let request = match query_type {
        StopsQueryType::ById(ids) => {
            let key_query = ids.join(";");
            client
            .get("https://developer.cumtd.com/api/v2.2/json/getstop")
            .query(&[("key", key.as_str()), ("stop_id", key_query.as_str())])
            .send().await
        },
        StopsQueryType::All => {
            client
            .get("https://developer.cumtd.com/api/v2.2/json/getstops")
            .query(&[("key", key.as_str())])
            .send().await
        },
        StopsQueryType::ByLatLon(latlon) => {
            match latlon.count {
                Some(x) => {
                    client
                    .get("https://developer.cumtd.com/api/v2.2/json/getstopsbylatlon")
                    .query(&[("key", key.as_str()), ("lat", latlon.lat.to_string().as_str()), ("lon", latlon.lon.to_string().as_str()), ("count", x.to_string().as_str())])
                    .send().await
                }
                None => {
                    client
                    .get("https://developer.cumtd.com/api/v2.2/json/getstopsbylatlon")
                    .query(&[("key", key.as_str()), ("lat", latlon.lat.to_string().as_str()), ("lon", latlon.lon.to_string().as_str())])
                    .send().await
                }
            }
        },
    };
    let json = request?.json::<GetStopsResponse>().await?;
    Ok(json.stops)
}

pub async fn stops_query_gtfs<P: AsRef<Path>>(query_type: &StopsQueryType, path: P) -> Result<Vec<Stop>, Box<dyn Error>> {
    let mut reader = Reader::from_path(path)?;
    let read_result = reader.deserialize::<StopPoint>();
    let serde_result: Result<Vec<StopPoint>, _> = match query_type {
        StopsQueryType::ById(id) => {
            read_result
            .filter(| single_res | single_res.is_err() || 
            id.contains(&single_res.as_ref().unwrap().stop_id))
            .collect()
        },
        StopsQueryType::All => {
            read_result
            .collect()
        },
        StopsQueryType::ByLatLon(latlon) => {
            read_result
            .filter(| single_res | single_res.is_err() || 
            (latlon.lat == single_res.as_ref().unwrap().stop_lat 
            && latlon.lon == single_res.as_ref().unwrap().stop_lon))
            .collect()
        }
    };
    let mut points_map: HashMap<String, Vec<StopPoint>> = HashMap::new();
    for point in serde_result? {
        match points_map.get_mut(&point.code) {
            Some(x) => x.push(point),
            None => {points_map.insert(point.code.clone(), vec!(point)); ()},
        };
    }
    Ok(points_map
    .into_iter()
    .map(|(key, val)| Stop {
        stop_id: val[0].stop_id.split(":").next().unwrap().to_string(),
        stop_name: val[0].stop_name.split(" (").next().unwrap().to_string(),
        code: key,
        stop_points: val,
    })
    .collect())
}

#[derive(Debug, Clone)]
pub enum StopsQueryType {
    ById(Vec<String>),
    All,
    ByLatLon(LatLonQuery),
}

#[derive(Debug, Clone)]
pub struct LatLonQuery {
    lat: f64,
    lon: f64,
    count: Option<i32>,
}

impl LatLonQuery {
    pub fn builder() -> LatLonQueryBuilder {
        LatLonQueryBuilder { lat: None, lon: None, count: None }
    }
    pub fn new(lat: f64, lon: f64) -> LatLonQuery {
        LatLonQuery { lat, lon, count: None }
    }
    pub fn new_with_count(lat: f64, lon: f64, count: i32) -> LatLonQuery {
        LatLonQuery { lat, lon, count: Some(count) }
    }
}

#[derive(Debug, Clone)]
pub struct LatLonQueryBuilder {
    lat: Option<f64>,
    lon: Option<f64>,
    count: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct LatLonQueryBuilderError;

impl Display for LatLonQueryBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing latitude or longitude")
    }
}

impl LatLonQueryBuilder {
    pub fn lat<'a>(&'a mut self, lat: f64) -> &'a mut LatLonQueryBuilder {
        self.lat = Some(lat);
        self
    }
    pub fn lon<'a>(&'a mut self, lon: f64) -> &'a mut LatLonQueryBuilder {
        self.lon = Some(lon);
        self
    }
    pub fn count<'a>(&'a mut self, count: i32) -> &'a mut LatLonQueryBuilder {
        self.count = Some(count);
        self
    }
    pub fn build(&self) -> Result<LatLonQuery, LatLonQueryBuilderError> {
        if self.lat.is_none() || self.lon.is_none() {
            return Err(LatLonQueryBuilderError);
        }
        Ok(
            LatLonQuery { lat: self.lat.unwrap(), lon: self.lon.unwrap(), count: self.count }
        )
    } 
}