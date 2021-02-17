use crate::func::expr::{Expr, Operator};
use crate::func::expr;
use crate::func::Formula;
use crate::model::{io, GroupedVariables, QModel};
use std::io::Write;

use crate::helper::error::{EmptyLomakResult, GenericError, LomakError, ParseError, LomakResult, CanFail};
use regex::Regex;
use roxmltree::{Children, Document, Node};
use std::str::FromStr;
use crate::helper::error::ParseError::ParseXML;
use std::num::ParseIntError;
use crate::helper::error::LomakError::Generic;
use std::any::Any;
use std::rc::Rc;
use crate::func::Repr::EXPR;
use crate::model::io::Format;

const BASE_NS: &'static str = r"http://www.sbml.org/sbml/level3/version(\d)";

lazy_static! {
    static ref SBML_NS: Regex = Regex::new(&(format!(r"{}/core", BASE_NS))).unwrap();
    static ref QUAL_NS: Regex = Regex::new(&(format!(r"{}/qual/version(\d)", BASE_NS))).unwrap();
    static ref LAYOUT_NS: Regex =
        Regex::new(&(format!(r"{}/layout/version(\d)", BASE_NS))).unwrap();
}

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
        unimplemented!()
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
                let e = GenericError::new(format!("Not a qualitative SBML model"));
                return Err(LomakError::from(e));
            }
            Some(n) => n,
        };

        let root_model = match root.children().find(|n| n.has_tag_name("model")) {
            None => {
                let e = GenericError::new(format!("This SBML file contains no model"));
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
            SBMLParser::parse_transitions(ns_qual, model, transitions);
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
                    .map(|n| n.children().find(|n| n.has_tag_name("layout")))
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
                let variables: Vec<usize> = model.get_variables(uid).iter().map(|c| *c).collect();
                for curid in variables {
                    model.push_var_rule(curid, Formula::from(Expr::ATOM(curid)));
                }
            }
        }
    }

    fn parse_transitions(ns: &str, model: &mut QModel, transitions: Children) -> CanFail<ParseError> {
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

            let mut rules = vec!();

            // Add the default value if it is not 0
            if let Some(d) = functions
                .children()
                .find(|n| n.has_tag_name("defaultTerm")) {
                if let Some(v) = SBMLParser::collect::<usize>( d.attribute((ns,"resultLevel"))) {
                    if v != 0 {
                        rules.push( (v, Expr::TRUE));
                    }
                }
            }

            // > functionTerm ( resultLevel=1 ) > math > apply > ...
            for term in functions
                .children()
                .filter(|n| n.has_tag_name("functionTerm")) {
                let math = match term.children().find(|n|n.has_tag_name("math")) {
                    None => continue,
                    Some(m) => m.children().find(|n| n.has_tag_name("apply")).unwrap(),
                };
                let target: usize = SBMLParser::collect( term.attribute((ns, "resultLevel"))).unwrap_or(0);
                let expr = SBMLParser::parse_math( model, &math)?;
                rules.push( (target, expr));
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
                let target = match o.attribute((ns, "qualitativeSpecies")).map( |t| model.get_handle(t)) {
                    Some( Some(t)) => t,
                    _ => Err( GenericError::new("Could not identify the output".to_owned()) )?,
                };

                for (v,e) in rules.iter() {
                    model.rules.push(target, *v,Formula::from(e.clone()));
                }
            }
        }
        Ok(())
    }

    fn parse_math(model: &QModel, math: &Node) -> Result<Expr, ParseError> {
        let children: Vec<Node> = math.children().filter(|n|n.is_element()).collect();
        if children.len() < 1 {
            Err( GenericError::new("Missing content in mathml?".to_owned()) )?;
        }

        let name = children.get(0).unwrap().tag_name().name();
        let params = &children[1..];
        match name {
            "eq" => SBMLParser::parse_comparison(model, Comparison::EQ, params),
            "neq" => SBMLParser::parse_comparison(model, Comparison::NEQ, params),
            "gt" => SBMLParser::parse_comparison(model, Comparison::GT, params),
            "geq" => SBMLParser::parse_comparison(model, Comparison::GEQ, params),
            "lt" => SBMLParser::parse_comparison(model, Comparison::LT, params),
            "leq" => SBMLParser::parse_comparison(model, Comparison::LEQ, params),
            "and" => SBMLParser::parse_operation(model, Operator::AND, params),
            "or" => SBMLParser::parse_operation(model, Operator::OR, params),
            "not" => SBMLParser::parse_not(model, params),
            _ => Err( GenericError::new(format!( "Unsupported mathml tag: {}", name)) )?,
        }
    }

    fn parse_comparison(model: &QModel, cmp: Comparison, params: &[Node]) -> Result<Expr, ParseError> {
        let mut variable = None;
        let mut value = None;

        for n in params {
            match n.tag_name().name() {
                "ci" => variable = n.text(),
                "cn" => value = n.text(),
                _  => return Err( GenericError::new(format!("Unsupported element in comparison: {}", n.tag_name().name()) ))?,
            }
        }

        let var = match variable.map(|v| model.get_handle(v.trim())) {
            Some(Some(u)) => Ok(u),
            _ => Err( GenericError::new("Missing or unknown variable".to_owned()) ),
        }?;

        let val = match value.map( |s| s.trim().parse::<usize>()) {
            Some( r) => r?,
            None => Err( GenericError::new("Missing associated value".to_owned()) )?,
        };

        let (min,max) = match cmp {
            Comparison::EQ => {
                if val == 0 {
                    (None,Some(1))
                } else {
                    (Some(val),Some(val+1))
                }
            },
            Comparison::NEQ => {
                if val > 0 {
                    if let Some(n) = model.get_variable(var, val + 1) {
                        // The next value exists and the current one is at least 1, so it must exist
                        let c = model.get_variable(var, val).unwrap();
                        return Ok(Expr::ATOM(n).or(&Expr::NATOM(c)));
                    }
                }
                (Some(val+1),None)
            },
            Comparison::GEQ => {
                (Some(val), None)
            },
            Comparison::LEQ => {
                (None, Some(val+1))
            },
            Comparison::GT => {
                (Some(val+1), None)
            },
            Comparison::LT => {
                (None, Some(val))
            },
        };

        let emin = min.map( |v| model.get_variable(var,v).map(|u| Expr::ATOM(u)));
        let emax = max.map( |v| model.get_variable(var,v).map(|u| Expr::NATOM(u)));

        match (emin,emax) {
            (Some(Some(mn)), Some(Some(mx))) => Ok( mn.and(&mx) ),
            (Some(Some(e)), _) => Ok( e ),
            (_, Some(Some(e))) => Ok( e ),
            _ => Err( GenericError::new("Could not construct a constraint!".to_owned()))?,
        }
    }

    fn parse_not(model: &QModel, params: &[Node]) -> Result<Expr, ParseError> {
        if params.len() != 1 {
            return Err( GenericError::new("Not operand should have a single child".to_owned()) )?;
        }

        let child = SBMLParser::parse_math(model, &params[0])?;
        Ok(child.not())
    }

    fn parse_operation(model: &QModel, op: Operator, params: &[Node]) -> Result<Expr, ParseError> {
        let mut children: Vec<Expr> = vec!();
        for c in params {
            let c = SBMLParser::parse_math(model, c)?;
            children.push(c);
        }
        let children = expr::Children {
            data: Rc::new( children),
        };
        Ok(Expr::OPER(op, children))
    }

    fn collect<T: FromStr>(value: Option<&str>) -> Option<T> {
        match value {
            None => None,
            Some(v) => v.parse::<T>().ok(),
        }
    }

    fn parse_layout(ns: &str, model: &mut QModel, species: &Node) {
        // TODO: use layout information

        // listOfLayouts > layout
        // > dimension
        // > listOfAdditionalGraphicalObjects > generalGlyph (id, reference)
        // >> boundingBox > position (x,y) ; dimensions (width,height)
    }
}

#[derive(Debug)]
enum Comparison {
    EQ,
    NEQ,
    GT,
    GEQ,
    LT,
    LEQ,
}
