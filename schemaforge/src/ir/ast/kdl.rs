use crate::error::Error;
use crate::ir::ast::{AstField, AstSchema, AstTable};
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

pub fn parse_kdl(src: &str) -> Result<AstSchema, Error> {
    let doc: KdlDocument = src.parse()?;
    let mut tables = Vec::new();

    for node in doc.nodes() {
        if node.name().value() != "table" {
            return Err(Error::Parse(format!(
                "unknown root node '{}', expected 'table'",
                node.name().value()
            )));
        }
        let table = parse_table(node)?;
        tables.push(table);
    }

    Ok(AstSchema { tables })
}

pub fn print_kdl(value: &AstSchema) -> String {
    let mut tables = value.tables.clone();
    tables.sort_by(|a, b| a.name.cmp(&b.name));

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

    out
}

fn parse_table(node: &KdlNode) -> Result<AstTable, Error> {
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
            let field = parse_field(child, &name)?;
            fields.push(field);
        }
    }

    Ok(AstTable { name, fields })
}

fn parse_field(node: &KdlNode, table_name: &str) -> Result<AstField, Error> {
    let name = expect_single_string_value(node, "field")?;
    let ty = expect_string_property(node, "type")?;
    ensure_only_property(node, "field", "type", table_name)?;

    Ok(AstField { name, ty })
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

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
