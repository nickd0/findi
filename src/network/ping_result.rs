use std::time::Duration;
use std::io::Error;

pub type PingResult = Result<Duration, Error>;
pub type PingResultOption = Option<Duration>;
