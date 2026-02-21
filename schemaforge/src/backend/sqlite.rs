use crate::error::Error;
use crate::ir::schema::{ProcIr, ResolvedSchema, TableIr};
use crate::plan::{ColumnId, Plan};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqlQuery {
    pub sql: String,
    pub result_columns: Vec<ColumnId>,
}

pub fn compile_plan_to_sql(
    plan: &Plan,
    schema: &ResolvedSchema,
) -> Result<SqlQuery, Error> {
    match plan {
        Plan::TableScan { .. } => Err(Error::Pass(
            "unsupported plan shape for SQLite backend: bare TableScan".into(),
        )),
        Plan::Project { input, columns } => {
            if columns.is_empty() {
                return Err(Error::Pass(
                    "unsupported plan shape for SQLite backend: empty projection"
                        .into(),
                ));
            }

            let table_id = match input.as_ref() {
                Plan::TableScan { table } => *table,
                _ => {
                    return Err(Error::Pass(
                        "unsupported plan shape for SQLite backend: expected Project(TableScan)"
                            .into(),
                    ))
                }
            };

            let table = schema.table(table_id).ok_or_else(|| {
                Error::Pass(format!(
                    "query references unknown table id {}",
                    table_id
                ))
            })?;

            let mut selected_columns = Vec::with_capacity(columns.len());
            for column_id in columns {
                if column_id.table != table_id {
                    return Err(Error::Pass(
                        "unsupported plan shape for SQLite backend: projection columns must come from scan table"
                            .into(),
                    ));
                }

                let column = schema.column(*column_id).ok_or_else(|| {
                    Error::Pass(format!(
                        "query references unknown column id {}:{}",
                        column_id.table, column_id.column
                    ))
                })?;
                selected_columns.push(quote_ident(&column.name));
            }

            Ok(SqlQuery {
                sql: format!(
                    "SELECT {} FROM {}",
                    selected_columns.join(", "),
                    quote_ident(&table.name)
                ),
                result_columns: columns.clone(),
            })
        }
    }
}

pub fn compile_create_table_sql(table: &TableIr) -> Result<String, Error> {
    let mut columns = Vec::with_capacity(table.fields.len());
    for field in &table.fields {
        columns.push(format!(
            "{} {}",
            quote_ident(&field.name),
            sqlite_type_name(&field.ty)?
        ));
    }

    Ok(format!(
        "CREATE TABLE {} ({})",
        quote_ident(&table.name),
        columns.join(", ")
    ))
}

pub fn compile_insert_proc_sql(
    proc_def: &ProcIr,
    schema: &ResolvedSchema,
) -> Result<String, Error> {
    let table = schema.table(proc_def.table).ok_or_else(|| {
        Error::Pass(format!(
            "proc '{}' references unknown table id {}",
            proc_def.name, proc_def.table
        ))
    })?;

    if proc_def.params.is_empty() {
        return Err(Error::Pass(format!(
            "proc '{}' is unsupported: insert proc must have at least one param",
            proc_def.name
        )));
    }

    let mut column_names = Vec::with_capacity(proc_def.params.len());
    let mut placeholders = Vec::with_capacity(proc_def.params.len());

    for (index, param) in proc_def.params.iter().enumerate() {
        if param.column.table != proc_def.table {
            return Err(Error::Pass(format!(
                "proc '{}' is unsupported: all params must target the same table",
                proc_def.name
            )));
        }

        let column = schema.column(param.column).ok_or_else(|| {
            Error::Pass(format!(
                "proc '{}' references unknown column id {}:{}",
                proc_def.name, param.column.table, param.column.column
            ))
        })?;

        column_names.push(quote_ident(&column.name));
        placeholders.push(format!("?{}", index + 1));
    }

    Ok(format!(
        "INSERT INTO {} ({}) VALUES ({})",
        quote_ident(&table.name),
        column_names.join(", "),
        placeholders.join(", ")
    ))
}

pub fn sqlite_type_name(type_name: &str) -> Result<&'static str, Error> {
    match type_name {
        "i64" => Ok("INTEGER"),
        "text" => Ok("TEXT"),
        other => Err(Error::Pass(format!(
            "unsupported scalar type '{}' for SQLite backend",
            other
        ))),
    }
}

pub fn quote_ident(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
