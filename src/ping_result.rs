use std::time::Duration;
use std::io::ErrorKind;

pub type PingResult = Result<Duration, ErrorKind>;
