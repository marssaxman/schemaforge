#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ColumnId {
    pub table: TableId,
    pub column: usize,
}

pub type TableId = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Plan {
    TableScan {
        table: TableId,
    },
    Project {
        input: Box<Plan>,
        columns: Vec<ColumnId>,
    },
}
