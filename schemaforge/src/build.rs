use crate::backend::sqlite::{
    compile_create_table_sql, compile_insert_proc_sql, compile_plan_to_sql,
};
use crate::error::Error;
use crate::ir;
use crate::ir::schema::{ProcIr, QueryIr, ResolvedSchema};
use crate::lower::{lower_queries, LoweredQuery};
use crate::plan::ColumnId;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn build(input: &Path) -> Result<PathBuf, Error> {
    let input_text = fs::read_to_string(input)?;
    let ast = ir::ast::parse_kdl(&input_text)?;
    let schema = crate::passes::resolve::run(&ast)?;
    let lowered = lower_queries(&schema)?;

    let output_dir = output_dir(input);
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir)?;
    }
    fs::create_dir_all(output_dir.join("src"))?;

    let crate_name = crate_name(input);

    fs::write(
        output_dir.join("Cargo.toml"),
        render_cargo_toml(&crate_name),
    )?;
    fs::write(
        output_dir.join("src/lib.rs"),
        render_lib_rs(&schema, &lowered)?,
    )?;
    fs::write(
        output_dir.join("src/main.rs"),
        render_main_rs(&schema, &lowered, &crate_name)?,
    )?;

    Ok(output_dir)
}

fn output_dir(input: &Path) -> PathBuf {
    let dir = sanitize_component(&input_base_name(input));
    PathBuf::from("target").join("schemaforge-out").join(dir)
}

fn crate_name(input: &Path) -> String {
    format!(
        "schemaforge_generated_{}",
        sanitize_ident(&input_base_name(input))
    )
}

fn render_cargo_toml(crate_name: &str) -> String {
    format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nanyhow = \"1\"\nrusqlite = \"0.31\"\n\n[workspace]\n",
        crate_name
    )
}

fn render_lib_rs(
    schema: &ResolvedSchema,
    lowered: &[LoweredQuery],
) -> Result<String, Error> {
    let mut create_table_sql = Vec::new();
    for table in &schema.tables {
        create_table_sql.push(format!("{};", compile_create_table_sql(table)?));
    }

    let lowered_map = lowered
        .iter()
        .map(|query| (query.name.clone(), query))
        .collect::<HashMap<_, _>>();

    let mut proc_methods = String::new();
    for proc_def in &schema.procs {
        proc_methods.push_str(&render_proc_method(proc_def, schema)?);
        proc_methods.push('\n');
    }

    let mut query_methods = String::new();
    for query in &schema.queries {
        let lowered_query = lowered_map.get(&query.name).ok_or_else(|| {
            Error::Pass(format!(
                "missing lowered plan for query '{}'",
                query.name
            ))
        })?;
        query_methods.push_str(&render_query_method(
            query,
            &lowered_query.plan,
            schema,
        )?);
        query_methods.push('\n');
    }

    let create_batch = create_table_sql.join("\n");
    let create_batch_literal = rust_string_literal(&create_batch);

    Ok(format!(
        "use rusqlite::{{params, Connection}};\n\npub struct Db {{\n    conn: Connection,\n}}\n\nimpl Db {{\n    pub fn new() -> anyhow::Result<Self> {{\n        let conn = Connection::open_in_memory()?;\n        conn.execute_batch({})?;\n        Ok(Self {{ conn }})\n    }}\n\n{}{}\n}}\n",
        create_batch_literal, proc_methods, query_methods
    ))
}

fn render_proc_method(
    proc_def: &ProcIr,
    schema: &ResolvedSchema,
) -> Result<String, Error> {
    let method_name = sanitize_ident(&proc_def.name);
    let insert_sql = compile_insert_proc_sql(proc_def, schema)?;
    let insert_sql_literal = rust_string_literal(&insert_sql);

    let mut signature_params = Vec::new();
    let mut arg_names = Vec::new();
    for param in &proc_def.params {
        let arg_name = sanitize_ident(&param.name);
        let arg_ty = rust_type_name(&param.ty)?;
        signature_params.push(format!("{}: {}", arg_name, arg_ty));
        arg_names.push(arg_name);
    }

    Ok(format!(
        "    pub fn {}(&mut self, {}) -> anyhow::Result<()> {{\n        self.conn.execute({}, params![{}])?;\n        Ok(())\n    }}\n",
        method_name,
        signature_params.join(", "),
        insert_sql_literal,
        arg_names.join(", ")
    ))
}

