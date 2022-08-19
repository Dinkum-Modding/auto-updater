use std::collections::HashMap;

use std::{
    error,
    fmt::Display,
    thread::sleep,
    time::{Duration, SystemTime},
};

use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SteamResponse {
    status: String,
    data: HashMap<i32, Data>,
}
#[derive(Deserialize, Debug)]
pub struct Data {
    depots: Depots,
}
#[derive(Deserialize, Debug)]
pub struct Depots {
    branches: HashMap<String, Branch>,
}

#[derive(Deserialize, Debug)]
pub struct Branch {
    buildid: String,
    #[serde(default = "default_timeupload")]
    timeupdated: String,
    #[serde(default = "default_description")]
    description: String,
}

fn default_description() -> String {
    "No description".to_string()
}

fn default_timeupload() -> String {
    i64::default().to_string()
}

struct Settings {
    app_id: i32,
    branch: String,
}

#[derive(Clone)]
struct BuildInfo {
    branch: String,
    build_id: i64,
    timestamp: i64,
    description: String,
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self {
            branch: "none".to_owned(),
            build_id: 0,
            timestamp: 0,
            description: default_description(),
        }
    }
}

impl Display for BuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Branch: {};\n\tBuild ID: {};\n\tTimestamp: {};\n\tDescription: {}",
            self.branch, self.build_id, self.timestamp, self.description
        )
    }
}

#[derive(Clone)]
struct State {
    curr_build: Option<BuildInfo>,
    prev_build: Option<BuildInfo>,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Current build:\n\t{}\nPrevious build:\n\t{}",
            self.curr_build.as_ref().unwrap_or(&BuildInfo::default()),
            self.prev_build.as_ref().unwrap_or(&BuildInfo::default()),
        )
    }
}

async fn get_build_info(settings: &Settings) -> Result<BuildInfo, Box<dyn error::Error>> {
    let req = reqwest::get(format!(
        "https://api.steamcmd.net/v1/info/{}",
        settings.app_id
    ))
    .await?;

    let resp = req.json::<SteamResponse>().await?;
    let app_data = resp.data.get(&settings.app_id).expect("failed to get data");

    let branch = app_data
        .depots
        .branches
        .get(&settings.branch)
        .expect("failed to get branch");
    let build_id = branch.buildid.parse::<i64>()?;
    let timestamp = branch.timeupdated.parse::<i64>()?;
    let description = branch.description.to_owned();

    Ok(BuildInfo {
        branch: settings.branch.as_str().to_owned(),
        build_id,
        timestamp,
        description,
    })
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let app_id = args
        .get(1)
        .map(|e| e.to_owned())
        .unwrap_or("420".to_owned())
        .parse::<i32>()
        .unwrap();
    let branch = args
        .get(2)
        .map(|e| e.to_owned())
        .unwrap_or("public".to_owned())
        .to_owned();

    let settings = Settings { app_id, branch };

    let mut state = State {
        curr_build: None,
        prev_build: None,
    };

    loop {
        let sys_time = SystemTime::now();
        let datetime: DateTime<Utc> = sys_time.into();
        println!("[{}] Check started", datetime.format("%d/%m/%Y %T"));

        let build_info = get_build_info(&settings).await.expect("faild to get info");
        let curr_build = state.curr_build.clone();

        if curr_build.is_none() {
            state.curr_build = Some(build_info)
        } else if curr_build.unwrap().build_id != build_info.build_id {
            state.prev_build = state.curr_build;
            state.curr_build = Some(build_info);
        }

        println!("{}", state);
        sleep(Duration::new(30, 0));
    }
}
