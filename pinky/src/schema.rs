use std::{collections::BTreeMap, fs::File};

use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub properties: BTreeMap<String, Property>,
}

impl Schema {
    pub fn to_yaml_string(&self) -> String {
        todo!()
    }

    pub fn from_yaml_string(string: String) -> Self {
        serde_yaml::from_str(&string).unwrap()
    }

    pub fn table_def(&self) -> String {
        todo!()
    }

    pub fn is_valid(&self, frontmatter: &Frontmatter) -> bool {
        todo!()
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
        todo!()
    }

    pub fn to_sql_upsert(&self) -> String {
        todo!()
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
    Enum,
    Ref,
}
