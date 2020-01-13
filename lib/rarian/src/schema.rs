use std::sync::Arc;

enum FieldType {
    Int,
    Text,
}

struct Field {
    ft: FieldType,
    indexed: bool,
}

struct Schema(Arc<Vec<Field>>);
impl Schema {
    pub fn builder() -> SchemaBuilder {
        SchemaBuilder::default()
    }
}

struct SchemaBuilder {}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self {}
    }
}
