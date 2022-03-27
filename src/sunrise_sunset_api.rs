use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Results {
    results: Result,
    #[serde(default = "Status::unknown_error")]
    status: Status,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Result {
    sunrise: DateTime<Utc>,
    sunset: DateTime<Utc>,
    day_length: usize,
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
