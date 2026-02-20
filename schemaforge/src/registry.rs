use crate::error::Error;
use crate::ir;
use crate::passes;

pub struct PassSpec {
    pub name: &'static str,
    pub help: &'static str,
    pub run: fn(&str) -> Result<String, Error>,
}

pub fn all_passes() -> &'static [PassSpec] {
    &PASS_REGISTRY
}

pub fn find_pass(name: &str) -> Option<&'static PassSpec> {
    PASS_REGISTRY.iter().find(|spec| spec.name == name)
}

fn run_ast_to_schema(input: &str) -> Result<String, Error> {
    let ast = ir::ast::parse_kdl(input)?;
    let schema = passes::ast_to_schema::run(&ast)?;
    Ok(ir::schema::print_kdl(&schema))
}

static PASS_REGISTRY: [PassSpec; 1] = [PassSpec {
    name: "ast-to-schema",
    help: "Lower AST IR to Schema IR",
    run: run_ast_to_schema,
}];
