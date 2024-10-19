use core::panic;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
};

use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub properties: BTreeMap<String, Property>,
}

impl Schema {
    pub fn from_yaml_string(string: String) -> Self {
        serde_yaml::from_str(&string).unwrap()
    }

    pub fn sql_commit_string(&self) -> String {
        let mut column_defs = String::new();
        column_defs.push_str("id TEXT PRIMARY KEY");
        for (prop_name, prop) in &self.properties {
            column_defs.push_str(",\n");
            column_defs.push_str(&format!("{prop_name} "));
            let type_def = match prop.r#type {
                PropertyType::String => "TEXT",
                PropertyType::Int => "INTEGER",
                PropertyType::Float => "REAL",
                PropertyType::Bool => "INTEGER",
            };
            column_defs.push_str(type_def);
            if !prop.nullable {
                column_defs.push_str(" NOT NULL");
            }
        }
        format!(
            "
            CREATE TABLE {} (
                {}
            );
            ",
            self.name, column_defs
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct Frontmatter {
    pub id: String,
    pub schema: String,
    pub properties: BTreeMap<String, Value>,
}

impl Frontmatter {
    pub fn new(id: Uuid, schema: &Schema) -> Self {
        Self {
            id: id.to_string(),
            schema: schema.name.clone(),
            properties: BTreeMap::from_iter(schema.properties.iter().map(|p| {
                let val = match &p.1.default {
                    Some(d) => d.clone(),
                    None => Value::Null,
                };

                (p.0.clone(), val)
            })),
        }
    }

    pub fn to_yaml_string(&self) -> String {
        serde_yaml::to_string(self).unwrap()
    }

    pub fn from_file(file: &File) -> Self {
        let rdr = BufReader::new(file);
        let mut lines = rdr.lines();
        assert!(lines.next().unwrap().unwrap() == "---");

        let mut frontmatter_string = String::new();
        loop {
            if let Some(Ok(mut line)) = lines.next() {
                if line == "---" {
                    break;
                }
                line.push_str("\n");
                frontmatter_string.push_str(&line);
            } else {
                break;
            }
        }

        serde_yaml::from_str(&frontmatter_string).unwrap()
    }

    pub fn sql_commit_string(&self) -> String {
        let mut column_names = String::new();
        column_names.push_str("id");

        let mut column_values = String::new();
        column_values.push_str(&self.id);

        for (prop_name, prop) in &self.properties {
            column_names.push_str(", ");
            column_names.push_str(prop_name);

            column_values.push_str(", ");
            let cv_str = match prop {
                Value::String(s) => s,
                Value::Number(n) => &n.to_string(),
                Value::Bool(b) => &b.to_string(),
                _ => panic!(),
            };
            column_values.push_str(cv_str);
        }

        format!(
            "
            REPLACE INTO {}({})
            VALUES({});
            ",
            self.schema, column_names, column_values,
        )
    }

    pub fn is_valid(&self, schema: &Schema) -> bool {
        for (prop_name, prop_val) in &self.properties {
            let prop_def = schema.properties.get(prop_name);
            match prop_def {
                Some(pd) => match prop_val {
                    Value::Null => {
                        if !pd.nullable {
                            return false;
                        }
                    }
                    Value::Number(n) => match &pd.r#type {
                        PropertyType::Int => {
                            if !(n.is_i64() || n.is_u64()) {
                                return false;
                            }
                        }
                        PropertyType::Float => {
                            if !n.is_f64() {
                                return false;
                            }
                        }
                        _ => return false,
                    },
                    Value::Bool(_) => match &pd.r#type {
                        PropertyType::Bool => {}
                        _ => return false,
                    },
                    Value::String(_) => match &pd.r#type {
                        PropertyType::String => {}
                        _ => return false,
                    },
                    _ => return false,
                },
                None => return false,
            }
        }

        true
    }
}

#[derive(Serialize, Deserialize)]
pub struct Property {
    r#type: PropertyType,
    nullable: bool,
    default: Option<Value>,
}

#[derive(Serialize, Deserialize)]
enum PropertyType {
    String,
    Int,
    Float,
    Bool,
}
