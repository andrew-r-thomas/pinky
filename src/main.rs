use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use rusqlite::{params, Connection};
use serde::Serialize;

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
    Lib {},
    Note {},
}

#[derive(Serialize)]
struct GlobalConfig {
    default_lib: String,
    default_editor_command: String,
}

#[derive(Serialize)]
struct LibConfig {
    name: String,
}

fn main() {
    let args = PinkyArgs::parse();

    let global_config_file = match File::open("~/.pinky/pinky.toml") {
        Ok(f) => f,
        Err(e) => {
            println!("global config error: {e}");
            return;
        }
    };

    let db_conn = match Connection::open("~/.pinky/pinky.db") {
        Ok(c) => c,
        Err(e) => {
            println!("db connection error: {e}");
            return;
        }
    };

    // TODO: figure out checks for initialization, or figure out if we wanna do an init

    match args.command {
        Command::New { new_what } => match new_what {
            NewWhat::Lib {} => {}
            NewWhat::Note {} => {}
        },
    }
}
