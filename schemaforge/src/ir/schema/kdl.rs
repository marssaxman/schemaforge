use crate::error::Error;
use crate::ir::schema::{FieldIr, SchemaIr, TableIr};
use crate::plan::ColumnId;
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

pub fn parse_kdl(src: &str) -> Result<SchemaIr, Error> {
    let doc: KdlDocument = src.parse()?;
    let mut tables = Vec::new();

    for node in doc.nodes() {
        if node.name().value() != "table" {
            return Err(Error::Parse(format!(
                "unknown root node '{}', expected 'table'",
                node.name().value()
            )));
        }

        let table_id = tables.len();
        tables.push(parse_table(node, table_id)?);
    }

    Ok(SchemaIr {
        tables,
        procs: Vec::new(),
        queries: Vec::new(),
    })
}

pub fn print_kdl(value: &SchemaIr) -> String {
    let mut tables = value.tables.clone();
    tables.sort_by(|a, b| a.name.cmp(&b.name));

    let mut procs = value.procs.clone();
    procs.sort_by(|a, b| a.name.cmp(&b.name));

    let mut queries = value.queries.clone();
    queries.sort_by(|a, b| a.name.cmp(&b.name));

    let mut out = String::new();

    for table in tables {
        let mut fields = table.fields.clone();
        fields.sort_by(|a, b| a.name.cmp(&b.name));

        if fields.is_empty() {
            out.push_str(&format!("table \"{}\"\n", escape(&table.name)));
            continue;
        }

        out.push_str(&format!("table \"{}\" {{\n", escape(&table.name)));
        for field in fields {
            out.push_str(&format!(
                "  field \"{}\" type=\"{}\"\n",
                escape(&field.name),
                escape(&field.ty)
            ));
        }
        out.push_str("}\n");
    }

    for proc_def in procs {
        let table_name = table_name(value, proc_def.table);
        out.push_str(&format!(
            "proc \"{}\" table=\"{}\"",
            escape(&proc_def.name),
            escape(table_name)
        ));

        if proc_def.params.is_empty() {
            out.push('\n');
            continue;
        }

        out.push_str(" {\n");
        for param in proc_def.params {
            out.push_str(&format!(
                "  param \"{}\" type=\"{}\"\n",
                escape(&param.name),
                escape(&param.ty)
            ));
        }
        out.push_str("}\n");
    }

    for query in queries {
        let table_name = table_name(value, query.table);
        out.push_str(&format!(
            "query \"{}\" table=\"{}\"",
            escape(&query.name),
            escape(table_name)
        ));

        if query.projection.is_empty() {
            out.push('\n');
            continue;
        }

        out.push_str(" {\n");
        for column_id in query.projection {
            let column_name = column_name(value, column_id);
            out.push_str(&format!("  project \"{}\"\n", escape(column_name)));
        }
        out.push_str("}\n");
    }

    out
}

fn parse_table(node: &KdlNode, table_id: usize) -> Result<TableIr, Error> {
    let name = expect_single_string_value(node, "table")?;
    ensure_no_properties(node, "table")?;

    let mut fields = Vec::new();
    if let Some(children) = node.children() {
        for child in children.nodes() {
            if child.name().value() != "field" {
                return Err(Error::Parse(format!(
                    "unknown node '{}' in table '{}', expected 'field'",
                    child.name().value(),
                    name
                )));
            }
            let field_id = ColumnId {
                table: table_id,
                column: fields.len(),
            };
            fields.push(parse_field(child, &name, field_id)?);
        }
    }

    Ok(TableIr {
        id: table_id,
        name,
        fields,
    })
}

fn parse_field(
    node: &KdlNode,
    table_name: &str,
    field_id: ColumnId,
) -> Result<FieldIr, Error> {
    let name = expect_single_string_value(node, "field")?;
    let ty = expect_string_property(node, "type")?;
    ensure_only_property(node, "field", "type", table_name)?;

    Ok(FieldIr {
        id: field_id,
        name,
        ty,
    })
}

fn expect_single_string_value(
    node: &KdlNode,
    kind: &str,
) -> Result<String, Error> {
    let values: Vec<&KdlEntry> = node
        .entries()
        .iter()
        .filter(|entry| entry.name().is_none())
        .collect();

    if values.len() != 1 {
        return Err(Error::Parse(format!(
            "'{}' node must have exactly one string value",
            kind
        )));
    }

    match values[0].value() {
        KdlValue::String(s) => Ok(s.to_string()),
        _ => Err(Error::Parse(format!(
            "'{}' node value must be a string",
            kind
        ))),
    }
}

fn expect_string_property(node: &KdlNode, key: &str) -> Result<String, Error> {
    let entry = node.entries().iter().find(|entry| {
        entry
            .name()
            .map(|name| name.value() == key)
            .unwrap_or(false)
    });

    let entry = entry.ok_or_else(|| {
        Error::Parse(format!("missing required '{}' property", key))
    })?;

    match entry.value() {
        KdlValue::String(s) => Ok(s.to_string()),
        _ => Err(Error::Parse(format!("property '{}' must be a string", key))),
    }
}

fn ensure_no_properties(node: &KdlNode, kind: &str) -> Result<(), Error> {
    for entry in node.entries() {
        if let Some(name) = entry.name() {
            return Err(Error::Parse(format!(
                "'{}' node does not support property '{}'",
                kind,
                name.value()
            )));
        }
    }
    Ok(())
}

fn ensure_only_property(
    node: &KdlNode,
    kind: &str,
    property: &str,
    table_name: &str,
) -> Result<(), Error> {
    for entry in node.entries() {
        if let Some(name) = entry.name() {
            if name.value() != property {
                return Err(Error::Parse(format!(
                    "'{}' node in table '{}' does not support property '{}'",
                    kind,
                    table_name,
                    name.value()
                )));
            }
        }
    }
    Ok(())
}

fn table_name(schema: &SchemaIr, table_id: usize) -> &str {
    schema
        .tables
        .get(table_id)
        .map(|table| table.name.as_str())
        .unwrap_or("<invalid>")
}

fn column_name(schema: &SchemaIr, column_id: ColumnId) -> &str {
    schema
        .tables
        .get(column_id.table)
        .and_then(|table| table.fields.get(column_id.column))
        .map(|field| field.name.as_str())
        .unwrap_or("<invalid>")
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
