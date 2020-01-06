
extern crate rusqlite;

use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::process::Command;
use std::ops::FnMut;

use console::{style, Term};
use dialoguer::{theme::CustomPromptCharacterTheme, Input};

use rusqlite::*;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ShaderType {
    VertexShader,
    FragmentShader,
    UnsupportedShader
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum BuildStatus {
    AssetBuilt,
    AssetNotBuilt,
    Unknown
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
            _ => write!(f, "WTF!")
       }
    }
}

pub struct Asset {
    pub asset_name: String,
    pub output_name: String,
    pub build_status: BuildStatus,
    pub cache_status: CacheStatus,
    pub asset_type: AssetType,
    pub modified: i64
}

impl Asset {
    pub fn update_status(&mut self, status: BuildStatus) {
        self.build_status = status
    }
}

pub struct CacheEntry {
    pub id: i64,
    pub name: String,
    pub modified: i64
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
    let rows = stmt.query_map(NO_PARAMS, |row|
        Ok(
            CacheEntry {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                modified: row.get(2).unwrap()
            }
        )
    ).unwrap();

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
        let time_modified = metadata.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        let file_name = String::from(entry.file_name().to_str().unwrap());
        let shader_name_split: Vec<&str> = file_name.split('.').collect();
        let shader_spv_name = String::from(shader_name_split[0]) + "_" + extension + ".spv";

        let mut shader_asset = Asset {
            asset_name: file_name,
            output_name: shader_spv_name.clone(),
            build_status: BuildStatus::Unknown,
            cache_status: CacheStatus::Unknown,
            asset_type: AssetType::Shader,
            modified: time_modified as i64
        };

        let shader_spv_pathbuf = String::from("../../copper/shaders/bin/") + shader_spv_name.as_str();
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
        let command:String = format!("INSERT INTO assets (name, modified) VALUES ('{}', {})", &asset.asset_name, &asset.modified);
        conn.execute(&command, NO_PARAMS).unwrap();
    }
    conn.close();
}

fn update_cache(local_assets: &Vec<Asset>, cached_assets: &Vec<CacheEntry>) {
    let conn = Connection::open("assets.db").unwrap();

    for asset in local_assets {
        let cache_entry = cached_assets.iter().find(|entry| entry.name == asset.asset_name);
        if cache_entry.is_none() {
            let command:String = format!("INSERT INTO assets (name, modified) VALUES ('{}', {})", &asset.asset_name, &asset.modified);
            conn.execute(&command, NO_PARAMS).unwrap();
        } else {
            
        }
    }
}

fn run_server() -> io::Result<()> {
    initialize_db();

    let cached_assets = get_cached_assets();
    let cache_empty = cached_assets.is_empty();

    let shader_asset_src_path: String = String::from("../../copper/shaders/src");
    let shader_asset_bin_path: String = String::from("../../copper/shaders/bin");
    let glslangvalidator_path: String = String::from("../../copper/shaders/glslangValidator.exe");

    let shader_assets = get_shader_assets(shader_asset_src_path.clone());

    if (cache_empty) {
        populate_cache(&shader_assets);
    } else {
        update_cache(&shader_assets, &cached_assets);
    }

    let terminal = Term::stdout();
    terminal.set_title("AssetServer");
    terminal.write_line("Retrieving shader assets")?;

    // let paths = fs::read_dir(shader_asset_src_path.clone()).unwrap();
    // let mut shader_assets: Vec<Asset> = vec![];

    // for path in paths {
    //     let entry: fs::DirEntry = path.unwrap();
    //     let pathbuf = entry.path();
    //     let extension = pathbuf.extension().unwrap().to_str().unwrap();
    //     let shader_type = if extension == "vert" { 
    //         ShaderType::VertexShader 
    //     } else if extension == "frag" {
    //         ShaderType::FragmentShader
    //     } else {
    //         ShaderType::UnsupportedShader
    //     };

    //     if shader_type == ShaderType::UnsupportedShader {
    //         continue;
    //     }

    //     let metadata = fs::metadata(entry.path())?;
    //     let time_modified = metadata.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

    //     let file_name = String::from(entry.file_name().to_str().unwrap());
    //     let shader_name_split: Vec<&str> = file_name.split('.').collect();
    //     let shader_spv_name = String::from(shader_name_split[0]) + "_" + extension + ".spv";

    //     let mut shader_asset = Asset {
    //         asset_name: file_name,
    //         output_name: shader_spv_name.clone(),
    //         status: BuildStatus::AssetNotBuilt
    //     };

    //     let shader_spv_pathbuf = String::from("../../copper/shaders/bin/") + shader_spv_name.as_str();
    //     let shader_spv_path = std::path::Path::new(shader_spv_pathbuf.as_str());
    //     if (shader_spv_path.exists()) {
    //         shader_asset.update_status(BuildStatus::AssetBuilt);
    //     }

    //     terminal.write_line(&format!(
    //         "{}[{}:{}]", 
    //         style(shader_asset.asset_name.clone()).white(),
    //         style(shader_asset.output_name.clone()).cyan(),
    //         style(shader_asset.status).green()
    //     ))?;

    //     shader_assets.push(shader_asset);
    // }

    let mut num_built_assets: u32 = 0;
    for (i, asset) in shader_assets.iter().enumerate() {
        if (asset.build_status != BuildStatus::AssetBuilt) {
            terminal.write_line(&format!("Building: {}", style(asset.asset_name.clone()).cyan()));
            let output = Command::new("../../copper/shaders/glslangValidator")
                .arg("-V")
                .arg(shader_asset_src_path.clone() + "/" + &asset.asset_name.as_str())
                .arg("-o")
                .arg(shader_asset_bin_path.clone() + "/" + &asset.output_name.as_str())
                .output()
                .expect("failed to compile shader!");

            if (output.status.success()) {
                terminal.write_line(&format!("{}.Produced output: {}", 
                    style("Build succesful!").green(), 
                    style(asset.output_name.clone()).cyan())
                );

                num_built_assets += 1;
            } else {
                terminal.write_line(&format!("{}", style("Build failed!").red()));
                terminal.write_line(&format!("stderr: {}", String::from_utf8_lossy(&output.stderr)));
            }
        }
    }
    terminal.write_line(&format!("Built {} shaders!", num_built_assets));

    let theme = CustomPromptCharacterTheme::new('>');
    let mut exit_requested = false;
    while (!exit_requested) {
        let input: String = Input::with_theme(&theme)
            .interact()
            .unwrap();
        if (input == "exit") {
            exit_requested = true;
        }
    }

    Ok(())
}

fn main() {
    run_server().unwrap()
}