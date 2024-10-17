use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
struct PinkyArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    New {
        #[command(subcommand)]
        new_what: NewWhat,
    },
}

#[derive(Subcommand)]
enum NewWhat {
    Lib {
        #[arg(short, long)]
        path: PathBuf,

        #[arg(short, long)]
        name: Option<String>,
    },
    Note {},
}

#[derive(Serialize, Deserialize)]
struct GlobalConfig {
    default_lib: Option<String>,
    default_editor_command: Option<String>,
}

#[derive(Serialize)]
struct LibConfig {
    name: String,
}

// TODO:
const TABLE_SETUP: &'static str = "";
// TODO: figure out config path
const GLOBAL_CONFIG_PATH: &'static str = "/Users/andrewthomas/.pinky/pinky.toml";

fn main() {
    let args = PinkyArgs::parse();

    let global_config_string = match fs::read_to_string(GLOBAL_CONFIG_PATH) {
        Ok(f) => f,
        Err(e) => {
            println!("global config error: {e}");
            return;
        }
    };

    let mut global_config = match toml::from_str::<GlobalConfig>(&global_config_string) {
        Ok(c) => c,
        Err(e) => {
            println!("global config toml error: {e}");
            return;
        }
    };

    // TODO: figure out checks for initialization, or figure out if we wanna do an init

    match args.command {
        Command::New { new_what } => match new_what {
            NewWhat::Lib { path, name } => {
                if let Err(e) = fs::create_dir(path.clone()) {
                    println!("new lib dir error: {e}");
                    return;
                }

                let lib_name = match name {
                    Some(n) => n,
                    None => String::from(
                        // this is dumb lol
                        path.clone()
                            .components()
                            .rev()
                            .next()
                            .unwrap()
                            .as_os_str()
                            .to_str()
                            .unwrap(),
                    ),
                };

                {
                    let lib_config = LibConfig {
                        name: lib_name.clone(),
                    };
                    let mut lib_config_path = path.clone();
                    lib_config_path.push(format!("{lib_name}.toml"));
                    fs::write(
                        lib_config_path,
                        toml::to_string(&lib_config).unwrap().as_bytes(),
                    )
                    .unwrap();
                }

                {
                    let mut db_path = path.clone();
                    db_path.push(format!("{lib_name}.db"));
                    let db_conn = Connection::open(db_path).unwrap();

                    db_conn.execute_batch(&TABLE_SETUP).unwrap();
                }

                if let None = global_config.default_lib {
                    global_config.default_lib = Some(path.to_str().unwrap().to_string());

                    fs::write(
                        GLOBAL_CONFIG_PATH,
                        toml::to_string(&global_config).unwrap().as_bytes(),
                    )
                    .unwrap();
                }

                println!(
                    "new lib \"{}\" created in {}",
                    lib_name,
                    path.to_str().unwrap()
                );
            }
            NewWhat::Note {} => {
                println!("new note called");
            }
        },
    }
}
