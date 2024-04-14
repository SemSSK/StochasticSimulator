use std::{collections::HashMap, fs};

use clap::{arg, Parser};
use itertools::Itertools;
use probability::Probability;
use reaction_registry::{Element, ReactionRegistry};
use simulation_parser::{Ast, InitDeclaration, Parsable};
use value_board::ValueBoard;

mod probability;
mod reaction_registry;
mod value_board;

#[derive(Debug, Default)]
struct IdElementTable {
    table: HashMap<String, (Element, i32)>,
    last_id: u64,
}

impl IdElementTable {
    fn insert_by_name(&mut self, element: String) -> Element {
        self.table.entry(element.clone()).or_insert_with(|| {
            let elem = Element { uuid: self.last_id };
            self.last_id += 1;
            (elem, 0)
        });
        self.table.get(&element).unwrap().0
    }
    fn insert_by_init(&mut self, init: InitDeclaration) -> Element {
        self.table
            .entry(init.identifier.clone())
            .and_modify(|(_, n)| {
                let _ = std::mem::replace(n, init.number as i32);
            })
            .or_insert_with(|| {
                let elem = Element { uuid: self.last_id };
                self.last_id += 1;
                (elem, init.number as i32)
            });
        self.table.get(&init.identifier).unwrap().0
    }
}

#[derive(Debug)]
pub struct Environment {
    pub board: ValueBoard,
    pub registry: ReactionRegistry,
    pub time: f32,
}

impl From<Ast> for Environment {
    fn from(Ast(expressions): Ast) -> Self {
        let mut id_element_table = IdElementTable::default();
        let mut registry = ReactionRegistry::new();

        for expr in expressions {
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
                        (vec![enzhym_solube], p1),
                    );
                    registry.insert(
                        reaction_registry::CollidedElements::Mono(enzhym_solube),
                        (vec![enzhym, solube], p2),
                    );
                    registry.insert(
                        reaction_registry::CollidedElements::Mono(enzhym_solube),
                        (vec![enzhym, result], p3),
                    );
                }
                simulation_parser::Expression::InitDeclaration(init) => {
                    id_element_table.insert_by_init(init);
                }
                _ => (),
            }
        }
        let mut columns = id_element_table
            .table
            .iter()
            .sorted_by_key(|(_, (e, _))| e.uuid)
            .map(|(k, _)| k)
            .cloned()
            .collect_vec();
        columns.push("time".to_string());

        let mut board = ValueBoard {
            rows: vec![],
            columns,
        };
        board.add_entry(
            id_element_table
                .table
                .iter()
                .sorted_by_key(|(_, (e, _))| e.uuid)
                .map(|(_, (_, n))| *n)
                .collect(),
            0.,
        );
        Self {
            board,
            registry,
            time: 0.,
        }
    }
}

impl Environment {
    fn update(&mut self) {
        let current_state = &self.board.rows.last().unwrap().0;
        let (update_vector, tau) = self
            .registry
            .calc_update_vector_and_tau(&self.board.rows.last().unwrap().0);
        let updated_state = current_state
            .iter()
            .zip(update_vector)
            .map(|(x, update)| x + update)
            .collect::<Vec<_>>();
        self.time += tau;
        self.board.add_entry(updated_state, self.time);
    }
    fn get_csv(&self) -> String {
        self.board.convert_to_csv()
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

fn main() {
    let arg = Args::parse();
    let text = fs::read_to_string(arg.source).unwrap();
    let mut environment = Environment::from(
        Ast::parse(text.as_str().into())
            .expect("Parsing Error")
            .content,
    );
    for _ in 0..10_000 {
        environment.update();
    }
    fs::write(arg.output, environment.get_csv()).unwrap();
}
