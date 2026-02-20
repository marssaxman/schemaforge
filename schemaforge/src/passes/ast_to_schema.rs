use crate::error::Error;
use crate::ir::ast::AstSchema;
use crate::ir::schema::{FieldIr, SchemaIr, TableIr};

pub fn run(input: &AstSchema) -> Result<SchemaIr, Error> {
    let tables = input
        .tables
        .iter()
        .map(|table| TableIr {
            name: table.name.clone(),
            fields: table
                .fields
                .iter()
                .map(|field| FieldIr {
                    name: field.name.clone(),
                    ty: field.ty.clone(),
                })
                .collect(),
        })
        .collect();

    Ok(SchemaIr { tables })
}
