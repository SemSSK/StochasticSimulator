#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod collided_molecule;
mod element;
mod molecule;
mod moved_molecule;
mod probability;
mod reaction_registry;
pub mod simulation;
mod value_board;
mod vector;

use clap::Parser;
use element::Element;
use itertools::Itertools;
use molecule::Molecule;
use probability::Probability;
use reaction_registry::ReactionRegistry;
use rustc_hash::FxHashMap;
use simulation::run;
use simulation_parser::{Ast, DiameterDeclaration, Parsable, SpeedDeclaration};
use std::{fs, time::Instant};
use value_board::ValueBoard;
use vector::generate_random_position;

#[derive(Debug, Default)]
struct IdElementTable {
    table: FxHashMap<String, Element>,
    last_id: u64,
}

impl IdElementTable {
    fn insert_by_speed(&mut self, element: SpeedDeclaration) -> Element {
        self.table
            .entry(element.identifier.clone())
            .or_insert_with(|| {
                let elem = Element {
                    uuid: self.last_id,
                    radius: 1.,
                    speed: element.speed,
                };
                self.last_id += 1;
                elem
            })
            .speed = element.speed;
        *self.table.get(&element.identifier).unwrap()
    }
    fn insert_by_diameter(&mut self, element: DiameterDeclaration) -> Element {
        self.table
            .entry(element.identifier.clone())
            .or_insert_with(|| {
                let elem = Element {
                    uuid: self.last_id,
                    radius: element.diameter / 2.0,
                    speed: 1.,
                };
                self.last_id += 1;
                elem
            })
            .radius = element.diameter / 2.0;
        *self.table.get(&element.identifier).unwrap()
    }
    fn insert_by_name(&mut self, element: String) -> Element {
        self.table.entry(element.clone()).or_insert_with(|| {
            let elem = Element {
                uuid: self.last_id,
                radius: 1.,
                speed: 1.,
            };
            self.last_id += 1;
            elem
        });
        *self.table.get(&element).unwrap()
    }
}

#[derive(Debug)]
pub struct Environment {
    pub board: ValueBoard,
    pub registry: ReactionRegistry,
    pub molecules: Vec<Molecule>,
}

impl From<Ast> for Environment {
    fn from(Ast(expressions): Ast) -> Self {
        let mut id_element_table = IdElementTable::default();
        let mut registry = ReactionRegistry::new();
        let mut molecules: Vec<Molecule> = vec![];

        for expr in expressions.into_iter().sorted_by(|a, b| match (a, b) {
            (_, simulation_parser::Expression::InitDeclaration(_)) => std::cmp::Ordering::Greater,
            (simulation_parser::Expression::InitDeclaration(_), _) => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Equal,
        }) {
            match expr {
                simulation_parser::Expression::Reaction(r) => {
                    let enzhym = id_element_table.insert_by_name(r.enzhym.clone());
                    let solube = id_element_table.insert_by_name(r.solubes.clone());
                    let result = id_element_table.insert_by_name(r.results.clone());
                    let amalgam = format!("{}--{}", r.enzhym.clone(), r.solubes.clone());
                    let enzhym_solube = id_element_table.insert_by_name(amalgam.clone());
                    let (p1, p2, p3) = Probability::calc_probability(r.km, r.kcat);
                    registry.insert(
                        reaction_registry::CollidedElements::Bi(enzhym, solube),
                        reaction_registry::Outcome::One(vec![enzhym_solube], p1),
                    );
                    registry.insert(
                        reaction_registry::CollidedElements::Mono(enzhym_solube),
                        reaction_registry::Outcome::Two(
                            (vec![enzhym, solube], p2),
                            (vec![enzhym, result], p3),
                        ),
                    );
                }
                simulation_parser::Expression::SpeedDeclaration(s) => {
                    id_element_table.insert_by_speed(s);
                }
                simulation_parser::Expression::DiameterDeclaration(d) => {
                    id_element_table.insert_by_diameter(d);
                }
                simulation_parser::Expression::InitDeclaration(init) => {
                    let elem = id_element_table.insert_by_name(init.identifier);
                    for _ in 0..init.number {
                        molecules.push(Molecule {
                            kind: elem,
                            position: generate_random_position(),
                        })
                    }
                }
            }
        }

        let board = ValueBoard {
            rows: vec![],
            columns: id_element_table.table.keys().cloned().collect_vec(),
        };
        Self {
            board,
            registry,
            molecules,
        }
    }
}

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Source file of the reaction
    #[arg(short, long, default_value = "reaction.txt")]
    source: String,

    /// Output file of the result of the simulation
    #[arg(short, long, default_value = "results.csv")]
    output: String,
}

fn main() -> anyhow::Result<()> {
    let arg = Args::parse();
    let text = fs::read_to_string(arg.source).unwrap();
    let environment = Environment::from(
        Ast::parse(text.as_str().into())
            .expect("Parsing Error")
            .content,
    );
    let now = Instant::now();
    fs::write(arg.output, run(environment)).unwrap();
    println!(
        "simulation took: {} milis | {} seconds ",
        now.elapsed().as_millis(),
        now.elapsed().as_secs()
    );
    Ok(())
}
