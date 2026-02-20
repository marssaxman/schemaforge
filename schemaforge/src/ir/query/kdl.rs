use crate::error::Error;
use crate::ir::query::QueryIr;
use kdl::KdlDocument;

pub fn parse_kdl(src: &str) -> Result<QueryIr, Error> {
    let doc: KdlDocument = src.parse()?;

    if doc.nodes().is_empty() {
        return Ok(QueryIr);
    }

    if doc.nodes().len() != 1 {
        return Err(Error::Parse(
            "query IR expects a single 'query' node".into(),
        ));
    }

    let node = &doc.nodes()[0];
    if node.name().value() != "query" {
        return Err(Error::Parse("query IR expects a 'query' node".into()));
    }

    if !node.entries().is_empty() || node.children().is_some() {
        return Err(Error::Parse(
            "query node does not accept entries or children".into(),
        ));
    }

    Ok(QueryIr)
}

pub fn print_kdl(_value: &QueryIr) -> String {
    "query\n".to_string()
}
