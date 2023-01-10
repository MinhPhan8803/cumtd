use std::error::Error;
use time::Date;
use time::format_description::well_known::Iso8601;
use serde::{Deserialize, Deserializer};
use reqwest::Client;

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct CalendarDate {
    #[serde(deserialize_with = "deserialize_to_date")]
    date: Date,
    service_id: String,
}

fn deserialize_to_date<'de, D>(deserializer: D) -> Result<Date, D::Error>
where D: Deserializer<'de> {
    let buf = Deserialize::deserialize(deserializer)?;
    Ok(Date::parse(buf, &Iso8601::DEFAULT).unwrap())
}

#[derive(Deserialize, Debug)]
struct GetDatesResponse {
    calendar_dates: Vec<CalendarDate>,
}

pub enum DatesQueryType {
    ByDate(Date),
    ByService(String),
}

pub async fn dates_query_api(key: String, query_type: &DatesQueryType) -> Result<Vec<CalendarDate>, Box<dyn Error>> {
    let client = Client::builder().build()?;
    let request = match query_type {
        DatesQueryType::ByDate(date) => {
            let date_string = date.format(&Iso8601::DEFAULT).unwrap();
            client
            .get("https://developer.cumtd.com/api/v2.2/json/getcalendardatesbydate")
            .query(&[("key", key.as_str()), ("date", date_string.as_str())])
            .send().await
        },
        DatesQueryType::ByService(id) => {
            client
            .get("https://developer.cumtd.com/api/v2.2/json/getcalendardatesbydate")
            .query(&[("key", key.as_str()), ("service_id", id.as_str())])
            .send().await
        },
    };
    let json= request?.json::<GetDatesResponse>().await?;
    Ok(json.calendar_dates)
}