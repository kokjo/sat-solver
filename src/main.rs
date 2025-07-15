use std::{path::PathBuf, str::FromStr};

use anyhow::Context;
use clap::Parser;
use serde_json::json;

pub mod gate;
use gate::Gate;

pub mod design;
use design::{Design, Cell};

pub mod cnf;
use cnf::{Cnf, MaybeBits};

fn map_aby_cell<T>(cell: &Cell, f: fn(u64, u64, u64) -> T) -> Vec<T> {
    let a = cell.connections.get("A").unwrap();
    let b = cell.connections.get("B").unwrap();
    let y = cell.connections.get("Y").unwrap();
    a.iter().zip(b).zip(y).map(|((&a, &b), &y)| f(a, b, y)).collect()
}

fn map_ay_cell<T>(cell: &Cell, f: fn(u64, u64) -> T) -> Vec<T> {
    let a = cell.connections.get("A").unwrap();
    let y = cell.connections.get("Y").unwrap();
    a.iter().zip(y).map(|(&a, &y)| f(a, y)).collect()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Assignment {
    name: String,
    bits: MaybeBits,
}

impl FromStr for Assignment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, bits) = s.split_once("=").ok_or("missing =")?;
        let bits = bits.parse()?;
        let name = name.to_string();
        Ok(Assignment{ name, bits })
    }
}

#[derive(Debug, Clone, Parser)]
struct Args {
    design: PathBuf,
    module: String,
    assignments: Vec<Assignment>
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let design_file = std::fs::File::open(args.design)?;

    let design: Design = serde_json::from_reader(design_file)?;
    let module = design.modules.get(&args.module).context(format!("Could not find module {:?} in design", args.module))?;

    let mut gates = vec![];
    for (_name, cell) in module.cells.iter() {
        gates.extend(match cell.module.as_str() {
            "$_AND_" => map_aby_cell(cell, Gate::And),
            "$_NAND_" => map_aby_cell(cell, Gate::Nand),
            "$_OR_" => map_aby_cell(cell, Gate::Or),
            "$_NOR_" => map_aby_cell(cell, Gate::Nor),
            "$_NOT_" => map_ay_cell(cell, Gate::Not),
            "$_XOR_" => map_aby_cell(cell, Gate::Xor),
            "$_XNOR_" => map_aby_cell(cell, Gate::Xnor),
            _ => panic!("Unknown Cell: {cell:?}")
        });
    }

    let mut cnf = Cnf::default();
    for gate in gates {
        for clause in gate.to_cnf_clauses() {
            cnf.add_clause(clause);
        }
    }

    for assign in args.assignments {
        let port = module.ports.get(&assign.name).context(format!("Could not find port {:?} in module", assign.name))?;
        for (&bit, &wire) in assign.bits.iter().zip(port.bits.iter()) {
            if let Some(bit) = bit {
                cnf.add_clause([bit.then_some(wire as i64).unwrap_or(-(wire as i64))]);
            }
        }
    }

    println!("Ports:");
    let model = cnf.dpll().unwrap();
    for (name, port) in module.ports.iter() {
        let bits = model.get_bits(port.bits.iter().cloned());
        println!("{name}: {bits}");
    }

    println!("Nets:");
    for (name, net) in module.nets.iter() {
        if net.extra.get("hide_name") == Some(&json!(1)) {
            continue;
        }
        let bits = model.get_bits(net.bits.iter().cloned());
        println!("{name}: {bits}");
    }

    Ok(())
}
