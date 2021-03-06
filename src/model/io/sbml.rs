use crate::func::expr;
use crate::func::expr::{Comparator, Expr, Operator};
use crate::func::Formula;
use crate::model::{io, GroupedVariables, QModel};
use std::io::Write;

use crate::helper::error::{CanFail, EmptyLomakResult, GenericError, LomakError, ParseError};
use crate::model::io::Format;
use crate::model::layout::NodeLayoutInfo;
use once_cell::sync::Lazy;
use regex::Regex;
use roxmltree::{Children, Document, Node};
use std::rc::Rc;
use std::str::FromStr;
use xmlwriter::XmlWriter;

const BASE_NS: &str = r"http://www.sbml.org/sbml/level3/version(\d)";

static SBML_NS: Lazy<Regex> = Lazy::new(|| Regex::new(&(format!(r"{}/core", BASE_NS))).unwrap());
static QUAL_NS: Lazy<Regex> =
    Lazy::new(|| Regex::new(&(format!(r"{}/qual/version(\d)", BASE_NS))).unwrap());
static LAYOUT_NS: Lazy<Regex> =
    Lazy::new(|| Regex::new(&(format!(r"{}/layout/version(\d)", BASE_NS))).unwrap());

#[derive(Default)]
pub struct SBMLFormat;
pub struct SBMLParser;

impl Format for SBMLFormat {
    fn description(&self) -> &str {
        "The SBML qual exchange format"
    }
}

impl io::SavingFormat for SBMLFormat {
    fn write_rules(&self, model: &QModel, out: &mut dyn Write) -> EmptyLomakResult {
        let sbml_ns = "http://www.sbml.org/sbml/level3/version2/core";
        let qual_ns = "http://www.sbml.org/sbml/level3/version1/qual/version1";
        let layout_ns = "http://www.sbml.org/sbml/level3/version1/layout/version1";
        let math_ns = "http://www.w3.org/1998/Math/MathML";

        let mut w = XmlWriter::new(xmlwriter::Options::default());
        w.start_element("sbml");
        w.write_attribute("xmlns", sbml_ns);
        w.write_attribute("level", "3");
        w.write_attribute("version", "2");

        // require the qual extension
        w.write_attribute("xmlns:qual", qual_ns);
        w.write_attribute("qual:required", "true");

        if model.layout.is_some() {
            // Layout information is available but not required
            w.write_attribute("xmlns:layout", layout_ns);
            w.write_attribute("layout:required", "false");
        }

        w.start_element("model");
        w.write_attribute("id", "model_id");

        // The single compartment
        w.start_element("listOfCompartments");
        w.start_element("compartments");
        w.write_attribute("id", "comp1");
        w.write_attribute("constant", "true");
        w.end_element();
        w.end_element();

        // Layout information if available
        if model.get_layout().is_some() {
            w.start_element("layout:listOfLayouts");
            w.start_element("layout:layout");
            w.start_element("layout:dimensions");
            // TODO: add global dimensions
            w.end_element();
            w.start_element("layout:listOfAdditionalGraphicalObjects");
            for uid in model.components() {
                if let Some(bb) = model.get_bounding_box(*uid) {
                    w.start_element("layout:generalGlyph");
                    w.write_attribute("layout:reference", model.get_name(*uid));
                    w.start_element("layout:boundingBox");

                    w.start_element("layout:position");
                    w.write_attribute("layout:x", &bb.x);
                    w.write_attribute("layout:y", &bb.y);
                    w.end_element();
                    w.start_element("layout:dimensions");
                    w.write_attribute("layout:width", &bb.width);
                    w.write_attribute("layout:height", &bb.height);
                    w.end_element();

                    w.end_element();
                    w.end_element();
                }
            }
            w.end_element();
            w.end_element();
            w.end_element();
        }

        // List of qualitative species
        w.start_element("qual:listOfQualitativeSpecies");
        w.write_attribute("xmlns:qual", qual_ns);
        for uid in model.components() {
            let max = model.get_variables(*uid).len();
            w.start_element("qual:qualitativeSpecies");
            w.write_attribute("qual:id", model.get_name(*uid));
            w.write_attribute("qual:compartment", "comp1");
            // TODO: detect and annotate inputs nodes?
            w.write_attribute("qual:constant", "false");
            w.write_attribute("qual:maxLevel", &max);
            w.end_element();
        }
        w.end_element();

        // List of transitions
        w.start_element("qual:listOfTransitions");
        w.write_attribute("xmlns:qual", qual_ns);

        for uid in model.components() {
            let rule = model.rules.get(*uid).unwrap();
            let name = model.get_name(*uid);

            w.start_element("qual:transition");
            w.write_attribute("qual:id", &format!("tr_{}", name));

            w.start_element("qual:listOfInputs");
            // FIXME: list regulators
            w.end_element();

            // Each transition has a single output
            w.start_element("qual:listOfOutputs");
            w.start_element("qual:output");
            w.write_attribute("qual:id", &format!("tr_{}_out", name));
            w.write_attribute("qual:qualitativeSpecies", &name);
            w.write_attribute("qual:transitionEffect", "assignmentLevel");
            w.end_element();
            w.end_element();

            w.start_element("qual:listOfFunctionTerms");
            w.start_element("qual:defaultTerm");
            w.write_attribute("qual:resultLevel", "0");
            w.end_element();
            for assign in rule.assignments() {
                w.start_element("qual:functionTerm");
                w.write_attribute("qual:resultLevel", &assign.target);
                w.start_element("math");
                w.write_attribute("xmlns", math_ns);
                write_expr(model, assign.formula.convert_as::<Expr>().as_ref(), &mut w);
                w.end_element();
                w.end_element();
            }
            w.end_element();

            // end transition
            w.end_element();
        }

        w.end_element();

        write!(out, "{}", w.end_document())?;
        Ok(())
    }
}

