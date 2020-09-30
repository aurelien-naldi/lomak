use std::fmt;
use std::vec::Vec;

use bit_set::BitSet;

use crate::func::expr::Expr;
use crate::func::pattern::Pattern;
use crate::func::state::State;
use crate::func::VariableNamer;
use crate::func::*;
use std::ops::Deref;

#[derive(Clone, Default)]
pub struct Implicants {
    patterns: Vec<Pattern>,
}

// Use the deref trick to delegate some functions to the inner vector
impl Deref for Implicants {
    type Target = Vec<Pattern>;
    fn deref(&self) -> &Vec<Pattern> {
        &self.patterns
    }
}

impl Implicants {
    /// Create a new list of implicants containing a single unrestricted implicant.
    /// The new list corresponds to the "true" function.
    pub fn new() -> Implicants {
        Implicants {
            patterns: vec![Pattern::new()],
        }
    }

    /// Remove all implicants from this list.
    /// The resulting empty list corresponds to the "false" function.
    pub fn clear(&mut self) {
        self.patterns.clear();
    }

    /// Retrieve all literals (including their negation) from this list of implicants.
    /// The result is represented as a pattern, which can contain conflicts if the same variables is
    /// fixed at different values in different implicants.
    fn get_literals(&self) -> Pattern {
        let mut lits = Pattern::new();
        for l in self.patterns.iter() {
            lits.add_constraints_from(l);
        }
        lits
    }

    /// Retrieve all unsigned regulators from this list of implicants.
    pub fn get_regulators(&self) -> BitSet {
        let mut lits = BitSet::new();
        for l in self.patterns.iter() {
            lits.union_with(l.positive());
            lits.union_with(l.negative());
        }
        lits
    }

    pub fn merge_raw(&mut self, next: &Implicants) {
        // Tag the subsumed paths
        let mut s_subsumed = BitSet::new();
        let mut n_subsumed = BitSet::new();

        'outer: for (i, b) in self.patterns.iter().enumerate() {
            for (j, t) in next.patterns.iter().enumerate() {
                if n_subsumed.contains(j) {
                    continue;
                }
                if b.contains(&t) {
                    n_subsumed.insert(j);
                } else if t.contains(&b) {
                    s_subsumed.insert(i);
                    continue 'outer;
                }
            }
        }

        //        let paths = self.paths.into_iter().enumerate().filter(|(i,x)|!s_subsumed.contains(*i)).map(|(i,x)|x).collect();
        //        self.paths = paths;

