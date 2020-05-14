use std::fmt;
use std::vec::Vec;

use bit_set::BitSet;

use crate::func::expr::Expr;
use crate::func::state::State;
use crate::func::VariableNamer;
use crate::func::*;

#[derive(Clone, PartialEq, Eq, Default)]
pub struct LiteralSet {
    positive: BitSet,
    negative: BitSet,
}

#[derive(Clone, Default)]
pub struct Paths {
    paths: Vec<LiteralSet>,
}

impl Paths {
    pub fn new() -> Paths {
        Paths {
            paths: vec![LiteralSet::new()],
        }
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub fn items(&self) -> &Vec<LiteralSet> {
        &self.paths
    }

    pub fn clear(&mut self) {
        self.paths.clear();
    }

    fn get_literals(&self) -> LiteralSet {
        // TODO: cache this result
        let mut lits = LiteralSet::new();
        for l in self.paths.iter() {
            lits.union_with(l);
        }
        lits
    }

    pub fn merge_raw(&mut self, next: &Paths) {
        // Tag the subsumed paths
        let mut s_subsumed = BitSet::new();
        let mut n_subsumed = BitSet::new();

        'outer: for (i, b) in self.paths.iter().enumerate() {
            for (j, t) in next.paths.iter().enumerate() {
                if n_subsumed.contains(j) {
                    continue;
                }
                if t.is_subsumed_by(&b) {
                    n_subsumed.insert(j);
                } else if b.is_subsumed_by(&t) {
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
        let mut cpaths = Vec::new();
        if !conflicts.is_empty() {
            // Another round to search for conflict-solving patterns
            for (i, b) in self.paths.iter().enumerate() {
                if s_subsumed.contains(i) {
                    continue;
                }
                'inner: for (j, t) in next.paths.iter().enumerate() {
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
                            if new_path.is_subsumed_by(x) {
                                // continue without adding this useless item
                                continue 'inner;
                            }
                            if x.is_subsumed_by(&new_path) {
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
                npaths.push(self.paths[i].clone());
            }
        }
        for i in 0..next.len() {
            if !n_subsumed.contains(i) {
                // TODO: could we avoid cloning here?
                npaths.push(next.paths[i].clone());
            }
        }

        //        println!("MERGE kept {} path out of {}+{}", npaths.len(), self.len(), next.len());
        self.paths = npaths;

        // Integrate the new conflict-solving patterns in the result
        if !cpaths.is_empty() {
            let cpaths = cpaths
                .into_iter()
                .enumerate()
                .filter(|(i, _x)| !c_subsumed.contains(*i))
                .map(|(_i, x)| x)
                .collect();
            let cnext = Paths { paths: cpaths };
            //            println!("####### {} new paths", cnext.len());
            self.merge_raw(&cnext);
        }
    }

    /// Remove all paths contained in an other set
    pub fn substract(&mut self, other: &Paths) {
        self.paths.retain(|b| !other.contains_path(&b));
    }

    fn contains_path(&self, path: &LiteralSet) -> bool {
        for b in &self.paths {
            if path.is_subsumed_by(b) {
                return true;
            }
        }
        false
    }

    /// Look for the extended node in all paths, extend them if needed.
    /// Trivially and properly extended paths are stored separately for final filtering
    pub fn extend_literal(&mut self, idx: usize, value: bool) {
        let n = self.paths.len();
        let mut trivial: Vec<LiteralSet> = Vec::with_capacity(n);
        let mut extended: Vec<LiteralSet> = Vec::with_capacity(n);
        for b in &self.paths {
            b.extend_sort_literal(idx, value, &mut trivial, &mut extended);
        }

        // Eliminate subsumed paths
        // selected = trivial ∪ ( extended −{ P | ∃ P' ( P' ∈ trivial ∧ P' ⊂ P )})
        let mut selected: Vec<LiteralSet> = trivial.clone();
        'outer: for b in &extended {
            for t in &trivial {
                if b.is_subsumed_by(&t) {
                    continue 'outer;
                }
            }
            // TODO: could we avoid cloning here?
            selected.push(b.clone());
        }

        self.paths = selected
    }

    pub fn to_json(&self, namer: &dyn VariableNamer) {
        print!("    [");
        let mut first = true;
        for p in self.paths.iter() {
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
        for p in self.paths.iter() {
            expr = expr.and(&p.to_expr());
        }
        expr
    }
}

impl LiteralSet {
    pub fn new() -> LiteralSet {
        LiteralSet {
            positive: BitSet::new(),
            negative: BitSet::new(),
        }
    }

    pub fn with(uid: usize, neg: bool) -> LiteralSet {
        let mut l = LiteralSet::new();
        l.extend(uid, neg);
        l
    }

    pub fn extend(&mut self, uid: usize, neg: bool) {
        if neg {
            self.negative.insert(uid);
        } else {
            self.positive.insert(uid);
        }
    }

    /*
        pub fn restrict(&mut self, p: Paths) -> Option<Paths> {
            match p.len() {
                0 => { self.clear(); Option::None },
                1 => { self.union_with(p.paths.get(0).as_ref().unwrap()); Option::None },
                _ => Option::Some(p),
            }
        }
    */
    pub fn union_with(&mut self, other: &LiteralSet) {
        self.positive.union_with(&other.positive);
        self.negative.union_with(&other.negative);
    }

    pub fn merge_with(&mut self, other: &LiteralSet) {
        self.union_with(other);
        let mut conflicts = self.positive.clone();
        conflicts.intersect_with(&self.negative);
        if conflicts.len() != 1 {
            panic!("When calling this, there should be exactly one conflicting bit");
        }
        self.positive.difference_with(&conflicts);
        self.negative.difference_with(&conflicts);
    }

    #[allow(dead_code)]
    pub fn intersect_with(&mut self, other: &LiteralSet) {
        self.positive.intersect_with(&other.positive);
        self.negative.intersect_with(&other.negative);
    }

    pub fn conflicts(&self, other: &LiteralSet) -> LiteralSet {
        let mut cpn = self.positive.clone();
        cpn.intersect_with(&other.negative);
        let mut cnp = other.positive.clone();
        cnp.intersect_with(&self.negative);
        LiteralSet {
            positive: cpn,
            negative: cnp,
        }
    }

    pub fn reverse(&self) -> LiteralSet {
        LiteralSet {
            positive: self.negative.clone(),
            negative: self.positive.clone(),
        }
    }

    pub fn len(&self) -> usize {
        self.len_pos() + self.len_neg()
    }

    pub fn is_empty(&self) -> bool {
        self.positive.is_empty() && self.negative.is_empty()
    }

    pub fn len_pos(&self) -> usize {
        self.positive.len()
    }

    pub fn len_neg(&self) -> usize {
        self.negative.len()
    }

    /* The following methods are used for Paths extension in PI search */

    pub fn get(&self, idx: usize) -> i8 {
        if self.positive.contains(idx) {
            return 1;
        }
        if self.negative.contains(idx) {
            return 0;
        }
        -1
    }

    pub fn set_literal(&mut self, idx: usize, value: bool) {
        if value {
            self.negative.remove(idx);
            self.positive.insert(idx);
        } else {
            self.positive.remove(idx);
            self.negative.insert(idx);
        }
    }

    fn extend_sort_literal(
        &self,
        idx: usize,
        val: bool,
        trivials: &mut Vec<Self>,
        extended: &mut Vec<Self>,
    ) {
        let (conflict, trivial) = if val {
            (&self.negative, &self.positive)
        } else {
            (&self.positive, &self.negative)
        };
        // Reject conflicting paths
        if conflict.contains(idx) {
            return;
        }

        // Detect trivially extended paths
        if trivial.contains(idx) {
            trivials.push(self.clone());
            return;
        }

        // Otherwise, create a properly extended path
        let mut p = self.clone();
        p.set_literal(idx, val);
        extended.push(p);
    }

    /// Test if this set is contained in another set,
    /// i.e. if all constraints of the other set are also constraints of this set.
    pub fn is_subsumed_by(&self, other: &LiteralSet) -> bool {
        self.positive.is_superset(&other.positive) && self.negative.is_superset(&other.negative)
    }

    pub fn to_json(&self, namer: &dyn VariableNamer) {
        print!("{{");
        let mut first = true;
        for uid in self.positive.iter() {
            if first {
                first = false;
            } else {
                print!(",");
            }
            print!("\"{}\":1", namer.name(uid));
        }
        for uid in self.negative.iter() {
            if first {
                first = false;
            } else {
                print!(",");
            }
            print!("\"{}\":0", namer.name(uid));
        }
        print!("}}");
    }

    pub fn to_expr(&self) -> Expr {
        let mut expr = Expr::TRUE;
        for uid in self.positive.iter() {
            expr = expr.and(&Expr::ATOM(uid))
        }
        for uid in self.positive.iter() {
            expr = expr.and(&Expr::NATOM(uid))
        }
        expr
    }

    pub fn positive(&self) -> &BitSet {
        &self.positive
    }

    pub fn negative(&self) -> &BitSet {
        &self.negative
    }

    /// Build a new LiteralSet where only the provided positions are retained and mapped to the provided order
    pub fn filter_map(self, filter: &[usize]) -> Self {
        let mut result = LiteralSet::new();
        for (k, uid) in filter.iter().enumerate() {
            if self.positive.contains(*uid) {
                result.positive.insert(k);
            }
            if self.negative.contains(*uid) {
                result.negative.insert(k);
            }
        }
        result
    }

    /// Test if the pattern contains a specific state
    ///
    /// All variables fixed as positive in the pattern must
    /// be active in the state.
    /// All variables fixed as negative in the pattern must
    /// be inactive in the state.
    pub fn contains_state(&self, state: &State) -> bool {
        if state.is_superset(&self.positive) {
            let mut conflicts = state.clone();
            conflicts.intersect_with(&self.negative);
            if conflicts.is_empty() {
                return true;
            }
        }
        false
    }
}

impl fmt::Display for LiteralSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = vec![];
        for v in &self.positive {
            if result.len() <= v {
                for _ in result.len()..=v {
                    result.push('-');
                }
            }
            result[v] = '1';
        }
        for v in &self.negative {
            if result.len() <= v {
                for _ in result.len()..=v {
                    result.push('-');
                }
            }
            result[v] = '0';
        }
        let s: String = result.iter().collect();
        write!(f, "{}", &s)
    }
}

impl LiteralSet {
    pub fn filter_fmt(&self, f: &mut fmt::Formatter, uids: &Vec<usize>) -> fmt::Result {
        for u in uids.iter() {
            if self.positive.contains(*u) {
                if self.negative.contains(*u) {
                    write!(f, "@")?;
                } else {
                    write!(f, "1")?;
                }
            } else if self.negative.contains(*u) {
                write!(f, "0")?;
            } else {
                write!(f, "-")?;
            }
        }
        write!(f, "")
    }
}

impl fmt::Display for Paths {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in &self.paths {
            writeln!(f, "{}", i)?;
        }
        write!(f, "")
    }
}

impl BoolRepr for Paths {
    fn into_repr(self) -> Repr {
        Repr::PRIMES(Rc::new(self))
    }

    fn eval(&self, state: &State) -> bool {
        for p in self.paths.iter() {
            if p.contains_state(state) {
                return true;
            }
        }
        false
    }
}

impl FromBoolRepr for Paths {
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
