use crate::error::Error;
use crate::ir::schema::{QueryIr, ResolvedSchema};
use crate::plan::Plan;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoweredQuery {
    pub name: String,
    pub plan: Plan,
}

pub fn lower_queries(
    schema: &ResolvedSchema,
) -> Result<Vec<LoweredQuery>, Error> {
    schema
        .queries
        .iter()
        .map(lower_query)
        .collect::<Result<Vec<_>, _>>()
}

fn lower_query(query: &QueryIr) -> Result<LoweredQuery, Error> {
    if query.projection.is_empty() {
        return Err(Error::Pass(format!(
            "query '{}' is unsupported: projection must include at least one column",
            query.name
        )));
    }

    for column in &query.projection {
        if column.table != query.table {
            return Err(Error::Pass(format!(
                "query '{}' is unsupported: projection columns must come from a single table",
                query.name
            )));
        }
    }

    let plan = Plan::Project {
        input: Box::new(Plan::TableScan { table: query.table }),
        columns: query.projection.clone(),
    };

    Ok(LoweredQuery {
        name: query.name.clone(),
        plan,
    })
}
