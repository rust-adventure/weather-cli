use owo_colors::FgDynColorDisplay;
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
            let response = client
                .get("https://api.waqi.info/search/")
                .query(&[("token", &opt.api_token), ("keyword", &search_term)])
                .send()
                .await?;

            match opt.output {
                OutputFormat::Human => {
                    let resp = response.json::<WeatherResponse<Vec<StationMeta>>>().await?;
                    for station_meta in resp.data.iter() {
                        println!("{}", station_meta)
                    }
                }
                OutputFormat::Json => {
                    let resp = response
                        .json::<WeatherResponse<serde_json::Value>>()
                        .await?;
                    println!("{}", resp.data);
                }
                OutputFormat::Url => {
                    let resp = response.json::<WeatherResponse<Vec<StationMeta>>>().await?;
                    for station_meta in resp.data.iter() {
                        println!("{}", station_meta.station.url)
                    }
                }
                OutputFormat::Debug => {
                    let resp = response.json::<WeatherResponse<Vec<StationMeta>>>().await?;
                    dbg!("{:?}", resp);
                }
            }

            Ok(())
        }
        Opt::Info {
            scale,
            station_name,
        } => {
            let response = client
                .get(format!("https://api.waqi.info/feed/{}/", station_name))
                .query(&[("token", opt.api_token)])
                .send()
                .await?;
            match opt.output {
                OutputFormat::Human => {
                    let resp = response.json::<WeatherResponse<WeatherData>>().await?;
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
                }
                OutputFormat::Json => {
                    let resp = response
                        .json::<WeatherResponse<serde_json::Value>>()
                        .await?;
                    println!("{}", resp.data);
                }
                OutputFormat::Url => {}
                OutputFormat::Debug => {}
            }

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

#[derive(Debug)]
enum OutputFormat {
    Human,
    Json,
    Url,
    Debug,
}
#[derive(Debug)]
struct InvalidOutputFormat;
impl fmt::Display for InvalidOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid output format")
    }
}

impl FromStr for OutputFormat {
    type Err = InvalidOutputFormat;
    fn from_str(temp: &str) -> Result<Self, Self::Err> {
        match temp {
            "h" => Ok(OutputFormat::Human),
            "human" => Ok(OutputFormat::Human),
            "json" => Ok(OutputFormat::Json),
            "url" => Ok(OutputFormat::Url),
            "d" => Ok(OutputFormat::Debug),
            "debug" => Ok(OutputFormat::Debug),
            _ => Err(InvalidOutputFormat),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "aqi", about = "Find weather quality from stations near you")]
struct AQI {
    #[structopt(short = "t", long = "token", env = "API_TOKEN")]
    api_token: String,

    #[structopt(short = "o", long = "output", default_value = "human")]
    output: OutputFormat,

    #[structopt(subcommand)]
    command: Opt,
}
#[derive(Debug, StructOpt)]
enum Opt {
    Search {
        search_term: String,
    },
    Info {
        #[structopt(short = "s", long = "scale", default_value = "fahrenheit")]
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

impl fmt::Display for WeatherData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let aqi = format!(
            "{}",
            match 0 {
                0..=50 => self.aqi.color(CssColors::Green),
                51..=100 => self.aqi.color(CssColors::Yellow),
                101..=150 => self.aqi.color(CssColors::Orange),
                151..=200 => self.aqi.color(CssColors::Red),
                201..=300 => self.aqi.color(CssColors::Purple),
                301..=500 => self.aqi.color(CssColors::Maroon),
                _ => self.aqi.color(CssColors::White),
            }
        );

        write!(
            f,
            "{}\n{}\naqi: {}",
            self.city.name.blue().bold(),
            self.city.url,
            aqi
        )
    }
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

impl fmt::Display for StationMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let aqi = match self.aqi.parse::<u32>() {
            Ok(num) => format!(
                "{}",
                match 0 {
                    0..=50 => num.color(CssColors::Green),
                    51..=100 => num.color(CssColors::Yellow),
                    101..=150 => num.color(CssColors::Orange),
                    151..=200 => num.color(CssColors::Red),
                    201..=300 => num.color(CssColors::Purple),
                    301..=500 => num.color(CssColors::Maroon),
                    _ => num.color(CssColors::White),
                }
            ),
            Err(_) => format!("{}", self.aqi.white()),
        };
        write!(
            f,
            "{}\n{}\naqi: {}",
            self.station.name.blue().bold(),
            self.station.url,
            aqi
        )
    }
}
#[derive(Debug, Deserialize)]
struct STime {
    stime: String,
    tz: String,
    vtime: u32,
}

// fn format_aqi(num: u32) -> FgDynColorDisplay {
//     match num {
//         0..=50 => {
//             num.color(CssColors::Green)
//         }
//         51..=100 => {
//             num.color(CssColors::Yellow)
//         }
//         101..=150 => {
//             num.color(CssColors::Orange)
//         }
//         151..=200 => {
//             num.color(CssColors::Red)
//         }
//         201..=300 => {
//             num.color(CssColors::Purple)
//         }
//         301..=500 => {
//             num.color(CssColors::Maroon)
//         }
//         _ => {
//             num.color(CssColors::White)
//         }
// }
