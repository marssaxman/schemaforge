use crate::error::Error;
use crate::ir::plan::PlanIr;
use kdl::KdlDocument;

pub fn parse_kdl(src: &str) -> Result<PlanIr, Error> {
    let doc: KdlDocument = src.parse()?;

    if doc.nodes().is_empty() {
        return Ok(PlanIr);
    }

    if doc.nodes().len() != 1 {
        return Err(Error::Parse(
            "plan IR expects a single 'plan' node".into(),
        ));
    }

    let node = &doc.nodes()[0];
    if node.name().value() != "plan" {
        return Err(Error::Parse("plan IR expects a 'plan' node".into()));
    }

    if !node.entries().is_empty() || node.children().is_some() {
        return Err(Error::Parse(
            "plan node does not accept entries or children".into(),
        ));
    }

    Ok(PlanIr)
}

pub fn print_kdl(_value: &PlanIr) -> String {
    "plan\n".to_string()
}
