use crate::Error;
//use std::path::Path;
use serde::Deserialize;
use reqwest::Client;
//use csv::Reader;

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Route {
    route_color: String,
    route_id: String,
    route_long_name: String,
    route_short_name: String,
    route_text_color: String,
}

impl Route {
    pub fn color(&self) -> &str {
        &self.route_color
    }
    pub fn id(&self) -> &str {
        &self.route_id
    }
    pub fn long_name(&self) -> &str {
        &self.route_long_name
    }
    pub fn short_name(&self) -> &str {
        &self.route_short_name
    }
    pub fn text_color(&self) -> &str {
        &self.route_text_color
    }
}

#[derive(Deserialize, Debug)]
struct GetRoutesResponse {
    routes: Vec<Route>,
}

#[derive(Debug, Clone)]
pub enum RoutesQueryType {
    ById(Vec<String>),
    All,
    ByStop(String),
}

pub async fn routes_query_api(key: &str, query_type: &RoutesQueryType) -> Result<Vec<Route>, Error> {
    let client = Client::builder().build()?;
    let request = match query_type {
        RoutesQueryType::ById(ids) => {
            let key_query = ids.join(";");
            client
            .get("https://developer.cumtd.com/api/v2.2/json/getroute")
            .query(&[("key", key), ("route_id", &key_query)])
            .send()
            .await
        },
        RoutesQueryType::All => {
            client
            .get("https://developer.cumtd.com/api/v2.2/json/getroutes")
            .query(&[("key", key)])
            .send()
            .await
        },
        RoutesQueryType::ByStop(stop) => {
            client
            .get("https://developer.cumtd.com/api/v2.2/json/getroutesbystop")
            .query(&[("key", key), ("stop_id", stop)])
            .send()
            .await
        }
    };
    let json= request?.json::<GetRoutesResponse>().await?;
    Ok(json.routes)
}