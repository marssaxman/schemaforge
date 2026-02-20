#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AstSchema {
    pub tables: Vec<AstTable>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AstTable {
    pub name: String,
    pub fields: Vec<AstField>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AstField {
    pub name: String,
    pub ty: String,
}
