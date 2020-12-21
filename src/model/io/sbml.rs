use crate::func::expr::Expr;
use crate::func::Formula;
use crate::model::{io, GroupedVariables, QModel};
use std::io::Write;

use crate::helper::error::{EmptyLomakResult, GenericError, LomakError, ParseError};
use regex::Regex;
use roxmltree::{Children, Document, Node};

const BASE_NS: &'static str = r"http://www.sbml.org/sbml/level3/version(\d)";

lazy_static! {
    static ref SBML_NS: Regex = Regex::new(&(format!(r"{}/core", BASE_NS))).unwrap();
    static ref QUAL_NS: Regex = Regex::new(&(format!(r"{}/qual/version(\d)", BASE_NS))).unwrap();
    static ref LAYOUT_NS: Regex =
        Regex::new(&(format!(r"{}/layout/version(\d)", BASE_NS))).unwrap();
}

pub struct SBMLFormat;
pub struct SBMLParser;

impl SBMLFormat {
    pub fn new() -> SBMLFormat {
        SBMLFormat {}
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

    fn parse_transitions(ns: &str, model: &mut QModel, transitions: Children) {
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
            let basal_level = match functions.children().find(|n| n.has_tag_name("defaultTerm")) {
                None => 0,
                Some(v) => {
                    // FIXME: retrieve the value
                    0
                }
            };
            // > functionTerm ( resultLevel=1 ) > math > apply > ...

            // listOfOutputs > output (qualitativeSpecies, transitionEffect=assignmentLevel)
            let outputs = match n_tr
                .children()
                .find(|n| n.has_tag_name("listOfOutputs"))
                .map(|n| n.children().filter(|n| n.has_tag_name("output")))
            {
                None => continue, // a transition must have at least one output
                Some(o) => o,
            };
            for o in outputs {
                // Apply the function to all outputs
            }
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
