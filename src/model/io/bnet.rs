use std::io::Write;

use pest::iterators::*;
use pest::Parser;

use crate::func::expr::{Expr, NamedExpr, Operator};
use crate::func::Formula;
use crate::helper::error::EmptyLomakResult;
use crate::model::io::Format;
use crate::model::QModel;
use crate::model::{io, GroupedVariables};

#[derive(Parser)]
#[grammar_inline = r####"
file  =  { SOI ~ NEWLINE* ~ ("targets" ~ "," ~ "factors" ~ NEWLINE*)? ~ rule* ~ EOI }
sxpr  =  { SOI ~ expr ~ EOI }
rule  =  { lit ~ "," ~ expr  ~ NEWLINE* }
expr  = _{ disj }
disj  =  { conj ~ ( "|"  ~ conj )* }
conj  =  { term ~ ( "&" ~ term )* }
term  = _{ neg | grp }
neg   =  { ("!" | "~") ~ grp }
grp   = _{ neg | bt | bf | lit | "(" ~ expr ~ ")" }
bt    =  { ^"true" | "1" }
bf    =  { ^"false" | "0" }
lit   = @{ uid }
value =  { ASCII_DIGIT }
uid   = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }

WHITESPACE = _{ " " | "\t" }
COMMENT = _{ "#" ~ (!NEWLINE ~ ANY)* ~ NEWLINE? }

"####]
pub struct BNETParser;

#[derive(Default)]
pub struct BNETFormat;

impl Format for BNETFormat {
    fn description(&self) -> &str {
        "List of Boolean functions used by (Py)BoolNet"
    }
}

impl BNETFormat {
    fn load_expr(&self, model: &mut QModel, expr: Pair<Rule>) -> Expr {
        let rule = expr.as_rule();
        match rule {
            Rule::bt => Expr::TRUE,
            Rule::bf => Expr::FALSE,
            Rule::lit => Expr::ATOM(model.ensure(expr.as_str())),
            _ => {
                let mut content = expr.into_inner().map(|e| self.load_expr(model, e));
                match rule {
                    Rule::conj => Operator::AND.join(&mut content),
                    Rule::disj => Operator::OR.join(&mut content),
                    Rule::neg => content.next().unwrap().not(),
                    Rule::expr => content.next().unwrap(),
                    // Other rules are outside of scope or hidden
                    _ => panic!("Parsing tokens should not get there"),
                }
            }
        }
    }
}

impl io::ParsingFormat for BNETFormat {
    fn parse_into_model(&self, model: &mut QModel, expression: &str) -> EmptyLomakResult {
        let mut ptree = BNETParser::parse(Rule::file, expression)?;

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

impl io::SavingFormat for BNETFormat {
    fn write_rules(&self, model: &QModel, out: &mut dyn Write) -> EmptyLomakResult {
        for vid in model.variables() {
            let func: Expr = model.get_var_rule(*vid);
            write!(out, "{}, ", model.get_name(*vid))?;
            match func {
                Expr::TRUE => writeln!(out, "1")?,
                Expr::FALSE => writeln!(out, "0")?,
                _ => writeln!(
                    out,
                    "{}",
                    NamedExpr {
                        expr: &func,
                        namer: model,
                    }
                )?,
            }
        }

        Ok(())
    }
}