fn render_query_method(
    query: &QueryIr,
    plan: &crate::plan::Plan,
    schema: &ResolvedSchema,
) -> Result<String, Error> {
    let compiled = compile_plan_to_sql(plan, schema)?;
    let method_name = sanitize_ident(&query.name);
    let sql_literal = rust_string_literal(&compiled.sql);

    let mut rust_types = Vec::new();
    for column_id in &compiled.result_columns {
        let field = schema.column(*column_id).ok_or_else(|| {
            Error::Pass(format!(
                "query '{}' references unknown column id {}:{}",
                query.name, column_id.table, column_id.column
            ))
        })?;
        rust_types.push(rust_type_name(&field.ty)?.to_string());
    }

    let tuple_type = tuple_type(&rust_types);
    let tuple_decode = tuple_decode_expr(&compiled.result_columns, schema)?;

    Ok(format!(
        "    pub fn {}(&self) -> anyhow::Result<Vec<{}>> {{\n        let mut stmt = self.conn.prepare({})?;\n        let rows = stmt.query_map([], |row| {{\n            Ok({})\n        }})?;\n\n        let mut out = Vec::new();\n        for row in rows {{\n            out.push(row?);\n        }}\n        Ok(out)\n    }}\n",
        method_name, tuple_type, sql_literal, tuple_decode
    ))
}

fn tuple_decode_expr(
    columns: &[ColumnId],
    schema: &ResolvedSchema,
) -> Result<String, Error> {
    let mut values = Vec::new();
    for (index, column_id) in columns.iter().enumerate() {
        let field = schema.column(*column_id).ok_or_else(|| {
            Error::Pass(format!(
                "query references unknown column id {}:{}",
                column_id.table, column_id.column
            ))
        })?;

        values.push(format!(
            "row.get::<_, {}>({})?",
            rust_type_name(&field.ty)?,
            index
        ));
    }

    if values.len() == 1 {
        Ok(format!("({},)", values[0]))
    } else {
        Ok(format!("({})", values.join(", ")))
    }
}

fn tuple_type(types: &[String]) -> String {
    if types.len() == 1 {
        format!("({},)", types[0])
    } else {
        format!("({})", types.join(", "))
    }
}

fn rust_type_name(type_name: &str) -> Result<&'static str, Error> {
    match type_name {
        "i64" => Ok("i64"),
        "text" => Ok("String"),
        other => Err(Error::Pass(format!(
            "unsupported scalar type '{}' for generated Rust code",
            other
        ))),
    }
}

fn render_main_rs(
    schema: &ResolvedSchema,
    lowered: &[LoweredQuery],
    crate_name: &str,
) -> Result<String, Error> {
    let proc_def = schema.procs.first().ok_or_else(|| {
        Error::Pass("build requires at least one proc".into())
    })?;
    let query_def = schema.queries.first().ok_or_else(|| {
        Error::Pass("build requires at least one query".into())
    })?;

    let proc_name = sanitize_ident(&proc_def.name);
    let query_name = sanitize_ident(&query_def.name);

    let mut demo_calls = String::new();
    for row_index in 0..2 {
        let mut args = Vec::new();
        for (param_index, param) in proc_def.params.iter().enumerate() {
            args.push(demo_value(param, row_index, param_index)?);
        }
        demo_calls.push_str(&format!(
            "    db.{}({})?;\n",
            proc_name,
            args.join(", ")
        ));
    }

    // Ensure lowering happened for at least the first query in this demo build.
    let lowered_map = lowered
        .iter()
        .map(|query| query.name.as_str())
        .collect::<Vec<_>>();
    if !lowered_map.iter().any(|name| *name == query_def.name) {
        return Err(Error::Pass(format!(
            "missing lowered plan for query '{}'",
            query_def.name
        )));
    }

    Ok(format!(
        "use anyhow::Result;\nuse {}::Db;\n\nfn main() -> Result<()> {{\n    let mut db = Db::new()?;\n{}\n    let rows = db.{}()?;\n    for row in rows {{\n        println!(\"{{:?}}\", row);\n    }}\n\n    Ok(())\n}}\n",
        crate_name,
        demo_calls,
        query_name
    ))
}

fn demo_value(
    param: &crate::ir::schema::ProcParamIr,
    row_index: usize,
    param_index: usize,
) -> Result<String, Error> {
    match param.ty.as_str() {
        "i64" => Ok(((row_index + param_index + 1) as i64).to_string()),
        "text" => Ok(format!(
            "\"{}\".to_string()",
            escape_rust_string(&format!("{}_{}", param.name, row_index + 1))
        )),
        other => Err(Error::Pass(format!(
            "unsupported scalar type '{}' in demo generator",
            other
        ))),
    }
}

fn rust_string_literal(value: &str) -> String {
    format!("{:?}", value)
}

fn sanitize_component(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }

    if out.is_empty() {
        "schema".to_string()
    } else {
        out
    }
}

fn sanitize_ident(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }

    if out.is_empty() {
        return "generated".to_string();
    }

    let first_is_ok = out
        .chars()
        .next()
        .map(|ch| ch.is_ascii_alphabetic() || ch == '_')
        .unwrap_or(false);
    if first_is_ok {
        out
    } else {
        format!("_{}", out)
    }
}

fn escape_rust_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn input_base_name(input: &Path) -> String {
    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("schema");
    stem.strip_suffix(".in").unwrap_or(stem).to_string()
}
