use crate::error::Error;
use crate::ir::ast::{
    AstField, AstParam, AstProc, AstQuery, AstSchema, AstTable,
};
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

pub fn parse_kdl(src: &str) -> Result<AstSchema, Error> {
    let doc: KdlDocument = src.parse()?;
    let mut tables = Vec::new();
    let mut procs = Vec::new();
    let mut queries = Vec::new();

    for node in doc.nodes() {
        match node.name().value() {
            "table" => tables.push(parse_table(node)?),
            "proc" => procs.push(parse_proc(node)?),
            "query" => queries.push(parse_query(node)?),
            other => {
                return Err(Error::Parse(format!(
                "unknown root node '{}', expected 'table', 'proc', or 'query'",
                other
            )))
            }
        }
    }

    Ok(AstSchema {
        tables,
        procs,
        queries,
    })
}

pub fn print_kdl(value: &AstSchema) -> String {
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
        out.push_str(&format!(
            "proc \"{}\" table=\"{}\"",
            escape(&proc_def.name),
            escape(&proc_def.table)
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
        out.push_str(&format!(
            "query \"{}\" table=\"{}\"",
            escape(&query.name),
            escape(&query.table)
        ));

        if query.projection.is_empty() {
            out.push('\n');
            continue;
        }

        out.push_str(" {\n");
        for column in query.projection {
            out.push_str(&format!("  project \"{}\"\n", escape(&column)));
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
            fields.push(parse_field(child, &name)?);
        }
    }

    Ok(AstTable { name, fields })
}

fn parse_field(node: &KdlNode, table_name: &str) -> Result<AstField, Error> {
    let name = expect_single_string_value(node, "field")?;
    let ty = expect_string_property(node, "type")?;
    ensure_only_properties(node, "field", &["type"], table_name)?;

    Ok(AstField { name, ty })
}

fn parse_proc(node: &KdlNode) -> Result<AstProc, Error> {
    let name = expect_single_string_value(node, "proc")?;
    let table = expect_string_property(node, "table")?;
    ensure_only_properties(node, "proc", &["table"], "")?;

    let mut params = Vec::new();
    if let Some(children) = node.children() {
        for child in children.nodes() {
            if child.name().value() != "param" {
                return Err(Error::Parse(format!(
                    "unknown node '{}' in proc '{}', expected 'param'",
                    child.name().value(),
                    name
                )));
            }
            params.push(parse_param(child, &name)?);
        }
    }

    Ok(AstProc {
        name,
        table,
        params,
    })
}

fn parse_param(node: &KdlNode, proc_name: &str) -> Result<AstParam, Error> {
    let name = expect_single_string_value(node, "param")?;
    let ty = expect_string_property(node, "type")?;
    ensure_only_properties(node, "param", &["type"], proc_name)?;

    Ok(AstParam { name, ty })
}

fn parse_query(node: &KdlNode) -> Result<AstQuery, Error> {
    let name = expect_single_string_value(node, "query")?;
    let table = expect_string_property(node, "table")?;
    ensure_only_properties(node, "query", &["table"], "")?;

    let mut projection = Vec::new();
    if let Some(children) = node.children() {
        for child in children.nodes() {
            if child.name().value() != "project" {
                return Err(Error::Parse(format!(
                    "unknown node '{}' in query '{}', expected 'project'",
                    child.name().value(),
                    name
                )));
            }
            projection.push(expect_single_string_value(child, "project")?);
            ensure_no_properties(child, "project")?;
            if child.children().is_some() {
                return Err(Error::Parse(format!(
                    "'project' node in query '{}' does not support children",
                    name
                )));
            }
        }
    }

    Ok(AstQuery {
        name,
        table,
        projection,
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

fn ensure_only_properties(
    node: &KdlNode,
    kind: &str,
    allowed: &[&str],
    parent_name: &str,
) -> Result<(), Error> {
    for entry in node.entries() {
        if let Some(name) = entry.name() {
            if !allowed.iter().any(|value| *value == name.value()) {
                if parent_name.is_empty() {
                    return Err(Error::Parse(format!(
                        "'{}' node does not support property '{}'",
                        kind,
                        name.value()
                    )));
                }
                return Err(Error::Parse(format!(
                    "'{}' node in '{}' does not support property '{}'",
                    kind,
                    parent_name,
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
