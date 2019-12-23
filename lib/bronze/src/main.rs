
use std::string;
use std::fs;

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use console::{style, Term};

use dialoguer::{theme::CustomPromptCharacterTheme, Input};

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ShaderType {
    VertexShader,
    FragmentShader
}

pub fn get_shader_type_str(t: ShaderType) -> String {
    if t == ShaderType::VertexShader {
        String::from("vertex shader")
    } else {
        String::from("fragment shader")
    }
}

fn run_server() -> io::Result<()> {
    let terminal = Term::stdout();
    terminal.set_title("AssetServer");
    terminal.write_line("Retrieving shader assets")?;

    let shader_asset_path: String = String::from("../../copper/shaders");
    let paths = fs::read_dir(shader_asset_path).unwrap();

    for path in paths {
        let entry: fs::DirEntry = path.unwrap();
        let pathbuf = entry.path();
        let extension = pathbuf.extension().unwrap().to_str().unwrap();
        let shader_type = if let "vert" = extension { ShaderType::VertexShader } else { ShaderType::FragmentShader };

        terminal.write_line(&format!("Shader[{}]: {}", style(get_shader_type_str(shader_type)).cyan(), style(entry.file_name().to_str().unwrap()).cyan()))?;
    }

    // let term = Term::stdout();
    // term.set_title("Counting...");
    // term.write_line("Going to do some counting now")?;
    // for x in 0..10 {
    //     if x != 0 {
    //         term.move_cursor_up(1)?;
    //     }
    //     term.write_line(&format!("Counting {}/10", style(x + 1).cyan()))?;
    //     thread::sleep(Duration::from_millis(400));
    // }
    // term.clear_last_lines(1)?;
    // term.write_line("Done counting!")?;
    // writeln!(&term, "Hello World!")?;

    // write!(&term, "To edit: ")?;
    // let res = term.read_line_initial_text("default")?;
    // writeln!(&term, "\n{}", res)?;

    Ok(())
}

fn main() {
    run_server().unwrap()

    // do_stuff().unwrap();

    // let theme = CustomPromptCharacterTheme::new('>');
    // let input: String = Input::with_theme(&theme)
    //     .with_prompt("Your name")
    //     .interact()
    //     .unwrap();
    // println!("Hello {}!", input);
}