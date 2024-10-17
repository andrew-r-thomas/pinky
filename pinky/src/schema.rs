use crate::library::Frontmatter;

pub struct Schema {
    pub name: String,
    pub properties: Vec<Property>,
}

impl Schema {
    pub fn to_yaml_string(&self) -> String {
        todo!()
    }

    pub fn table_def(&self) -> String {
        todo!()
    }

    pub fn from_yaml_string(string: String) -> Self {
        todo!()
    }

    pub fn is_valid(&self, frontmatter: &Frontmatter) -> bool {
        todo!()
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            name: "default".into(),
            properties: Vec::new(),
        }
    }
}

pub struct Property {}
