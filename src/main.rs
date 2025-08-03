use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use yosys_json_netlist::{Netlist, Cell, Bit};

pub mod gate;
use gate::Gate;

pub mod cnf;
use cnf::{Cnf, MaybeBits};


pub fn cell_get_conns<'a>(name: &str, cell: &'a Cell, conn: &str) -> Result<&'a Vec<Bit>> {
    cell.connections.get(conn).context(format!("Could not find {conn:?} connection on module {name:?} of type {:?}", cell.module))
}

fn signal(bit: &Bit) -> Result<u64> {
    match *bit {
        Bit::Signal(signal) => Ok(signal),
        otherwise => Err(anyhow!(format!("{:?} is not a signal", otherwise)))
    }
}

fn map_aby_cell<T>(name: &str, cell: &Cell, f: fn(u64, u64, u64) -> T) -> Result<Vec<T>> {
    let a = cell_get_conns(name, cell, "A")?.iter().map(signal).collect::<Result<Vec<u64>>>()?;
    let b = cell_get_conns(name, cell, "B")?.iter().map(signal).collect::<Result<Vec<u64>>>()?;
    let y = cell_get_conns(name, cell, "Y")?.iter().map(signal).collect::<Result<Vec<u64>>>()?;
    Ok(a.into_iter().zip(b).zip(y).map(|((a, b), y)| f(a, b, y)).collect())
}

fn map_ay_cell<T>(name: &str, cell: &Cell, f: fn(u64, u64) -> T) -> Result<Vec<T>> {
    let a = cell_get_conns(name, cell, "A")?.iter().map(signal).collect::<Result<Vec<u64>>>()?;
    let y = cell_get_conns(name, cell, "Y")?.iter().map(signal).collect::<Result<Vec<u64>>>()?;
    Ok(a.into_iter().zip(y).map(|(a, y)| f(a, y)).collect())
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
    #[clap(short='n', long="nets", default_value="false")]
    nets: bool,
    #[clap(short='d', long="dump")]
    dump: Option<PathBuf>,
    netlist: PathBuf,
    module: String,
    assignments: Vec<Assignment>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let netlist_file = std::fs::File::open(args.netlist)?;

    let netlist = Netlist::from_reader(netlist_file)?;
    let module = netlist.modules.get(&args.module).context(format!("Could not find module {:?} in design", args.module))?;

    let mut gates = vec![];
    for (name, cell) in module.cells.iter() {
        gates.extend(match cell.module.as_str() {
            "$_AND_" => map_aby_cell(name, cell, Gate::And)?,
            "$_NAND_" => map_aby_cell(name, cell, Gate::Nand)?,
            "$_OR_" => map_aby_cell(name, cell, Gate::Or)?,
            "$_NOR_" => map_aby_cell(name, cell, Gate::Nor)?,
            "$_NOT_" => map_ay_cell(name, cell, Gate::Not)?,
            "$_XOR_" => map_aby_cell(name, cell, Gate::Xor)?,
            "$_XNOR_" => map_aby_cell(name, cell, Gate::Xnor)?,
            _ => return Err(anyhow::Error::msg(format!("Unknown Cell: {cell:?}")))
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
        for (&bit, port_bit) in assign.bits.iter().zip(port.bits.iter()) {
            let signal = signal(port_bit)?;
            if let Some(bit) = bit {
                cnf.add_clause([bit.then_some(signal as i64).unwrap_or(-(signal as i64))]);
            }
        }
    }

    if let Some(dump) = args.dump {
        let dump = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(dump)?;
        serde_json::to_writer(dump, &cnf)?;
    }

    if let Some(model) = cnf.dpll() {
        if args.nets {
            println!("Nets:");
            for (name, net) in module.nets.iter() {
                if net.hide_name {
                    continue;
                }
                let bits = model.get_bits(net.bits.iter().map(signal).collect::<Result<Vec<u64>>>()?);
                println!("{name}: {bits}");
            }
        }

        println!("Ports:");
        for (name, port) in module.ports.iter() {
            let bits = model.get_bits(port.bits.iter().map(signal).collect::<Result<Vec<u64>>>()?);
            println!("{name}: {bits}");
        }
    } else {
        println!("Unsatisfiable")
    }

    Ok(())
}
