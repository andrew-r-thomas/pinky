use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use toml::Table;
use uuid::Uuid;

pub struct Pinky {
    config: PinkyConfig,
    lib: Option<Library>,
}

pub struct PinkyConfig {
    default_lib: Option<PathBuf>,
}

impl Pinky {
    pub fn new(config: PinkyConfig) -> Self {
        let lib = match &config.default_lib {
            Some(p) => Some(Library::open(p)),
            None => None,
        };

        Self { config, lib }
    }

    pub fn new_lib(&mut self, path: &Path, set_as_current: bool, config: Option<LibraryConfig>) {
        let lib = Library::new(path, config);
        if set_as_current {
            self.lib = Some(lib);
        }
    }

    pub fn new_note(&mut self, lib: Option<PathBuf>, type_: Option<String>) {
        match lib {
            Some(l) => {
                let mut lib = Library::open(&l);
                lib.new_note(type_);
            }
            None => {
                self.lib.as_mut().unwrap().new_note(type_);
            }
        }
    }
}

pub struct Library {
    path: PathBuf,
    config: LibraryConfig,
}

#[derive(Serialize, Deserialize)]
pub struct LibraryConfig {
    type_dir: Option<String>,
    default_type: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Frontmatter {
    id: String,
    type_: String,
    props: Option<Table>,
}

impl Library {
    pub fn new(path: &Path, config: Option<LibraryConfig>) -> Self {
        let path = path.to_path_buf();
        let name = path
            .components()
            .rev()
            .next()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();

        fs::create_dir(path.clone()).unwrap();

        let config = match config {
            Some(c) => c,
            None => LibraryConfig {
                type_dir: None,
                default_type: None,
            },
        };

        let mut config_path = path.clone();
        config_path.push(format!("{name}.toml"));

        fs::write(config_path, toml::to_string(&config).unwrap().as_bytes()).unwrap();

        if let Some(td) = &config.type_dir {
            let mut td_path = path.clone();
            td_path.push(td);
            fs::create_dir(td_path).unwrap();
        }

        Self { path, config }
    }

    pub fn open(path: &Path) -> Self {
        let path = path.to_path_buf();
        let name = path
            .components()
            .rev()
            .next()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();

        let mut config_path = path.clone();
        config_path.push(format!("{name}.toml"));
        let config_string = fs::read_to_string(config_path).unwrap();
        let config = toml::from_str::<LibraryConfig>(&config_string).unwrap();

        Self { path, config }
    }

    pub fn new_note(&mut self, type_: Option<String>) {
        let id = Uuid::new_v4();
        // TODO: fills
        let (contents, prefix) = match type_ {
            Some(t) => {
                let mut t_path = PathBuf::new();
                t_path.push(self.config.type_dir.as_ref().unwrap());
                t_path.push(format!("{t}.toml"));

                (fs::read(t_path).unwrap(), t)
            }
            None => match &self.config.default_type {
                Some(dt) => {
                    let mut dt_path = PathBuf::new();
                    dt_path.push(self.config.type_dir.as_ref().unwrap());
                    dt_path.push(format!("{dt}.toml"));

                    (fs::read(dt_path).unwrap(), dt.clone())
                }
                None => (Vec::new(), String::new()),
            },
        };

        let mut path = self.path.clone();
        path.push(format!("{prefix}.{id}.md"));

        fs::write(path, contents).unwrap();
    }
}