fn write_atom(model: &QModel, vid: usize, w: &mut XmlWriter) {
    // TODO: Handle multivalued
    w.start_element("apply");

    w.start_element("geq");
    w.end_element();

    w.start_element("ci");
    w.write_text(model.get_name(vid));
    w.end_element();

    w.start_element("cn");
    w.write_attribute("type", "integer");
    w.write_text("1");
    w.end_element();

    w.end_element();
}

fn write_op(model: &QModel, op: Operator, children: &expr::Children, w: &mut XmlWriter) {
    w.start_element("apply");
    match op {
        Operator::AND => {
            w.start_element("and");
            w.end_element();
            write_children(model, children, w);
        }
        Operator::NAND => {
            w.start_element("not");
            w.end_element();
            write_op(model, Operator::AND, children, w);
        }
        Operator::OR => {
            w.start_element("or");
            w.end_element();
            write_children(model, children, w);
        }
        Operator::NOR => {
            w.start_element("not");
            w.end_element();
            write_op(model, Operator::OR, children, w);
        }
    }
    w.end_element();
}

fn write_children(model: &QModel, children: &expr::Children, w: &mut XmlWriter) {
    for e in children.data.iter() {
        write_expr(model, e, w)
    }
}

fn write_expr(model: &QModel, expr: &Expr, w: &mut XmlWriter) {
    match expr {
        Expr::ATOM(u) => {
            write_atom(model, *u, w);
        }
        Expr::NATOM(u) => {
            w.start_element("apply");
            w.start_element("not");
            w.end_element();
            write_atom(model, *u, w);
            w.end_element();
        }
        Expr::TRUE => {
            w.start_element("true");
            w.end_element();
        }
        Expr::FALSE => {
            w.start_element("false");
            w.end_element();
        }
        Expr::OPER(o, c) => {
            write_op(model, *o, c, w);
        }
    }
}

impl io::ParsingFormat for SBMLFormat {
    fn parse_into_model(&self, model: &mut QModel, expression: &str) -> EmptyLomakResult {
        SBMLParser::parse(model, expression)
    }
}

pub fn load_xml(expression: &str) -> Result<Document, ParseError> {
    let parsed = roxmltree::Document::parse(expression);

    match parsed {
        Err(e) => Err(ParseError::ParseXML(e)),
        Ok(d) => Ok(d),
    }
}

impl SBMLParser {
    fn parse(model: &mut QModel, expression: &str) -> EmptyLomakResult {
        let doc = load_xml(expression)?;
        let root = doc.root_element();
        let ns_core = root.default_namespace().unwrap();

        if !SBML_NS.is_match(ns_core) {
            let e = GenericError::new(format!("Not an SBML document namespace: {}", ns_core));
            return Err(LomakError::from(e));
        }

        let ns_qual = match root
            .namespaces()
            .iter()
            .find(|ns| QUAL_NS.is_match(ns.uri()))
            .map(|ns| ns.uri())
        {
            None => {
                let e = GenericError::new("Not a qualitative SBML model".to_string());
                return Err(LomakError::from(e));
            }
            Some(n) => n,
        };

        let root_model = match root.children().find(|n| n.has_tag_name("model")) {
            None => {
                let e = GenericError::new("This SBML file contains no model".to_string());
                return Err(LomakError::from(e));
            }
            Some(model) => model,
        };

        // listOfCompartments
        // TODO: warning for models with multiple compartments?

        // Add all qualitative species
        if let Some(species) = root_model
            .children()
            .find(|n| n.has_tag_name((ns_qual, "listOfQualitativeSpecies")))
        {
            SBMLParser::parse_species(ns_qual, model, species.children());
        }

        // Add transitions
        if let Some(transitions) = root_model
            .children()
            .find(|t| t.has_tag_name((ns_qual, "listOfTransitions")))
            .map(|n| n.children())
        {
            SBMLParser::parse_transitions(ns_qual, model, transitions)?;
        }

        // Load layout
        match root
            .namespaces()
            .iter()
            .find(|ns| LAYOUT_NS.is_match(ns.uri()))
            .map(|ns| ns.uri())
        {
            None => (),
            Some(ns) => {
                if let Some(Some(layout)) = root_model
                    .children()
                    .find(|n| n.has_tag_name((ns, "listOfLayouts")))
                    .map(|n| n.children().find(|n| n.has_tag_name((ns, "layout"))))
                {
                    SBMLParser::parse_layout(ns, model, &layout);
                };
            }
        };

        Ok(())
    }

