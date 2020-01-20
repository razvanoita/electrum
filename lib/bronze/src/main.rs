extern crate rusqlite;

use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::ops::FnMut;
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::collections::HashSet;
use std::time;

use console::{style, Term};
use dialoguer::{theme::CustomPromptCharacterTheme, Input};

use notify::{RecommendedWatcher, RecursiveMode, Result, Watcher};

use rusqlite::*;

use std::sync::mpsc::channel;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ShaderType {
    VertexShader,
    FragmentShader,
    UnsupportedShader,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum BuildStatus {
    AssetBuilt,
    AssetNotBuilt,
    Unknown,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum CacheStatus {
    OutOfDate,
    UpToDate,
    Unknown,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum AssetType {
    Shader,
    FrameGraph,
}

impl fmt::Display for BuildStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BuildStatus::AssetBuilt => write!(f, "Built"),
            BuildStatus::AssetNotBuilt => write!(f, "Not Built"),
            _ => write!(f, "Unknown build status...CacheEntry"),
        }
    }
}

impl fmt::Display for CacheStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CacheStatus::OutOfDate => write!(f, "Was out of date"),
            CacheStatus::UpToDate => write!(f, "Up to date"),
            _ => write!(f, "Unknown cache status..."),
        }
    }
}

#[derive(Clone)]
pub struct Asset {
    pub asset_name: String,
    pub output_name: String,
    pub build_status: BuildStatus,
    pub cache_status: CacheStatus,
    pub asset_type: AssetType,
    pub modified: i64,
}

impl Asset {
    pub fn update_status(&mut self, status: BuildStatus) {
        self.build_status = status
    }
}

pub struct CacheEntry {
    pub id: i64,
    pub name: String,
    pub modified: i64,
}

fn initialize_db() {
    let conn = Connection::open("assets.db").unwrap();

    conn.execute(
        "create table if not exists assets (
             id integer primary key,
             name text not null unique,
             modified integer not null
         )",
        NO_PARAMS,
    );

    conn.close();
}

fn get_cached_assets() -> Vec<CacheEntry> {
    let conn = Connection::open("assets.db").unwrap();

    let mut stmt = conn.prepare("SELECT * FROM assets").unwrap();
    let rows = stmt
        .query_map(NO_PARAMS, |row| {
            Ok(CacheEntry {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                modified: row.get(2).unwrap(),
            })
        })
        .unwrap();

    let mut cached_assets: Vec<CacheEntry> = Vec::new();
    for entry in rows {
        cached_assets.push(entry.unwrap());
    }

    cached_assets
}

fn get_shader_assets(src_path: String) -> Vec<Asset> {
    let paths = fs::read_dir(src_path).unwrap();
    let mut shader_assets: Vec<Asset> = vec![];

    for path in paths {
        let entry: fs::DirEntry = path.unwrap();
        let pathbuf = entry.path();
        let extension = pathbuf.extension().unwrap().to_str().unwrap();
        let shader_type = if extension == "vert" {
            ShaderType::VertexShader
        } else if extension == "frag" {
            ShaderType::FragmentShader
        } else {
            ShaderType::UnsupportedShader
        };

        if shader_type == ShaderType::UnsupportedShader {
            continue;
        }

        let metadata = fs::metadata(entry.path()).unwrap();
        let time_modified = metadata
            .modified()
            .unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let file_name = String::from(entry.file_name().to_str().unwrap());
        let shader_name_split: Vec<&str> = file_name.split('.').collect();
        let shader_spv_name = String::from(shader_name_split[0]) + "_" + extension + ".spv";

        let mut shader_asset = Asset {
            asset_name: file_name,
            output_name: shader_spv_name.clone(),
            build_status: BuildStatus::Unknown,
            cache_status: CacheStatus::Unknown,
            asset_type: AssetType::Shader,
            modified: time_modified as i64,
        };

        let shader_spv_pathbuf =
            String::from("../../copper/shaders/bin/") + shader_spv_name.as_str();
        let shader_spv_path = std::path::Path::new(shader_spv_pathbuf.as_str());
        if (shader_spv_path.exists()) {
            shader_asset.update_status(BuildStatus::AssetBuilt);
        }

        shader_assets.push(shader_asset);
    }

    shader_assets
}

fn populate_cache(assets: &Vec<Asset>) {
    let conn = Connection::open("assets.db").unwrap();

    for asset in assets {
        let command: String = format!(
            "INSERT INTO assets (name, modified) VALUES ('{}', {})",
            &asset.asset_name, &asset.modified
        );
        conn.execute(&command, NO_PARAMS).unwrap();
    }
    conn.close();
}

fn check_against_cache(local_assets: &mut Vec<Asset>, cached_assets: &Vec<CacheEntry>) {
    let conn = Connection::open("assets.db").unwrap();

    for asset in local_assets {
        let cache_entry = cached_assets
            .iter()
            .find(|entry| entry.name == asset.asset_name);
        if cache_entry.is_none() {
            let command: String = format!(
                "INSERT INTO assets (name, modified) VALUES ('{}', {})",
                &asset.asset_name, &asset.modified
            );
            conn.execute(&command, NO_PARAMS).unwrap();
            asset.cache_status = CacheStatus::UpToDate;
            asset.build_status = BuildStatus::AssetNotBuilt;
        } else {
            if asset.modified > cache_entry.unwrap().modified {
                asset.cache_status = CacheStatus::OutOfDate;
                asset.build_status = BuildStatus::AssetNotBuilt;
            } else if asset.modified == cache_entry.unwrap().modified {
                asset.cache_status = CacheStatus::UpToDate;
            }
        }
    }

    conn.close();
}

