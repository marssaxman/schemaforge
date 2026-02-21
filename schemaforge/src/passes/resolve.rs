use crate::error::Error;
use crate::ir::ast::AstSchema;
use crate::ir::schema::{
    FieldIr, ProcIr, ProcParamIr, QueryIr, SchemaIr, TableIr,
};
use crate::plan::ColumnId;
use std::collections::{HashMap, HashSet};

pub fn run(input: &AstSchema) -> Result<SchemaIr, Error> {
    let mut table_name_to_id = HashMap::new();
    let mut tables = Vec::new();

    for table in &input.tables {
        if table_name_to_id
            .insert(table.name.clone(), tables.len())
            .is_some()
        {
            return Err(Error::Pass(format!(
                "duplicate table name '{}'",
                table.name
            )));
        }

        let table_id = tables.len();
        let mut seen_fields = HashSet::new();
        let mut fields = Vec::new();

        for field in &table.fields {
            if !seen_fields.insert(field.name.clone()) {
                return Err(Error::Pass(format!(
                    "duplicate field '{}' in table '{}'",
                    field.name, table.name
                )));
            }

            fields.push(FieldIr {
                id: ColumnId {
                    table: table_id,
                    column: fields.len(),
                },
                name: field.name.clone(),
                ty: field.ty.clone(),
            });
        }

        tables.push(TableIr {
            id: table_id,
            name: table.name.clone(),
            fields,
        });
    }

    let mut procs = Vec::new();
    let mut seen_proc_names = HashSet::new();
    for proc_def in &input.procs {
        if !seen_proc_names.insert(proc_def.name.clone()) {
            return Err(Error::Pass(format!(
                "duplicate proc name '{}'",
                proc_def.name
            )));
        }

        let table_id = table_name_to_id
            .get(&proc_def.table)
            .copied()
            .ok_or_else(|| {
                Error::Pass(format!(
                    "proc '{}' references unknown table '{}'",
                    proc_def.name, proc_def.table
                ))
            })?;

        let table = &tables[table_id];
        let mut params = Vec::new();

        for param in &proc_def.params {
            let column = find_column(table, &param.name).ok_or_else(|| {
                Error::Pass(format!(
                    "proc '{}' references unknown column '{}' in table '{}'",
                    proc_def.name, param.name, table.name
                ))
            })?;

            if param.ty != column.ty {
                return Err(Error::Pass(format!(
                    "proc '{}' param '{}' type '{}' does not match table column type '{}'",
                    proc_def.name, param.name, param.ty, column.ty
                )));
            }

            params.push(ProcParamIr {
                name: param.name.clone(),
                ty: param.ty.clone(),
                column: column.id,
            });
        }

        procs.push(ProcIr {
            name: proc_def.name.clone(),
            table: table_id,
            params,
        });
    }

    let mut queries = Vec::new();
    let mut seen_query_names = HashSet::new();
    for query in &input.queries {
        if !seen_query_names.insert(query.name.clone()) {
            return Err(Error::Pass(format!(
                "duplicate query name '{}'",
                query.name
            )));
        }

        let table_id =
            table_name_to_id.get(&query.table).copied().ok_or_else(|| {
                Error::Pass(format!(
                    "query '{}' references unknown table '{}'",
                    query.name, query.table
                ))
            })?;

        let table = &tables[table_id];
        let mut projection = Vec::new();
        for column_name in &query.projection {
            let column = find_column(table, column_name).ok_or_else(|| {
                Error::Pass(format!(
                    "query '{}' projects unknown column '{}' in table '{}'",
                    query.name, column_name, table.name
                ))
            })?;
            projection.push(column.id);
        }

        queries.push(QueryIr {
            name: query.name.clone(),
            table: table_id,
            projection,
        });
    }

    Ok(SchemaIr {
        tables,
        procs,
        queries,
    })
}

fn find_column<'a>(table: &'a TableIr, name: &str) -> Option<&'a FieldIr> {
    table.fields.iter().find(|field| field.name == name)
}
