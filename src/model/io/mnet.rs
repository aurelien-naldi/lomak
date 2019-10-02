use crate::model::io;

use pest::iterators::*;
use pest::Parser;
use std::io::{Error, Write};

use crate::func::expr::{Expr, NamedExpr, Operator};
use crate::func::Formula;
use crate::model::QModel;

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
lit   = @{ uid ~ (":" ~ value)?  }
value =  { ASCII_DIGIT }
uid   = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }

WHITESPACE = _{ " " | "\t" }
COMMENT = _{ "#" ~ (!NEWLINE ~ ANY)* ~ NEWLINE* }

"####]
pub struct MNETParser;

pub struct MNETFormat;

impl MNETFormat {
    pub fn new() -> MNETFormat {
        return MNETFormat {};
    }

    fn load_expr(&self, model: &mut dyn QModel, expr: Pair<Rule>) -> Expr {
        let rule = expr.as_rule();
        match rule {
            Rule::bt => Expr::TRUE,
            Rule::bf => Expr::FALSE,
            Rule::lit => Expr::ATOM(model.ensure_variable(expr.as_str(), 1)),
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

impl io::ParsingFormat for MNETFormat {
    fn parse_rules(&self, model: &mut dyn QModel, expression: &String) {
        let ptree = MNETParser::parse(Rule::file, expression);

        if ptree.is_err() {
            println!("Parsing error: {}", ptree.unwrap_err());
            return;
        }

        let ptree = ptree.unwrap().next().unwrap();
        for record in ptree.into_inner() {
            match record.as_rule() {
                Rule::rule => {
                    let mut inner = record.into_inner();
                    let target = inner.next().unwrap().as_str();
                    let target = model.ensure_variable(target, 1);
                    let expr = inner.next().unwrap();
                    let expr = self.load_expr(model, expr);
                    model.set_rule(target, 1, Formula::from(expr));
                }
                Rule::EOI => (),
                _ => panic!("Should not get there!"),
            }
        }
    }

    fn parse_formula(&self, model: &mut dyn QModel, formula: &str) -> Result<Expr, String> {
        let ptree = MNETParser::parse(Rule::sxpr, formula);
        match ptree {
            Err(s) => return Err(format!("Parsing error: {}", s)),
            Ok(mut ptree) => {
                let expr = ptree.next().unwrap().into_inner().next().unwrap();
                let expr = self.load_expr(model, expr);
                return Ok(expr);
            }
        }
    }
}

impl io::SavingFormat for MNETFormat {
    fn write_rules(&self, model: &dyn QModel, out: &mut dyn Write) -> Result<(), Error> {
        let namer = model.as_namer();
        for (_, c) in model.components() {
            for assign in &c.rule.assignments {
                write!(out, "{}", &c.name)?;
                if assign.target != 1 {
                    write!(out, ":{}", assign.target)?;
                }
                writeln!(
                    out,
                    "<- {}",
                    NamedExpr {
                        expr: &assign.formula.convert_as(),
                        namer: namer,
                    }
                )?;
            }
        }

        Ok(())
    }
}
