use std::io::Write;

use pest::iterators::*;
use pest::Parser;

use crate::func::expr::{Expr, NamedExpr, Operator};
use crate::func::Formula;
use crate::helper::error::{EmptyLomakResult, LomakResult};
use crate::model::io::Format;
use crate::model::QModel;
use crate::model::{io, GroupedVariables};

#[derive(Parser)]
#[grammar_inline = r####"
file  =  { SOI ~ NEWLINE* ~ rule* ~ EOI }
sxpr  =  { SOI ~ expr ~ EOI }
rule  =  { lit ~ "<-" ~ expr  ~ NEWLINE* }
expr  = _{ disj }
disj  =  { conj ~ ( "|"  ~ conj )* }
conj  =  { term ~ ( "&" ~ term )* }
term  = _{ neg | grp }
neg   =  { ("!" | "~") ~ grp }
grp   = _{ neg | bt | bf | lit | "(" ~ expr ~ ")" }
bt    =  { ^"true" | "1" }
bf    =  { ^"false" | "0" }
lit   = ${ uid ~ (":" ~ value)?  }
value =  { ASCII_DIGIT }
uid   = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }

WHITESPACE = _{ " " | "\t" }
COMMENT = _{ "#" ~ (!NEWLINE ~ ANY)* ~ NEWLINE* }

"####]
pub struct MNETParser;

#[derive(Default)]
pub struct MNETFormat;

impl Format for MNETFormat {
    fn description(&self) -> &str {
        "List of Multi-valued functions"
    }
}

impl MNETFormat {
    fn load_expr(model: &mut QModel, expr: Pair<Rule>) -> Expr {
        let rule = expr.as_rule();
        match rule {
            Rule::bt => Expr::TRUE,
            Rule::bf => Expr::FALSE,
            Rule::lit => Expr::ATOM(Self::load_lit(model, expr)),
            _ => {
                let mut content = expr.into_inner().map(|e| Self::load_expr(model, e));
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

    fn load_lit(model: &mut QModel, expr: Pair<Rule>) -> usize {
        let mut expr = expr.into_inner();
        let cid = model.ensure(expr.next().unwrap().as_str());
        if let Some(e) = expr.next() {
            return model.ensure_threshold(cid, e.as_str().parse().unwrap());
        }
        cid
    }

    fn parse_formula(model: &mut QModel, formula: &str) -> LomakResult<Expr> {
        let mut ptree = MNETParser::parse(Rule::sxpr, formula)?;
        let expr = ptree.next().unwrap().into_inner().next().unwrap();
        let expr = Self::load_expr(model, expr);
        Ok(expr)
    }
}

impl io::ParsingFormat for MNETFormat {
    fn parse_into_model(&self, model: &mut QModel, expression: &str) -> EmptyLomakResult {
        let mut ptree = MNETParser::parse(Rule::file, expression)?;

        // Load all lines to restore the component order
        let ptree = ptree.next().unwrap();
        let mut expressions = vec![];
        for record in ptree.into_inner() {
            match record.as_rule() {
                Rule::rule => {
                    let mut inner = record.into_inner();
                    let target = inner.next().unwrap();
                    let var = Self::load_lit(model, target);
                    expressions.push((var, inner.next().unwrap()));
                }
                Rule::EOI => (),
                _ => panic!("Should not get there!"),
            }
        }

        // Parse all expressions
        for (vid, e) in expressions {
            let expr = Self::load_expr(model, e);
            model.push_var_rule(vid, Formula::from(expr));
        }

        Ok(())
    }
}

impl io::SavingFormat for MNETFormat {
    fn write_rules(&self, model: &QModel, out: &mut dyn Write) -> EmptyLomakResult {
        for cid in model.components() {
            let rule = model.rules.get(*cid).unwrap();
            let name = model.get_name(*cid);
            for assign in rule.assignments.iter() {
                write!(out, "{}", name)?;
                if assign.target != 1 {
                    write!(out, ":{}", assign.target)?;
                }
                writeln!(
                    out,
                    "<- {}",
                    NamedExpr {
                        expr: &assign.formula.convert_as(),
                        namer: model,
                    }
                )?;
            }
        }

        Ok(())
    }
}
