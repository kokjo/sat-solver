pub enum Gate {
    And(u64, u64, u64),
    Nand(u64, u64, u64),
    Or(u64, u64, u64),
    Nor(u64, u64, u64),
    Not(u64, u64),
    Xor(u64, u64, u64),
    Xnor(u64, u64, u64),
}

impl Gate {
    pub fn to_cnf_clauses(&self) -> Vec<Vec<i64>> {
        match *self {
            Gate::And(a, b, c ) => vec![
                vec![-(a as i64), -(b as i64), c as i64],
                vec![a as i64, -(c as i64)],
                vec![b as i64, -(c as i64)],
            ],
            Gate::Nand(a, b, c) => vec![
                vec![-(a as i64), -(b as i64), -(c as i64)],
                vec![a as i64, c as i64],
                vec![b as i64, c as i64],
            ],
            Gate::Or(a, b, c) => vec![
                vec![a as i64, b as i64, -(c as i64)],
                vec![-(a as i64), c as i64],
                vec![-(b as i64), c as i64],
            ],
            Gate::Nor(a, b, c) => vec![
                vec![a as i64, b as i64, c as i64],
                vec![-(a as i64), -(c as i64)],
                vec![-(b as i64), -(c as i64)],
            ],
            Gate::Not(a, c) => vec![
                vec![-(a as i64), -(c as i64)],
                vec![a as i64, c as i64],
            ],
            Gate::Xor(a, b, c) => vec![
                vec![-(a as i64), -(b as i64), -(c as i64)],
                vec![a as i64, b as i64, -(c as i64)],
                vec![a as i64, -(b as i64), c as i64],
                vec![-(a as i64), b as i64, c as i64],
            ],
            Gate::Xnor(a, b, c) => vec![
                vec![-(a as i64), -(b as i64), c as i64],
                vec![a as i64, b as i64, c as i64],
                vec![a as i64, -(b as i64), -(c as i64)],
                vec![-(a as i64), b as i64, -(c as i64)],
            ],
        }
    }
}