

use std::fmt;

use std::fs;

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use console::{style, Term};

use dialoguer::{theme::CustomPromptCharacterTheme, Input};

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
}

impl fmt::Display for BuildStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match *self {
           BuildStatus::AssetBuilt => write!(f, "Built"),
           BuildStatus::AssetNotBuilt => write!(f, "Not Built"),
       }
    }
}

pub struct Asset {
    pub asset_name: String,
    pub output_name: String,
    pub status: BuildStatus
}

impl Asset {
    pub fn update_status(&mut self, status: BuildStatus) {
        self.status = status
    }
}

fn run_server() -> io::Result<()> {
    let terminal = Term::stdout();
    terminal.set_title("AssetServer");
    terminal.write_line("Retrieving shader assets")?;

    let shader_asset_src_path: String = String::from("../../copper/shaders/src");
    let shader_asset_bin_path: String = String::from("../../copper/shaders/bin");
    let paths = fs::read_dir(shader_asset_src_path).unwrap();
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

        let file_name = String::from(entry.file_name().to_str().unwrap());
        let shader_name_split: Vec<&str> = file_name.split('.').collect();
        let shader_spv_name = String::from(shader_name_split[0]) + "_" + extension + ".spv";

        let mut shader_asset = Asset {
            asset_name: file_name,
            output_name: shader_spv_name.clone(),
            status: BuildStatus::AssetNotBuilt
        };

        let shader_spv_pathbuf = String::from("../../copper/shaders/bin/") + shader_spv_name.as_str();
        let shader_spv_path = std::path::Path::new(shader_spv_pathbuf.as_str());
        if (shader_spv_path.exists()) {
            shader_asset.update_status(BuildStatus::AssetBuilt);
        }

        terminal.write_line(&format!(
            "{}[{}:{}]", 
            style(shader_asset.asset_name.clone()).white(),
            style(shader_asset.output_name.clone()).cyan(),
            style(shader_asset.status).green()
        ))?;

        shader_assets.push(shader_asset);
    }

    let mut num_built_assets: u32 = 0;
    for (i, asset) in shader_assets.iter().enumerate() {
        if (asset.status != BuildStatus::AssetBuilt) {
            num_built_assets += 1;
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