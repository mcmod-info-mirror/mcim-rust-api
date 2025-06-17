pub mod common;
pub mod curseforge;
pub mod modrinth;
pub mod translate;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

pub fn deserialize_bson_datetime_flexible<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: From<DateTimeWrapper>,
{
    let value = serde_value::Value::deserialize(deserializer)?;

    if let Ok(bson_datetime) = bson::DateTime::deserialize(value.clone()) {
        return Ok(T::from(DateTimeWrapper::from(Some(
            bson_datetime.to_chrono(),
        ))));
    }

    if let Ok(s) = String::deserialize(value) {
        if !s.is_empty() {
            if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
                return Ok(T::from(DateTimeWrapper::from(Some(dt.with_timezone(&Utc)))));
            }
        }
    }

    Ok(T::from(DateTimeWrapper::from(None)))
}
pub struct DateTimeWrapper(pub Option<DateTime<Utc>>);

impl From<Option<DateTime<Utc>>> for DateTimeWrapper {
    fn from(opt: Option<DateTime<Utc>>) -> Self {
        DateTimeWrapper(opt)
    }
}

impl From<DateTimeWrapper> for DateTime<Utc> {
    fn from(wrapper: DateTimeWrapper) -> Self {
        wrapper.0.unwrap_or_else(|| Utc::now())
    }
}

impl From<DateTimeWrapper> for Option<DateTime<Utc>> {
    fn from(wrapper: DateTimeWrapper) -> Self {
        wrapper.0
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}
