use owo_colors::{CssColors, OwoColorize};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = AQI::from_args();

    let client = reqwest::Client::new();

    match opt.command {
        Opt::Search { search_term } => {
            let resp = client
                .get("https://api.waqi.info/search/")
                .query(&[("token", &opt.api_token), ("keyword", &search_term)])
                .send()
                .await?
                .json::<WeatherResponse<Vec<StationMeta>>>()
                .await?;

            for station_meta in resp.data.iter() {
                println!(
                    "id: {}, url: {}",
                    station_meta.uid, station_meta.station.url
                )
            }

            Ok(())
        }
        Opt::Info {
            scale,
            station_name,
        } => {
            let resp = client
                .get(format!("https://api.waqi.info/feed/{}/", station_name))
                .query(&[("token", opt.api_token)])
                .send()
                .await?
                .json::<WeatherResponse<WeatherData>>()
                .await?;
            println!(
                "{}
aqi: {}",
                resp.data.city.name.blue().bold(),
                match resp.data.aqi {
                    0..=50 => {
                        resp.data.aqi.color(CssColors::Green)
                    }
                    51..=100 => {
                        resp.data.aqi.color(CssColors::Yellow)
                    }
                    101..=150 => {
                        resp.data.aqi.color(CssColors::Orange)
                    }
                    151..=200 => {
                        resp.data.aqi.color(CssColors::Red)
                    }
                    201..=300 => {
                        resp.data.aqi.color(CssColors::Purple)
                    }
                    301..=500 => {
                        resp.data.aqi.color(CssColors::Maroon)
                    }
                    _ => {
                        resp.data.aqi.color(CssColors::White)
                    }
                }
            );
            Ok(())
        }
    }
}

#[derive(Debug)]
enum Scale {
    Celsius,
    Fahrenheit,
    Kelvin,
}
#[derive(Debug)]
struct InvalidScale;
impl fmt::Display for InvalidScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid scale")
    }
}

impl FromStr for Scale {
    type Err = InvalidScale;
    fn from_str(temp: &str) -> Result<Self, Self::Err> {
        match temp {
            "c" => Ok(Scale::Celsius),
            "celsius" => Ok(Scale::Celsius),
            "f" => Ok(Scale::Fahrenheit),
            "fahrenheit" => Ok(Scale::Fahrenheit),
            "k" => Ok(Scale::Kelvin),
            "kelvin" => Ok(Scale::Kelvin),
            _ => Err(InvalidScale),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "aqi", about = "Find weather quality from stations near you")]
struct AQI {
    #[structopt(short = "t", long = "token", env = "API_TOKEN")]
    api_token: String,
    #[structopt(subcommand)]
    command: Opt,
}
#[derive(Debug, StructOpt)]
enum Opt {
    Search {
        search_term: String,
    },
    Info {
        #[structopt(
            short = "s",
            long = "scale",
            default_value = "fahrenheit",
            parse(try_from_str)
        )]
        scale: Scale,
        station_name: String,
    },
}

#[derive(Debug, Deserialize)]
struct WeatherResponse<T> {
    status: String,
    data: T,
}
#[derive(Debug, Deserialize)]
struct WeatherData {
    aqi: u32,
    idx: u32,
    attributions: Vec<Attribution>,
    city: City,
    dominentpol: String,
    iaqi: HashMap<String, HashMap<String, f32>>,
    forecast: Forecast,
    debug: SyncDebug,
}
#[derive(Debug, Deserialize)]

struct Attribution {
    url: String,
    name: String,
}
#[derive(Debug, Deserialize)]
struct City {
    geo: Vec<f32>,
    name: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct Time {
    s: String,
    tz: String,
    v: u32,
    iso: String,
}
// enum Measurement(f32);
#[derive(Debug, Deserialize)]
struct SyncDebug {
    sync: String,
}
#[derive(Debug, Deserialize)]
struct Forecast {
    daily: Option<Daily>,
}
#[derive(Debug, Deserialize)]
struct Daily {
    o3: Vec<ForecastStats>,
    pm10: Vec<ForecastStats>,
    pm25: Vec<ForecastStats>,
    uvi: Vec<ForecastStats>,
}
#[derive(Debug, Deserialize)]
struct ForecastStats {
    avg: u32,
    day: String,
    max: u32,
    min: u32,
}

#[derive(Debug, Deserialize)]
struct StationMeta {
    aqi: String,
    station: City,
    time: STime,
    uid: u32,
}

#[derive(Debug, Deserialize)]
struct STime {
    stime: String,
    tz: String,
    vtime: u32,
}
