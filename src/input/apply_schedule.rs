use serde_helpers::with::duration::duration_option_format;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use time::Duration;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ApplySchedule {
    #[serde(rename = "@path")]
    pub path: PathBuf,

    #[serde(rename = "@firstStopTime", with = "duration_option_format", default, skip_serializing_if = "Option::is_none")]
    pub first_stop_time: Option<Duration>,

    #[serde(rename = "@lastStopTime", with = "duration_option_format", default, skip_serializing_if = "Option::is_none")]
    pub last_stop_time: Option<Duration>,
}