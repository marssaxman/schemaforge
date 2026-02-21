use schemaforge::backend::sqlite::compile_plan_to_sql;
use schemaforge::ir;
use schemaforge::lower::lower_queries;
use schemaforge::passes;
use schemaforge::plan::{ColumnId, Plan};
use std::fs;
use std::path::PathBuf;

#[test]
fn lowers_projection_query_to_plan() {
    let schema = load_resolved_schema();
    let lowered = lower_queries(&schema).expect("lower query plans");

    let query = lowered
        .iter()
        .find(|query| query.name == "list_names_and_ids")
        .expect("query exists");

    let expected = Plan::Project {
        input: Box::new(Plan::TableScan { table: 0 }),
        columns: vec![
            ColumnId {
                table: 0,
                column: 1,
            },
            ColumnId {
                table: 0,
                column: 0,
            },
        ],
    };

    assert_eq!(query.plan, expected);
}

#[test]
fn compiles_projection_plan_to_sqlite_sql() {
    let schema = load_resolved_schema();
    let lowered = lower_queries(&schema).expect("lower query plans");

    let query = lowered
        .iter()
        .find(|query| query.name == "list_names_and_ids")
        .expect("query exists");

    let sql = compile_plan_to_sql(&query.plan, &schema).expect("compile sql");

    assert_eq!(sql.sql, "SELECT \"name\", \"id\" FROM \"people\"");
    assert_eq!(
        sql.result_columns,
        vec![
            ColumnId {
                table: 0,
                column: 1,
            },
            ColumnId {
                table: 0,
                column: 0,
            },
        ]
    );
}

fn load_resolved_schema() -> schemaforge::ir::schema::ResolvedSchema {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("tests/fixtures/spike/spike.in.kdl");
    let input = fs::read_to_string(&fixture).expect("read fixture");
    let ast = ir::ast::parse_kdl(&input).expect("parse fixture");
    passes::resolve::run(&ast).expect("resolve fixture")
}
