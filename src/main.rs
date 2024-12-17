use std::collections::HashSet;

type Literal = i64;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct CNF(Vec<HashSet<Literal>>);

impl CNF {
    pub fn add_clause(&mut self, clause: impl IntoIterator<Item = Literal>) {
        self.0.push(clause.into_iter().collect());
    }

    pub fn with_clause(mut self, clause: impl IntoIterator<Item = Literal>) -> Self {
        self.add_clause(clause);
        self
    }

    pub fn iter_literals(&self) -> impl Iterator<Item = &Literal> {
        self.0.iter().flatten()
    }

    pub fn find_pure_literal(&self) -> Option<Literal> {
        self.iter_literals()
            .filter(|&lit| self.iter_literals().any(|&other| other == -lit))
            .next()
            .copied()
    }

    pub fn find_unit(&self) -> Option<Literal> {
        self.0
            .iter()
            .filter(|clause| clause.len() == 1)
            .flatten()
            .next()
            .copied()
    }

    pub fn choose_literal(&self) -> Option<Literal> {
        self.iter_literals().next().copied()
    }

    pub fn dpll(mut self) -> Option<Vec<Literal>> {
        let mut assignments = Vec::new();

        while let Some(lit) = self.find_unit() {
            self.0.retain_mut(|clause| {
                clause.remove(&-lit);
                !clause.contains(&lit)
            });
            assignments.push(lit);
        }

        while let Some(lit) = self.find_pure_literal() {
            self.0.retain(|clause| !clause.contains(&lit));
            assignments.push(lit);
        }

        if self.0.is_empty() {
            return Some(assignments);
        } else if self.0.iter().any(|clause| clause.is_empty()) {
            return None;
        }

        let lit = self.choose_literal().expect("This cannot happen!");

        assignments.extend(
            self.clone()
                .with_clause([lit])
                .dpll()
                .or_else(|| self.with_clause([-lit]).dpll())?,
        );
        Some(assignments)
    }
}

fn main() {
    let mut cnf = CNF::default();
    cnf.add_clause([1, -1]);
    cnf.add_clause([2, -2]);
    cnf.add_clause([3, -3]);
    cnf.add_clause([-1, -2]);
    cnf.add_clause([-2, -3]);
    cnf.add_clause([2]);

    dbg!(cnf.dpll());
}
