use anyhow::anyhow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Deserialize, Serialize, Debug)]
pub struct DaylightCollection {
    pub results: DaylightResponse,
    #[serde(default = "Status::unknown_error")]
    pub status: Status,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DaylightResponse {
    pub sunrise: DateTime<Utc>,
    pub sunset: DateTime<Utc>,
    pub day_length: usize,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Ok,
    InvalidRequest,
    InvalidDate,
    UnknownError,
}

impl Status {
    fn unknown_error() -> Status {
        Status::UnknownError
    }
}

pub async fn query_daylight<S: Display>(date: &Option<S>) -> anyhow::Result<DaylightResponse> {
    let url = format!(
        "https://api.sunrise-sunset.org/json?lat=40.743722&lng=-73.978020&formatted=0{}",
        date.as_ref()
            .map(|d| format!("&date={d}"))
            .unwrap_or_default()
    );

    log::debug!("Querying Daylight: {url}");

    let collection = reqwest::get(url)
        .await?
        .json::<DaylightCollection>()
        .await?;

    log::debug!("API Response: {collection:?}");

    match collection.status {
        Status::Ok => Ok(collection.results),
        Status::InvalidRequest => Err(anyhow!("Invalid Request")),
        Status::InvalidDate => Err(anyhow!("Invalid Date")),
        Status::UnknownError => Err(anyhow!("Unknown Error")),
    }
}
