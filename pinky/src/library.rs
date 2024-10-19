use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use rusqlite::{Connection, Error};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::{Frontmatter, Schema};

pub struct Library {
    pub path: PathBuf,
    pub config: LibraryConfig,
    db_conn: Connection,
}

#[derive(Serialize, Deserialize)]
pub struct LibraryConfig {
    pub schema_dir: String,
    pub default_schema: String,
}

impl Library {
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
        config_path.push(format!("{name}.yaml"));
        let config_string = fs::read_to_string(config_path).unwrap();
        let config = serde_yaml::from_str::<LibraryConfig>(&config_string).unwrap();

        let mut db_conn_path = path.clone();
        db_conn_path.push(format!("{name}.db"));
        let db_conn = Connection::open(db_conn_path).unwrap();

        Self {
            path,
            config,
            db_conn,
        }
    }

    pub fn new_page(&self, schema: Option<String>) -> PathBuf {
        let mut schema_dir_path = self.path.clone();
        schema_dir_path.push(self.config.schema_dir.clone());
        let schema_dir = fs::read_dir(schema_dir_path).unwrap();

        let mut page_schema = None;
        let schema = schema.unwrap_or(self.config.default_schema.clone());
        for entry in schema_dir {
            if let Ok(e) = entry {
                if e.path().file_stem().unwrap().to_str().unwrap() == schema {
                    let schema_string = fs::read_to_string(e.path()).unwrap();
                    page_schema = Some(Schema::from_yaml_string(schema_string));
                    break;
                }
            }
        }
        let page_schema = page_schema.unwrap();
        //
        let page_id = Uuid::new_v4();
        let frontmatter = Frontmatter::new(page_id, &page_schema);

        let mut page_path = self.path.clone();
        page_path.push(format!("{}.{}.md", page_schema.name, page_id));
        fs::write(
            &page_path,
            String::from_iter(["---\n", &frontmatter.to_yaml_string(), "---"]),
        )
        .unwrap();

        page_path
    }

    pub fn new_schema(&self, name: &str) -> PathBuf {
        let mut schema_path = self.path.clone();
        schema_path.push(self.config.schema_dir.clone());
        schema_path.push(format!("{name}.yaml"));

        fs::write(&schema_path, format!("name: {name}\nproperties:")).unwrap();
        schema_path
    }

    pub fn commit_page(&self, page_path: PathBuf) {
        let file_name_path = page_path.clone();
        let mut delim_iter = file_name_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .split(".");
        let schema_name = delim_iter.next().unwrap();
        let id = delim_iter.next().unwrap();

        let page_file = File::open(page_path).unwrap();
        let frontmatter = Frontmatter::from_file(&page_file);

        // TODO: change these to errors at some point
        assert!(id == frontmatter.id);
        assert!(schema_name == frontmatter.schema);

        let mut schema_dir_path = self.path.clone();
        schema_dir_path.push(self.config.schema_dir.clone());
        let schema_dir = fs::read_dir(schema_dir_path).unwrap();

        let mut schema = None;
        for entry in schema_dir {
            if let Ok(e) = entry {
                if e.path().file_stem().unwrap().to_str().unwrap() == schema_name {
                    let schema_string = fs::read_to_string(e.path()).unwrap();
                    schema = Some(Schema::from_yaml_string(schema_string));
                    break;
                }
            }
        }

        let schema = schema.unwrap();
        match frontmatter.is_valid(&schema) {
            true => {
                self.db_conn
                    .execute(&frontmatter.sql_commit_string(), ())
                    .unwrap();
            }
            false => todo!(),
        }
    }

    pub fn commit_schema(&self, schema_path: PathBuf) -> Result<usize, rusqlite::Error> {
        let schema_string = fs::read_to_string(schema_path).unwrap();
        let schema = Schema::from_yaml_string(schema_string);
        Ok(self.db_conn.execute(&schema.sql_commit_string(), ())?)
    }
}
