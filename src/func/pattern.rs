use std::fmt;
use std::vec::Vec;

use bit_set::BitSet;

use crate::func::expr::Expr;
use crate::func::state::State;
use crate::func::VariableNamer;

/// Patterns are subspaces in which a subset of variables are fixed (true or false).
/// They are represented as a pair of bitsets to store positive and negative variables.
/// In well-formed pattern, a variable should not be constrained to both values,
/// i.e. the intersection of both bitsets should be empty. However, some operations
/// on patterns do not prevent the creation of conflicts, either for performance reasons
/// or to use them to carry extra information.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct Pattern {
    positive: BitSet,
    negative: BitSet,
}

pub enum PatternState {
    TRUE,
    FALSE,
    ANY,
}

/// Describe the relation between two patterns and identify merged patterns when they exist
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PatternRelation {
    /// The patterns are identical
    Identical,
    /// The patterns are fully separated (at least two conflicts)
    Disjoint,
    /// The patterns overlap but do not include each other (no conflict)
    Overlap,
    /// The first pattern includes the second one
    Contains,
    /// The first pattern is included in the second one
    Contained,
    /// A joined pattern includes both patterns
    JoinBoth(Pattern),
    /// A joined pattern includes only the first pattern
    JoinFirst(Pattern),
    /// A joined pattern includes only the second pattern
    JoinSecond(Pattern),
    /// A joined pattern does not include any of the two patterns
    JoinOverlap(Pattern),
}

impl Pattern {
    /// Create an new pattern, without any restricted variables.
    /// This pattern corresponds to the full state space.
    pub fn new() -> Pattern {
        Pattern {
            positive: BitSet::new(),
            negative: BitSet::new(),
        }
    }

    /// Create a new pattern with a single restricted variable.
    /// This pattern corresponds to half of the full state space.
    pub fn with(uid: usize, value: bool) -> Pattern {
        let mut l = Pattern::new();
        l.set(uid, value);
        l
    }

    /// Create a new pattern and fix variables according to a string
    pub fn from_str(descr: &str) -> Pattern {
        let mut l = Pattern::new();
        for (idx, c) in descr.chars().enumerate() {
            match c {
                '0' => l.set(idx, false),
                '1' => l.set(idx, true),
                _ => (),
            }
        }
        l
    }

    /// Create a pattern restricted to a single state
    /// To properly set all variables fixed at 0, this requires
    /// an extra parameter giving the total number of variables.
    pub fn from_state(state: &State, len: usize) -> Pattern {
        let mut neg = BitSet::with_capacity(len);
        for idx in 0..len {
            if !state.contains(idx) {
                neg.insert(idx);
            }
        }
        Pattern {
            positive: state.clone(),
            negative: neg,
        }
    }

    /// Test if a given variable is fixed to a specific value
    pub fn is_fixed_at(&self, uid: usize, value: bool) -> bool {
        if value {
            &self.positive
        } else {
            &self.negative
        }
        .contains(uid)
    }

    /// Test if a given variable is fixed to any value.
    pub fn is_fixed(&self, uid: usize) -> bool {
        self.positive.contains(uid) || self.negative.contains(uid)
    }

    /// Fix a variable to a specific value.
    ///
    /// If this variable was free, this leads to a restriction of the pattern.
    /// If it was fixed to the same value, the pattern is unchanged.
    /// If it was fixed to the opposite value, the existing restriction
    /// is replaced by the new one (giving a mirror pattern).
    pub fn set(&mut self, uid: usize, value: bool) {
        if value {
            self.negative.remove(uid);
            self.positive.insert(uid);
        } else {
            self.positive.remove(uid);
            self.negative.insert(uid);
        }
    }

    /// Fix a set of variables to a specific value.
    /// This will replace variables set at opposing values.
    pub fn set_variables(&mut self, vars: &BitSet, value: bool) {
        if value {
            self.negative.difference_with(vars);
            self.positive.union_with(vars);
        } else {
            self.positive.difference_with(vars);
            self.negative.union_with(vars);
        }
    }

    /// Fix a variable to a specific value, even if it is already fixed at another.
    ///
    /// If this variable was free, this leads to a restriction of the pattern.
    /// If it was fixed to the same value, the pattern is unchanged.
    /// If it was fixed to the opposite value, a conflict is introduced
    pub fn set_ignoring_conflicts(&mut self, uid: usize, value: bool) {
        if value {
            self.positive.insert(uid);
        } else {
            self.negative.insert(uid);
        }
    }

    /// Release a constrained variable
    pub fn release(&mut self, uid: usize) {
        self.positive.remove(uid);
        self.negative.remove(uid);
    }

