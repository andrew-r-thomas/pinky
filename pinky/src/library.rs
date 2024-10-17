use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::{Property, Schema};

pub struct Library {
    path: PathBuf,
    config: LibraryConfig,
    db_conn: Connection,
}

#[derive(Serialize, Deserialize)]
pub struct LibraryConfig {
    schema_dir: String,
    default_schema: String,
}

pub struct Frontmatter {
    id: String,
    schema: String,
    properties: Vec<Property>,
}

impl Frontmatter {
    pub fn new(id: Uuid, schema: &Schema) -> Self {
        todo!()
    }

    pub fn to_yaml_string(&self) -> String {
        todo!()
    }

    pub fn from_file(file: &File) -> Self {
        todo!()
    }

    pub fn to_sql_upsert(&self) -> String {
        todo!()
    }
}

impl Library {
    pub fn new(path: &Path, schema_dir: Option<String>, default_schema: Option<Schema>) -> Self {
        // get path and name of lib from path
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

        // make the lib dir
        fs::create_dir(path.clone()).unwrap();

        // get the schema dir and default schema from args, make config from that
        let schema_dir = schema_dir.unwrap_or("schemas".into());
        let default_schema = default_schema.unwrap_or(Schema::default());
        let config = LibraryConfig {
            schema_dir,
            default_schema: default_schema.name.clone(),
        };

        // write the config
        let mut config_path = path.clone();
        config_path.push(format!("{name}.yaml"));
        fs::write(
            config_path,
            serde_yaml::to_string(&config).unwrap().as_bytes(),
        )
        .unwrap();

        // set up schema dir and default schema, write them out
        let mut schema_dir_path = path.clone();
        schema_dir_path.push(config.schema_dir.clone());
        fs::create_dir(schema_dir_path.clone()).unwrap();
        let mut default_schema_path = schema_dir_path.clone();
        default_schema_path.push(format!("{}.yaml", default_schema.name));
        fs::write(default_schema_path, default_schema.to_yaml_string()).unwrap();

        // set up db, define default schema
        let mut db_conn_path = path.clone();
        db_conn_path.push(format!("{name}.db"));
        let db_conn = Connection::open(db_conn_path).unwrap();
        db_conn.execute(&default_schema.table_def(), ()).unwrap();

        Self {
            path,
            config,
            db_conn,
        }
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

    pub fn new_page(&mut self, schema: Option<String>) -> PathBuf {
        let mut schema_dir_path = self.path.clone();
        schema_dir_path.push(self.config.schema_dir.clone());
        let schema_dir = fs::read_dir(schema_dir_path).unwrap();

        let mut page_schema = None;
        let schema = schema.unwrap_or(self.config.default_schema.clone());
        for entry in schema_dir {
            if let Ok(e) = entry {
                if e.file_name().to_str().unwrap() == schema {
                    let schema_string = fs::read_to_string(e.path()).unwrap();
                    page_schema = Some(Schema::from_yaml_string(schema_string));
                    break;
                }
            }
        }
        let page_schema = page_schema.unwrap();

        let page_id = Uuid::new_v4();
        let frontmatter = Frontmatter::new(page_id, &page_schema);

        let mut page_path = self.path.clone();
        page_path.push(format!("{}.{}.md", page_schema.name, page_id));
        fs::write(&page_path, frontmatter.to_yaml_string()).unwrap();

        page_path
    }

    pub fn new_schema(&mut self, name: String) -> PathBuf {
        todo!()
    }

    pub fn commit_page(&mut self, file_name: String) {
        let mut page_path = self.path.clone();
        page_path.push(file_name.clone());

        let mut delim_iter = file_name.split(".");
        let schema_name = delim_iter.next().unwrap();
        let id = delim_iter.next().unwrap();

        let page_file = File::open(page_path).unwrap();
        let frontmatter = Frontmatter::from_file(&page_file);

        assert!(id == frontmatter.id);
        assert!(schema_name == frontmatter.schema);

        let mut schema_dir_path = self.path.clone();
        schema_dir_path.push(self.config.schema_dir.clone());
        let schema_dir = fs::read_dir(schema_dir_path).unwrap();

        let mut schema = None;
        for entry in schema_dir {
            if let Ok(e) = entry {
                if e.file_name().to_str().unwrap() == schema_name {
                    let schema_string = fs::read_to_string(e.path()).unwrap();
                    schema = Some(Schema::from_yaml_string(schema_string));
                    break;
                }
            }
        }

        let schema = schema.unwrap();
        match schema.is_valid(&frontmatter) {
            true => {
                self.db_conn
                    .execute(&frontmatter.to_sql_upsert(), ())
                    .unwrap();
            }
            false => todo!(),
        }
    }

    pub fn commit_schema(&mut self) {
        todo!()
    }
}
