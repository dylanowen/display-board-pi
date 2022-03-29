use crate::sunrise_sunset_api::query_daylight;
use crate::DaylightResponse;
use async_recursion::async_recursion;
use chrono::{DateTime, Duration, Utc};

pub enum Daylight {
    Sunrise {
        response: DaylightResponse,
    },
    Sunset {
        response: DaylightResponse,
    },
    Unknown {
        date: Option<String>,
        try_again: DateTime<Utc>,
        backoff_seconds: usize,
    },
}

impl Daylight {
    pub fn unknown(date: Option<String>) -> Self {
        Daylight::Unknown {
            date,
            try_again: Utc::now(),
            backoff_seconds: 1,
        }
    }

    #[async_recursion]
    pub async fn update(self, now: DateTime<Utc>) -> Self {
        match self {
            Daylight::Sunrise { response } => {
                if now >= response.sunrise {
                    Daylight::Sunset { response }.update(now).await
                } else {
                    // we're valid for this time
                    Daylight::Sunrise { response }
                }
            }
            Daylight::Sunset { response } => {
                if now >= response.sunset {
                    Self::unknown(Some("tomorrow".to_string()))
                        .update(now)
                        .await
                } else {
                    // we're valid for this time
                    Daylight::Sunset { response }
                }
            }
            Daylight::Unknown {
                date,
                try_again,
                backoff_seconds,
            } => {
                if now >= try_again {
                    query_or_backoff(now, date, backoff_seconds).await
                } else {
                    // keep waiting, we're not ready to retry
                    Daylight::Unknown {
                        date,
                        try_again,
                        backoff_seconds,
                    }
                }
            }
        }
    }

    pub fn until(&self, now: DateTime<Utc>) -> (usize, usize) {
        match self {
            Daylight::Sunrise {
                response: DaylightResponse { sunrise, .. },
            } => until(*sunrise - now),
            Daylight::Sunset {
                response: DaylightResponse { sunset, .. },
            } => until(*sunset - now),
            Daylight::Unknown { .. } => (0, 0),
        }
    }
}

impl Default for Daylight {
    fn default() -> Self {
        Self::unknown(None)
    }
}

async fn query_or_backoff(
    now: DateTime<Utc>,
    date: Option<String>,
    last_backoff_seconds: usize,
) -> Daylight {
    match query_daylight(&date).await {
        Ok(response) => {
            // even if we're not on sunrise, we'll figure it out in our next loop
            Daylight::Sunrise { response }.update(now).await
        }
        Err(error) => {
            // wait for max 12 hours
            let backoff_seconds = (60 * 60 * 12).min(last_backoff_seconds * 2);
            let try_again = now + Duration::seconds(backoff_seconds as i64);
            log::error!("Error getting daylight, trying at {try_again}: {error}");

            Daylight::Unknown {
                date,
                try_again,
                backoff_seconds,
            }
        }
    }
}

fn until(duration: Duration) -> (usize, usize) {
    (
        (duration.num_hours() % 24) as usize,
        (duration.num_minutes() % 60) as usize,
    )
}
