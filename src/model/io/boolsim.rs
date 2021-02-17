use std::io::Write;

use pest::iterators::*;
use pest::Parser;

use crate::func::expr::{Expr, Operator};
use crate::func::implicant::Implicants;
use crate::func::Formula;
use crate::helper::error::{EmptyLomakResult, LomakResult};
use crate::model::io::Format;
use crate::model::QModel;
use crate::model::{io, GroupedVariables};

#[derive(Parser)]
#[grammar_inline = r####"
file  =  { SOI ~ NEWLINE* ~ ("targets" ~ "," ~ "factors" ~ NEWLINE*)? ~ rule* ~ EOI }
sxpr  =  { SOI ~ expr ~ EOI }
rule  =  { expr ~ "->" ~ lit ~ NEWLINE* }
expr  =  { ( term ~ ( "&" ~ term )* )? }
term  = _{ neg | lit }
neg   =  { ( "^" | "!" | "~") ~ lit }
lit   = @{ uid }
value =  { ASCII_DIGIT }
uid   = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }

WHITESPACE = _{ " " | "\t" }
COMMENT = _{ "#" ~ (!NEWLINE ~ ANY)* ~ NEWLINE? }

"####]
pub struct BoolSimParser;

#[derive(Default)]
pub struct BoolSimFormat;

impl Format for BoolSimFormat {
    fn description(&self) -> &str {
        "List of Boolean functions used by boolSim/genYsis"
    }
}

impl BoolSimFormat {
    fn load_expr(&self, model: &mut QModel, expr: Pair<Rule>) -> Expr {
        let rule = expr.as_rule();
        match rule {
            Rule::lit => Expr::ATOM(model.ensure(expr.as_str())),
            _ => {
                let mut content = expr.into_inner().map(|e| self.load_expr(model, e));
                match rule {
                    Rule::expr => Operator::AND.join(&mut content),
                    Rule::neg => content.next().unwrap().not(),
                    // Other rules are outside of scope or hidden
                    _ => panic!("Parsing tokens should not get there"),
                }
            }
        }
    }

    fn parse_formula(&self, model: &mut QModel, formula: &str) -> LomakResult<Expr> {
        let mut ptree = BoolSimParser::parse(Rule::sxpr, formula)?;
        let expr = ptree.next().unwrap().into_inner().next().unwrap();
        let expr = self.load_expr(model, expr);
        Ok(expr)
    }
}

impl io::ParsingFormat for BoolSimFormat {
    fn parse_into_model(&self, model: &mut QModel, expression: &str) -> EmptyLomakResult {
        let mut ptree = BoolSimParser::parse(Rule::file, expression)?;

        // Load all lines to restore the component order
        let ptree = ptree.next().unwrap();
        let mut expressions = vec![];
        for record in ptree.into_inner() {
            match record.as_rule() {
                Rule::rule => {
                    let mut inner = record.into_inner();
                    let target = inner.next().unwrap().as_str();
                    let target = model.ensure(target);
                    expressions.push((target, inner.next().unwrap()));
                }
                Rule::EOI => (),
                _ => panic!("Should not get there!"),
            }
        }

        // Parse all expressions
        for (vid, e) in expressions {
            let expr = self.load_expr(model, e);
            model.push_var_rule(vid, Formula::from(expr));
        }

        Ok(())
    }
}

impl io::SavingFormat for BoolSimFormat {
    fn write_rules(&self, model: &QModel, out: &mut dyn Write) -> EmptyLomakResult {
        for vid in model.variables() {
            let paths: Implicants = model.get_var_rule(*vid).prime_implicants();
            for _func in paths.iter() {
                // FIXME: write boolsim
                writeln!(out, "-> {}", model.get_name(*vid))?;
            }
        }

        Ok(())
    }
}
