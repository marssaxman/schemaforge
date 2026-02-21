#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AstSchema {
    pub tables: Vec<AstTable>,
    pub procs: Vec<AstProc>,
    pub queries: Vec<AstQuery>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AstProc {
    pub name: String,
    pub table: String,
    pub params: Vec<AstParam>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AstParam {
    pub name: String,
    pub ty: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AstQuery {
    pub name: String,
    pub table: String,
    pub projection: Vec<String>,
}
