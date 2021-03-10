use std::fmt;
use std::vec::Vec;

use bit_set::BitSet;

use crate::func::expr::Expr;
use crate::func::pattern::{Pattern, PatternRelation};
use crate::func::state::State;
use crate::func::VariableNamer;
use crate::func::*;
use std::ops::Deref;

const DEBUG: bool = false;

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

    /// Add a new candidate pattern.
    /// If this candidate is included in at least one existing pattern then do nothing.
    /// If it includes one or several existing patterns, then replace them.
    /// Also handle merged patterns which could arise
    fn add_candidate(&mut self, c: Pattern) {
        let mut subsumed = BitSet::new();
        let mut candidates = Vec::new();

        for (i, p) in self.patterns.iter().enumerate() {
            match p.relate(&c) {
                PatternRelation::Disjoint => {}
                PatternRelation::Overlap => {}
                PatternRelation::Contains => {
                    return;
                }
                PatternRelation::Identical => {
                    return;
                }
                PatternRelation::Contained => {
                    subsumed.insert(i);
                }
                PatternRelation::JoinBoth(m) => {
                    return self.add_candidate(m);
                }
                PatternRelation::JoinFirst(m) => {
                    subsumed.insert(i);
                    candidates.push(m);
                }
                PatternRelation::JoinSecond(m) => {
                    return self.add_candidate(m);
                }
                PatternRelation::JoinOverlap(m) => {
                    candidates.push(m);
                }
            }
        }

        // eliminate subsumed patterns
        let mut idx = 0;
        self.patterns.retain(|_| {
            idx += 1;
            !subsumed.contains(idx - 1)
        });

        // Add the new pattern
        self.patterns.push(c);

        // Add potential new candidates
        for p in candidates {
            self.add_candidate(p)
        }
    }

    /// Check if a new pattern is already covered by this list
    fn covers(&self, p: &Pattern) -> bool {
        self.patterns.iter().any(|c| c.contains(p))
    }

    pub fn merge_raw(&mut self, next: &Implicants) {
        // Tag the subsumed paths
        let mut s_subsumed = BitSet::new();
        let mut n_subsumed = BitSet::new();

        // Store new candidate patterns
        let mut candidates = Implicants::default();

        'outer: for (i, b) in self.patterns.iter().enumerate() {
            for (j, t) in next.patterns.iter().enumerate() {
                if n_subsumed.contains(j) {
                    continue;
                }

                match b.relate(&t) {
                    PatternRelation::Disjoint => {}
                    PatternRelation::Overlap => {}
                    PatternRelation::Contains => {
                        n_subsumed.insert(j);
                    }
                    PatternRelation::Identical => {
                        n_subsumed.insert(j);
                    }
                    PatternRelation::Contained => {
                        s_subsumed.insert(i);
                        continue 'outer;
                    }
                    PatternRelation::JoinBoth(m) => {
                        candidates.add_candidate(m);
                        s_subsumed.insert(i);
                        n_subsumed.insert(j);
                        continue 'outer;
                    }
                    PatternRelation::JoinFirst(m) => {
                        s_subsumed.insert(i);
                        if !next.covers(&m) {
                            candidates.add_candidate(m);
                        }
                        continue 'outer;
                    }
                    PatternRelation::JoinSecond(m) => {
                        n_subsumed.insert(j);
                        if !self.covers(&m) {
                            candidates.add_candidate(m);
                        }
                    }
                    PatternRelation::JoinOverlap(m) => {
                        if !self.covers(&m) && !next.covers(&m) {
                            candidates.add_candidate(m);
                        }
                    }
                }
            }
        }

        // TODO: remove the debug bloc after more testing
        if DEBUG && !candidates.patterns.is_empty() {
            println!("DEBUG MERGE...");
            for (s, p) in self
                .patterns
                .iter()
                .enumerate()
                .map(|(i, p)| (s_subsumed.contains(i), p))
            {
                let s = if s { "*" } else { " " };
                println!("[{}] {}", s, p);
            }
            println!("------------");
            for (s, p) in next
                .patterns
                .iter()
                .enumerate()
                .map(|(i, p)| (n_subsumed.contains(i), p))
            {
                let s = if s { "*" } else { " " };
                println!("[{}] {}", s, p);
            }
            println!("------------");
            println!("CANDIDATES:");
            print!("{}", candidates);
        }

        // eliminate subsumed patterns
        let mut idx = 0;
        self.patterns.retain(|_| {
            idx += 1;
            !s_subsumed.contains(idx - 1)
        });

        // Add new patterns from the other list
        for (_, p) in next
            .patterns
            .iter()
            .enumerate()
            .filter(|(i, _)| !n_subsumed.contains(*i))
        {
            self.patterns.push(p.clone());
        }

        // Integrate the new conflict-solving patterns in the result
        if !candidates.patterns.is_empty() {
            self.merge_raw(&candidates);
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
        matches!(repr, Repr::PRIMES(_))
    }

    fn rc_to_repr(rc: Rc<Self>) -> Repr {
        Repr::PRIMES(rc)
    }
}

#[cfg(test)]
mod tests {
    use crate::func::implicant::*;
    use std::str::FromStr;

    #[test]
    fn test_implicants() {
        let a = Pattern::from_str("--0-1--00-").unwrap();
        let b = Pattern::from_str("0-0-11-00-").unwrap();
        let c = Pattern::from_str("0-1-11-00-").unwrap();

        let implicants = Implicants::new();

        assert_eq!(implicants.eval_in_pattern(&a), true);
        assert_eq!(implicants.covers_pattern(&a), true);
    }

    #[test]
    fn test_eliminated_candidates() {
        let v1 = Expr::ATOM(1);
        let v2 = Expr::ATOM(2);
        let v3 = Expr::ATOM(3);

        let expr = v1
            .and(&v2)
            .and(&v3)
            .or(&v1.not().and(&v2.not()).and(&v3.not()));
        let pi = expr.prime_implicants();

        let nexpr = expr.not();
        let npi = nexpr.prime_implicants();
    }
}
