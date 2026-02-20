#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaIr {
    pub tables: Vec<TableIr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableIr {
    pub name: String,
    pub fields: Vec<FieldIr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldIr {
    pub name: String,
    pub ty: String,
}