fn build_shader(
    src_path: String,
    bin_path: String,
    asset_name: String,
    output_name: String,
) -> std::process::Output {
    let output = Command::new("../../copper/shaders/glslangValidator")
        .arg("-V")
        .arg(src_path + "/" + &asset_name.as_str())
        .arg("-o")
        .arg(bin_path.clone() + "/" + &output_name.as_str())
        .output()
        .expect("failed to compile shader!");

    output
}

fn run_server() -> io::Result<()> {
    initialize_db();

    let cached_assets = get_cached_assets();
    let cache_empty = cached_assets.is_empty();

    let shader_asset_src_path: String = String::from("../../copper/shaders/src");
    let shader_asset_bin_path: String = String::from("../../copper/shaders/bin");
    let glslangvalidator_path: String = String::from("../../copper/shaders/glslangValidator.exe");

    let shader_assets = get_shader_assets(shader_asset_src_path.clone());
    let mut local_assets: Vec<Asset> = vec![];
    local_assets.extend(shader_assets.iter().cloned());

    if (cache_empty) {
        populate_cache(&local_assets);
    } else {
        check_against_cache(&mut local_assets, &cached_assets);
    }

    let terminal = Term::stdout();
    terminal.set_title("AssetServer");
    terminal.write_line("Retrieving shader assets")?;

    for asset in &local_assets {
        terminal.write_line(&format!(
            "{}[{}][{}][{}]",
            style(asset.asset_name.clone()).white(),
            style(asset.output_name.clone()).cyan(),
            style(asset.cache_status).green(),
            style(asset.build_status).green()
        ))?;
    }

    let mut num_built_assets: u32 = 0;
    for (i, asset) in local_assets.iter().enumerate() {
        if (asset.build_status != BuildStatus::AssetBuilt) {
            terminal.write_line(&format!(
                "Building: {}",
                style(asset.asset_name.clone()).cyan()
            ));
            let output = build_shader(
                shader_asset_src_path.clone(),
                shader_asset_bin_path.clone(),
                asset.asset_name.clone(),
                asset.output_name.clone(),
            );

            if (output.status.success()) {
                terminal.write_line(&format!(
                    "{} produced output: {}",
                    style("Build succesful!").green(),
                    style(asset.output_name.clone()).cyan()
                ));

                if (asset.cache_status == CacheStatus::OutOfDate) {
                    let conn = Connection::open("assets.db").unwrap();
                    let command: String = format!(
                        "UPDATE assets SET modified={} WHERE name ='{}'",
                        &asset.modified, &asset.asset_name
                    );
                    conn.execute(&command, NO_PARAMS).unwrap();
                    conn.close();

                    terminal.write_line(&format!(
                        "{} was out of date. Cache updated.",
                        style(asset.output_name.clone()).cyan()
                    ));
                }

                num_built_assets += 1;
            } else {
                terminal.write_line(&format!("{}", style("Build failed!").red()));
                terminal.write_line(&format!(
                    "stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
    }
    terminal.write_line(&format!("Built {} shaders!", num_built_assets));
    let watcher_terminal = terminal.clone();
    let mut watcher_paths: HashSet<std::path::PathBuf> = HashSet::new();

    let (sender, receiver) = channel::<std::path::PathBuf>();

    let mut watcher: RecommendedWatcher =
        Watcher::new_immediate(move |res: Result<notify::Event>| match res {
            Ok(event) => {
                sender.send(event.paths.first().unwrap().clone());
            }
            Err(e) => {
                watcher_terminal.write_line(&format!("Watch error: {:?}", e));
            }
        })
        .unwrap();

    watcher
        .watch(shader_asset_src_path.clone(), RecursiveMode::Recursive)
        .unwrap();

    let mut event_collecter: Vec<std::path::PathBuf> = Vec::new();
    const NUM_EVENTS_MODIFIED: usize = 3;

    let theme = CustomPromptCharacterTheme::new('>');
    loop {
        let event = receiver.try_recv();
        if event.is_ok() {
            event_collecter.push(event.unwrap());
        } else if event_collecter.len() == NUM_EVENTS_MODIFIED {
            let path = event_collecter.pop().unwrap();
            event_collecter.clear();

            let sleep_duration = time::Duration::from_millis(500);
            thread::sleep(sleep_duration);

            terminal.write_line(&format!("Asset {} touched", path.to_str().unwrap()));

            let asset_name = String::from(path.file_name().unwrap().to_str().unwrap());
            let metadata = fs::metadata(path).unwrap();
            let time_modified = metadata
                .modified()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let output_name = local_assets
                .iter()
                .find(|&asset| asset.asset_name == asset_name)
                .unwrap()
                .output_name
                .clone();

            let output = build_shader(
                shader_asset_src_path.clone(),
                shader_asset_bin_path.clone(),
                asset_name.clone(),
                output_name.clone(),
            );

            if (output.status.success()) {
                terminal.write_line(&format!(
                    "{} produced output: {}",
                    style("Build succesful!").green(),
                    style(output_name.clone()).cyan()
                ));

                let conn = Connection::open("assets.db").unwrap();
                let command: String = format!(
                    "UPDATE assets SET modified={} WHERE name ='{}'",
                    &time_modified, &asset_name
                );
                conn.execute(&command, NO_PARAMS).unwrap();
                conn.close();

                terminal.write_line(&format!(
                    "{} was out of date. Cache updated.",
                    style(output_name.clone()).cyan()
                ));
            }
            else {
                terminal.write_line(&format!(
                    "{} did not produce output: {}",
                    style("Build failed!").red(),
                    style(output_name.clone()).cyan()
                ));
                terminal.write_line(&format!(
                    "stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
    }

    Ok(())
}

fn main() {
    run_server().unwrap()
}