    /// Release a set of variables
    pub fn release_variables(&mut self, vars: BitSet) {
        self.positive.difference_with(&vars);
        self.negative.difference_with(&vars);
    }

    /// Get the number of fixed variables in this pattern
    /// If a variable is fixed at both values, it will be counted twice
    pub fn len(&self) -> usize {
        self.positive.len() + self.negative.len()
    }

    /// Test if this pattern lacks any restriction.
    /// Equivalent (but faster) than testing is self.len() == 0,
    pub fn is_unrestricted(&self) -> bool {
        self.positive.is_empty() && self.negative.is_empty()
    }

    /// Add all fixed variables from another pattern.
    /// This operation can lead to the introduction of conflicts.
    pub fn add_constraints_from(&mut self, other: &Pattern) {
        self.positive.union_with(&other.positive);
        self.negative.union_with(&other.negative);
    }

    /// Identify variables fixed at both 1 and 0 in this pattern
    fn inner_conflicts(&self) -> BitSet {
        let mut conflicts = self.positive.clone();
        conflicts.intersect_with(&self.negative);
        conflicts
    }

    /// Test if the subspace defined by this pattern contains another pattern.
    ///
    /// i.e. test if all constraints of this pattern are also constraints of the other one.
    ///
    /// The geometric inclusion does not hold if one of the patterns has conflicts.
    pub fn contains(&self, other: &Pattern) -> bool {
        other.positive.is_superset(&self.positive) && other.negative.is_superset(&self.negative)
    }

    /// Evaluate the relation between two patterns
    pub fn relate(&self, p: &Pattern) -> PatternRelation {
        match self.conflicts(p).len() {
            0 => match (self.contains(p), p.contains(self)) {
                (true, true) => PatternRelation::Contains,
                (true, false) => PatternRelation::Contains,
                (false, true) => PatternRelation::Contained,
                (false, false) => PatternRelation::Overlap,
            },
            1 => {
                // Check if the merged pattern contains the original ones
                let mut merged = self.clone();
                merged.merge_with(p);
                match (merged.contains(self), merged.contains(p)) {
                    (true, true) => PatternRelation::JoinBoth(merged),
                    (true, false) => PatternRelation::JoinFirst(merged),
                    (false, true) => PatternRelation::JoinSecond(merged),
                    (false, false) => PatternRelation::JoinOverlap(merged),
                }
            }
            _ => PatternRelation::Disjoint,
        }
    }
}

// The following functions are only used for prime implicant search
impl Pattern {
    pub fn merge_with(&mut self, other: &Pattern) {
        self.add_constraints_from(other);
        let conflicts = self.inner_conflicts();
        if conflicts.len() != 1 {
            panic!("When calling this, there should be exactly one conflicting bit");
        }
        self.release_variables(conflicts);
    }

    pub fn get_common_restrictions(&self, other: &Pattern) -> Pattern {
        let mut result = self.clone();
        result.positive.intersect_with(&other.positive);
        result.negative.intersect_with(&other.negative);
        result
    }

    pub fn conflicts(&self, other: &Pattern) -> Pattern {
        let mut result = self.clone();
        result.positive.intersect_with(&other.negative);
        result.negative.intersect_with(&other.positive);
        result
    }

    pub fn extend_sort_literal(
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
        p.set(idx, val);
        extended.push(p);
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
        let mut result = Pattern::new();
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

impl fmt::Display for Pattern {
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

impl Pattern {
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

#[cfg(test)]
mod tests {
    use crate::func::pattern::Pattern;
    use crate::func::pattern::PatternRelation::{Disjoint, JoinBoth, JoinFirst};

    #[test]
    fn test_patterns() {
        let p = Pattern::from_str("1-0-1--10-");
        let a = Pattern::from_str("--0-1--00-");
        let mpa = Pattern::from_str("1-0-1---0-");

        let b = Pattern::from_str("0-0-11-00-");
        let c = Pattern::from_str("0-1-11-00-");
        let mbc = Pattern::from_str("0---11-00-");

        assert_eq!(a.len(), 4);
        assert_eq!(a.positive().len(), 1);
        assert_eq!(b.len(), 6);
        assert_eq!(c.len(), 6);
        assert_eq!(c.positive().len(), 3);

        assert_eq!(a.contains(&b), true);
        assert_eq!(a.contains(&c), false);

        assert_eq!(b.contains(&a), false);
        assert_eq!(c.contains(&a), false);

        assert_eq!(mbc.contains(&a), false);
        assert_eq!(mbc.contains(&b), true);
        assert_eq!(mbc.contains(&c), true);
        assert_eq!(mbc.contains(&p), false);

        assert_eq!(p.relate(&a), JoinFirst(mpa));
        assert_eq!(b.relate(&c), JoinBoth(mbc));
    }
}
