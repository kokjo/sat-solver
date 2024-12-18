use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct CNF(Vec<BTreeSet<i64>>);

impl CNF {
    pub fn add_clause(&mut self, clause: impl IntoIterator<Item = i64>) {
        self.0.push(clause.into_iter().collect());
    }

    pub fn with_clause(mut self, clause: impl IntoIterator<Item = i64>) -> Self {
        self.add_clause(clause);
        self
    }

    pub fn find_pure_literal(&self) -> Option<&i64> {
        self.0
            .iter()
            .flatten()
            .find(|&lit| self.0.iter().flatten().all(|&o| o != -lit))
    }

    pub fn find_unit(&self) -> Option<&i64> {
        self.0
            .iter()
            .find_map(|clause| clause.first().filter(|_| clause.len() == 1))
    }

    pub fn dpll(mut self) -> Option<Vec<i64>> {
        let mut assignments = Vec::new();

        while let Some(&lit) = self.find_unit() {
            self.0.retain_mut(|clause| {
                clause.remove(&-lit);
                !clause.contains(&lit)
            });
            assignments.push(lit);
        }

        while let Some(&lit) = self.find_pure_literal() {
            self.0.retain(|clause| !clause.contains(&lit));
            assignments.push(lit);
        }

        if self.0.iter().any(|clause| clause.is_empty()) {
            return None;
        } else if let Some(&lit) = self.0.first().and_then(|clause| clause.first()) {
            assignments.extend(
                self.clone()
                    .with_clause([lit])
                    .dpll()
                    .or_else(|| self.with_clause([-lit]).dpll())?,
            );
        }

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
    cnf.add_clause([1, 3]);

    dbg!(cnf.dpll());
}