    fn parse_species(ns: &str, model: &mut QModel, species: Children) {
        for n_qs in species {
            if !n_qs.has_tag_name("qualitativeSpecies") {
                continue;
            }

            // Create the main variable for this species
            let sid = n_qs.attribute((ns, "id")).unwrap();
            let uid = model.ensure(sid);

            // Retrieve the max level and create associated variables if needed
            if let Some(Ok(m)) = n_qs.attribute((ns, "maxLevel")).map(|v| v.parse()) {
                model.ensure_threshold(uid, m);
            }

            // Add the implicit self-loop on constant species
            if n_qs
                .attribute((ns, "constant"))
                .unwrap_or("false")
                .parse()
                .unwrap_or(false)
            {
                let variables: Vec<usize> = model.get_variables(uid).iter().copied().collect();
                for curid in variables {
                    model.push_var_rule(curid, Formula::from(Expr::ATOM(curid)));
                }
            }
        }
    }

    fn parse_transitions(
        ns: &str,
        model: &mut QModel,
        transitions: Children,
    ) -> CanFail<ParseError> {
        for n_tr in transitions {
            if !n_tr.has_tag_name("transition") {
                continue;
            }

            // listOfInputs > input ( qualitativeSpecies, sign, transitionEffect=none )
            // TODO: parse inputs for quality control (consistent sign)

            // Load the function controlling this transition
            let functions = match n_tr
                .children()
                .find(|n| n.has_tag_name("listOfFunctionTerms"))
            {
                None => continue, // a transition without any function is useless
                Some(f) => f,
            };

            let mut rules = vec![];

            // Add the default value if it is not 0
            if let Some(d) = functions.children().find(|n| n.has_tag_name("defaultTerm")) {
                if let Some(v) = SBMLParser::collect::<usize>(d.attribute((ns, "resultLevel"))) {
                    if v != 0 {
                        rules.push((v, Expr::TRUE));
                    }
                }
            }

            // > functionTerm ( resultLevel=1 ) > math > apply > ...
            for term in functions
                .children()
                .filter(|n| n.has_tag_name("functionTerm"))
            {
                let math = match term.children().find(|n| n.has_tag_name("math")) {
                    None => continue,
                    Some(m) => m.children().find(|n| n.has_tag_name("apply")).unwrap(),
                };
                let target: usize =
                    SBMLParser::collect(term.attribute((ns, "resultLevel"))).unwrap_or(0);
                let expr = SBMLParser::parse_math(model, &math)?;
                rules.push((target, expr));
            }

            // listOfOutputs > output (qualitativeSpecies, transitionEffect=assignmentLevel)
            let outputs = match n_tr
                .children()
                .find(|n| n.has_tag_name("listOfOutputs"))
                .map(|n| n.children().filter(|n| n.has_tag_name("output")))
            {
                None => continue, // a transition must have at least one output
                Some(o) => o,
            };

            // Apply the rules to all outputs
            for o in outputs {
                let target = match o
                    .attribute((ns, "qualitativeSpecies"))
                    .map(|t| model.get_handle(t))
                {
                    Some(Some(t)) => t,
                    _ => {
                        return Err(
                            GenericError::new("Could not identify the output".to_owned()).into(),
                        )
                    }
                };

                for (v, e) in rules.iter() {
                    Rc::make_mut(&mut model.rules).push(target, *v, Formula::from(e.clone()));
                }
            }
        }
        Ok(())
    }

