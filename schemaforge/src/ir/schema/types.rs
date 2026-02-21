use crate::plan::{ColumnId, TableId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaIr {
    pub tables: Vec<TableIr>,
    pub procs: Vec<ProcIr>,
    pub queries: Vec<QueryIr>,
}

pub type ResolvedSchema = SchemaIr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableIr {
    pub id: TableId,
    pub name: String,
    pub fields: Vec<FieldIr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldIr {
    pub id: ColumnId,
    pub name: String,
    pub ty: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcIr {
    pub name: String,
    pub table: TableId,
    pub params: Vec<ProcParamIr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcParamIr {
    pub name: String,
    pub ty: String,
    pub column: ColumnId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryIr {
    pub name: String,
    pub table: TableId,
    pub projection: Vec<ColumnId>,
}

impl SchemaIr {
    pub fn table(&self, table_id: TableId) -> Option<&TableIr> {
        self.tables.get(table_id)
    }

    pub fn column(&self, column_id: ColumnId) -> Option<&FieldIr> {
        self.tables
            .get(column_id.table)
            .and_then(|table| table.fields.get(column_id.column))
    }
}
