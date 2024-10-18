use uuid::Uuid;

use crate::{library::LibraryConfig, schema::Schema};

pub struct Page {
    id: Uuid,
    schema: Schema,
}

// impl Page {
//     pub fn new(schema_name: Option<String>, lib_config: LibraryConfig) -> Self {
//         Self { id: Uuid::new_v4() }
//     }
// }
