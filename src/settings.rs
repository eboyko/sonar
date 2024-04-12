use std::fs;
use std::fs::canonicalize;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use url::Url;

#[derive(Parser)]
#[command(version, about)]
pub(crate) struct Settings {
    #[arg(long = "url", help = "Sets the capture stream URL")]
    pub(crate) stream_url: Url,

    #[arg(long = "timeout", default_value = "5", value_parser = get_duration, help = "Sets the seconds for the stream connection timeout")]
    pub(crate) stream_timeout: Duration,

    #[arg(long = "path", default_value = "records", value_parser = get_records_path, help = "Sets the path to the records directory")]
    pub(crate) records_path: PathBuf,

    #[arg(long = "port", default_value = "3000", help = "Sets the monitor server port")]
    pub(crate) monitor_port_number: u16,

    #[arg(long, default_value = "info", help = "Sets the log level")]
    pub(crate) log_level: log::LevelFilter,
}

pub(crate) fn parse() -> Settings {
    Settings::parse()
}

fn get_duration(seconds: &str) -> Result<Duration, ParseIntError> {
    Ok(Duration::from_secs(seconds.parse()?))
}

fn get_records_path(path: &str) -> Result<PathBuf, std::io::Error> {
    let path = PathBuf::from(path);
    fs::create_dir_all(&path)?;
    Ok(canonicalize(path)?)
}
