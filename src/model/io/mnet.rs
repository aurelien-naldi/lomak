use std::io::{Error, Write};

use pest::iterators::*;
use pest::Parser;

use crate::func::expr::{Expr, NamedExpr, Operator};
use crate::func::Formula;
use crate::model::{io, GroupedVariables};
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
lit   = ${ uid ~ (":" ~ value)?  }
value =  { ASCII_DIGIT }
uid   = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }

WHITESPACE = _{ " " | "\t" }
COMMENT = _{ "#" ~ (!NEWLINE ~ ANY)* ~ NEWLINE* }

"####]
pub struct MNETParser;

pub struct MNETFormat;

impl MNETFormat {
    pub fn new() -> MNETFormat {
        MNETFormat {}
    }

    fn load_expr(&self, model: &mut QModel, expr: Pair<Rule>) -> Expr {
        let rule = expr.as_rule();
        match rule {
            Rule::bt => Expr::TRUE,
            Rule::bf => Expr::FALSE,
            Rule::lit => Expr::ATOM(self.load_lit(model, expr)),
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

    fn load_lit(&self, model: &mut QModel, expr: Pair<Rule>) -> usize {
        let mut expr = expr.into_inner();
        let component = expr.next().unwrap().as_str();
        let value = match expr.next() {
            None => 1,
            Some(e) => e.as_str().parse().unwrap(),
        };
        model.ensure_variable(component, value)
    }
}

impl io::ParsingFormat for MNETFormat {
    fn parse_rules(&self, model: &mut QModel, expression: &str) {
        let ptree = MNETParser::parse(Rule::file, expression);

        if let Err(err) = ptree {
            println!("Parsing error: {}", err);
            return;
        }

        // Load all lines to restore the component order
        let ptree = ptree.unwrap().next().unwrap();
        let mut expressions = vec![];
        for record in ptree.into_inner() {
            match record.as_rule() {
                Rule::rule => {
                    let mut inner = record.into_inner();
                    let target = inner.next().unwrap();
                    let var = self.load_lit(model, target);
                    expressions.push((var, inner.next().unwrap()));
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
    }

    fn parse_formula(&self, model: &mut QModel, formula: &str) -> Result<Expr, String> {
        let ptree = MNETParser::parse(Rule::sxpr, formula);
        match ptree {
            Err(s) => Err(format!("Parsing error: {}", s)),
            Ok(mut ptree) => {
                let expr = ptree.next().unwrap().into_inner().next().unwrap();
                let expr = self.load_expr(model, expr);
                Ok(expr)
            }
        }
    }
}

impl io::SavingFormat for MNETFormat {
    fn write_rules(&self, model: &QModel, out: &mut dyn Write) -> Result<(), Error> {
        for cid in model.components() {
            let rule = model.cpt_rules.get(cid).unwrap();
            let name = model.get_cpt_name(*cid);
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