        // Look for potential conflicts
        let s_lits = self.get_literals();
        let n_lits = next.get_literals();
        let conflicts = s_lits.conflicts(&n_lits);
        let mut c_subsumed = BitSet::new();
        let mut cpaths: Vec<Pattern> = Vec::new();
        if !conflicts.is_unrestricted() {
            // Another round to search for conflict-solving patterns
            for (i, b) in self.patterns.iter().enumerate() {
                if s_subsumed.contains(i) {
                    continue;
                }
                'inner: for (j, t) in next.patterns.iter().enumerate() {
                    if n_subsumed.contains(j) {
                        continue;
                    }
                    if b.conflicts(&t).len() == 1 {
                        // Generate a new pattern solving the conflict, and check if it subsumes it's parents
                        let mut new_path = b.clone();
                        new_path.merge_with(t);
                        /*
                                                if new_path.len() == b.len() - 1 {
                                                    s_subsumed.insert(i);
                                                }
                                                if new_path.len() == t.len() - 1 {
                                                    n_subsumed.insert(j);
                                                }
                        */
                        // Found a new candidate, check that it is new and if it subsumes an existing one
                        for (k, x) in cpaths.iter().enumerate() {
                            if x.contains(&new_path) {
                                // continue without adding this useless item
                                continue 'inner;
                            }
                            if new_path.contains(x) {
                                c_subsumed.insert(k);
                            }
                        }
                        cpaths.push(new_path);
                    }
                }
            }
        }

        // TODO: initialize directly with the right capacity
        let mut npaths = Vec::with_capacity(self.len() + next.len());
        for i in 0..self.len() {
            if !s_subsumed.contains(i) {
                // TODO: could we avoid cloning here?
                npaths.push(self.patterns[i].clone());
            }
        }
        for i in 0..next.len() {
            if !n_subsumed.contains(i) {
                // TODO: could we avoid cloning here?
                npaths.push(next.patterns[i].clone());
            }
        }

        //        println!("MERGE kept {} path out of {}+{}", npaths.len(), self.len(), next.len());
        self.patterns = npaths;

        // Integrate the new conflict-solving patterns in the result
        if !cpaths.is_empty() {
            let cpaths = cpaths
                .into_iter()
                .enumerate()
                .filter(|(i, _x)| !c_subsumed.contains(*i))
                .map(|(_i, x)| x)
                .collect();
            let cnext = Implicants { patterns: cpaths };
            //            println!("####### {} new paths", cnext.len());
            self.merge_raw(&cnext);
        }
    }

    /// Remove all paths contained in another list of implicants
    pub fn substract(&mut self, other: &Implicants) {
        self.patterns.retain(|b| !other.contains_path(&b));
    }

    fn contains_path(&self, path: &Pattern) -> bool {
        for b in &self.patterns {
            if b.contains(path) {
                return true;
            }
        }
        false
    }

    /// Look for the extended node in all paths, extend them if needed.
    /// Trivially and properly extended paths are stored separately for final filtering
    pub fn extend_literal(&mut self, idx: usize, value: bool) {
        let n = self.patterns.len();
        let mut trivial: Vec<Pattern> = Vec::with_capacity(n);
        let mut extended: Vec<Pattern> = Vec::with_capacity(n);
        for b in &self.patterns {
            b.extend_sort_literal(idx, value, &mut trivial, &mut extended);
        }

        // Eliminate subsumed paths
        // selected = trivial ∪ ( extended −{ P | ∃ P' ( P' ∈ trivial ∧ P' ⊂ P )})
        let mut selected: Vec<Pattern> = trivial.clone();
        'outer: for b in &extended {
            for t in &trivial {
                if t.contains(b) {
                    continue 'outer;
                }
            }
            // TODO: could we avoid cloning here?
            selected.push(b.clone());
        }

        self.patterns = selected
    }

    pub fn to_json(&self, namer: &dyn VariableNamer) {
        print!("    [");
        let mut first = true;
        for p in self.patterns.iter() {
            if first {
                first = false;
            } else {
                print!(",");
            }
            p.to_json(namer);
        }
        print!("]");
    }

    /// Generate a function based on the prime implicants
    pub fn to_expr(&self) -> Expr {
        let mut expr = Expr::FALSE;
        for p in self.patterns.iter() {
            expr = expr.and(&p.to_expr());
        }
        expr
    }

    /// Test if one of the implicants contains the given pattern.
    /// This indicates that all states of the given pattern satisfy the function
    /// represented by this list of implicants.
    ///
    /// If this is false, some state(s) in the pattern may still satisfy it.
    pub fn covers_pattern(&self, pattern: &Pattern) -> bool {
        for p in self.patterns.iter() {
            // FIXME: double check the inclusion test
            if p.contains(pattern) {
                return true;
            }
        }
        false
    }

    /// Test if at least one of the implicants overlaps with the given pattern.
    /// This indicates that at least one state of the given pattern satisfies the function
    /// represented by this list of implicants.
    ///
    /// Note that the function may still evaluate to false for some other state of the given pattern.
    pub fn eval_in_pattern(&self, pattern: &Pattern) -> bool {
        for p in self.patterns.iter() {
            if p.conflicts(pattern).is_unrestricted() {
                return true;
            }
        }
        false
    }
}

impl fmt::Display for Implicants {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in &self.patterns {
            writeln!(f, "{}", i)?;
        }
        write!(f, "")
    }
}

impl BoolRepr for Implicants {
    fn into_repr(self) -> Repr {
        Repr::PRIMES(Rc::new(self))
    }

    fn eval(&self, state: &State) -> bool {
        for p in self.patterns.iter() {
            if p.contains_state(state) {
                return true;
            }
        }
        false
    }
}

impl FromBoolRepr for Implicants {
    fn convert(repr: &Repr) -> Rc<Self> {
        match repr {
            Repr::PRIMES(p) => p.clone(),
            Repr::GEN(g) => Rc::new(g.to_expr().prime_implicants()),
            Repr::EXPR(e) => Rc::new(e.prime_implicants()),
        }
    }

    fn is_converted(repr: &Repr) -> bool {
        match repr {
            Repr::PRIMES(_) => true,
            _ => false,
        }
    }

    fn rc_to_repr(rc: Rc<Self>) -> Repr {
        Repr::PRIMES(rc)
    }
}

#[cfg(test)]
mod tests {
    use crate::func::implicant::*;

    #[test]
    fn test_implicants() {
        let mut a = Pattern::from_str("--0-1--00-");
        let mut b = Pattern::from_str("0-0-11-00-");
        let mut c = Pattern::from_str("0-1-11-00-");

        let mut implicants = Implicants::new();

        assert_eq!(implicants.eval_in_pattern(&a), true);
        assert_eq!(implicants.covers_pattern(&a), true);
    }
}