    fn parse_math(model: &QModel, math: &Node) -> Result<Expr, ParseError> {
        let children: Vec<Node> = math.children().filter(|n| n.is_element()).collect();
        if children.is_empty() {
            return Err(GenericError::new("Missing content in mathml?".to_owned()).into());
        }

        let name = children.get(0).unwrap().tag_name().name();
        let params = &children[1..];
        match name {
            "eq" => SBMLParser::parse_comparison(model, Comparator::EQ, params),
            "neq" => SBMLParser::parse_comparison(model, Comparator::NEQ, params),
            "gt" => SBMLParser::parse_comparison(model, Comparator::GT, params),
            "geq" => SBMLParser::parse_comparison(model, Comparator::GEQ, params),
            "lt" => SBMLParser::parse_comparison(model, Comparator::LT, params),
            "leq" => SBMLParser::parse_comparison(model, Comparator::LEQ, params),
            "and" => SBMLParser::parse_operation(model, Operator::AND, params),
            "or" => SBMLParser::parse_operation(model, Operator::OR, params),
            "not" => SBMLParser::parse_not(model, params),
            "true" => Ok(Expr::TRUE),
            "false" => Ok(Expr::FALSE),
            _ => {
                return Err(GenericError::new(format!(
                    "Unsupported mathml tag: {} ({:?})",
                    name,
                    math.document()
                        .text_pos_at(children.get(0).unwrap().range().start)
                ))
                .into())
            }
        }
    }

    fn parse_comparison(
        model: &QModel,
        cmp: Comparator,
        params: &[Node],
    ) -> Result<Expr, ParseError> {
        let mut variable = None;
        let mut value = None;

        for n in params {
            match n.tag_name().name() {
                "ci" => variable = n.text(),
                "cn" => value = n.text(),
                _ => {
                    return Err(GenericError::new(format!(
                        "Unsupported element in comparison: {}",
                        n.tag_name().name()
                    ))
                    .into())
                }
            }
        }

        let var = match variable.map(|v| model.get_handle(v.trim())) {
            Some(Some(u)) => u,
            _ => {
                return Err(
                    GenericError::new(format!("Missing or unknown variable in {:?}", cmp)).into(),
                )
            }
        };

        let val = match value.map(|s| s.trim().parse::<usize>()) {
            Some(r) => r?,
            None => {
                return Err(
                    GenericError::new(format!("Missing associated value in {:?}", cmp)).into(),
                )
            }
        };

        cmp.get_expr(model, var, val)
    }

    fn parse_not(model: &QModel, params: &[Node]) -> Result<Expr, ParseError> {
        if params.len() != 1 {
            return Err(GenericError::new(format!(
                "Not operand should have a single child, found {}",
                params.len()
            ))
            .into());
        }

        let child = SBMLParser::parse_math(model, &params[0])?;
        Ok(child.not())
    }

    fn parse_operation(model: &QModel, op: Operator, params: &[Node]) -> Result<Expr, ParseError> {
        let mut children: Vec<Expr> = vec![];
        for c in params {
            let c = SBMLParser::parse_math(model, c)?;
            children.push(c);
        }
        let children = expr::Children {
            data: Rc::new(children),
        };
        Ok(Expr::OPER(op, children))
    }

    fn collect<T: FromStr>(value: Option<&str>) -> Option<T> {
        match value {
            None => None,
            Some(v) => v.parse::<T>().ok(),
        }
    }

    fn parse_layout(ns: &str, model: &mut QModel, layout: &Node) {
        // Do we need the dimension?
        if let Some(objects) = layout
            .children()
            .find(|n| n.has_tag_name((ns, "listOfAdditionalGraphicalObjects")))
        {
            for glyph in objects
                .children()
                .filter(|n| n.has_tag_name((ns, "generalGlyph")))
            {
                let uid = match glyph
                    .attribute((ns, "reference"))
                    .map(|r| model.get_handle(r))
                {
                    Some(Some(uid)) => uid,
                    _ => continue,
                };
                if let Some(bb) = glyph
                    .children()
                    .find(|n| n.has_tag_name((ns, "boundingBox")))
                {
                    let mut info = NodeLayoutInfo::default();
                    if let Some(pos) = bb.children().find(|n| n.has_tag_name((ns, "position"))) {
                        info.x = Self::collect(pos.attribute((ns, "x"))).unwrap_or(0.0) as usize;
                        info.y = Self::collect(pos.attribute((ns, "y"))).unwrap_or(0.0) as usize;
                    }
                    if let Some(pos) = bb.children().find(|n| n.has_tag_name((ns, "dimensions"))) {
                        info.width =
                            Self::collect(pos.attribute((ns, "width"))).unwrap_or(0.0) as u8;
                        info.height =
                            Self::collect(pos.attribute((ns, "height"))).unwrap_or(0.0) as u8;
                    }
                    model.set_bounding_box(uid, info);
                }
            }
        }
    }
}
