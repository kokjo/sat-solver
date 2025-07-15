use std::{collections::BTreeSet, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MaybeBits(Vec<Option<bool>>);

impl std::ops::Deref for MaybeBits {
    type Target = Vec<Option<bool>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MaybeBits {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromStr for MaybeBits {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.chars().map(|bit| match bit {
            '0' => Ok(Some(false)),
            '1' => Ok(Some(true)),
            'x' => Ok(None),
            _ => Err("bits must be '1', '0', 'x'"),
        }).collect::<Result<Vec<_>, _>>()?))
    }
}

impl std::fmt::Display for MaybeBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.iter().map(|mb_bit| match mb_bit {
            Some(true) => '1',
            Some(false) => '0',
            None => 'x',
        }).collect::<String>())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Model(BTreeSet<i64>);

impl Model {
    pub fn get_bit(&self, bit: u64) -> Option<bool> {
        if self.contains(&(bit as i64)) {
            Some(true)
        } else if self.contains(&-(bit as i64)) {
            Some(false)
        } else {
            None
        }
    }

    pub fn get_bits(&self, bits: impl IntoIterator<Item = u64>) -> MaybeBits {
        MaybeBits(bits.into_iter().map(|bit| self.get_bit(bit)).collect())
    }
}

impl std::ops::Deref for Model {
    type Target = BTreeSet<i64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Model {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cnf(Vec<BTreeSet<i64>>);

impl Cnf {
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

    pub fn dpll(mut self) -> Option<Model> {
        let mut assignments = Model::default();

        while let Some(&lit) = self.find_unit().or_else(|| self.find_pure_literal()) {
            self.0.retain_mut(|clause| {
                clause.remove(&-lit);
                !clause.contains(&lit)
            });
            assignments.insert(lit);
        }

        if self.0.iter().any(BTreeSet::is_empty) {
            return None;
        } else if let Some(&lit) = self.0.first().and_then(BTreeSet::first) {
            let sol = |lit| self.clone().with_clause([lit]).dpll().map(|m| m.0);
            // assignments.extend(sol(lit).or_else(|| sol(-lit))?);
            assignments.extend(sol(lit).or_else(|| sol(-lit))?);
        }

        Some(assignments)
    }
}