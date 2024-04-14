use itertools::Itertools;
use rand::prelude::*;
use simulation_parser::{Ast, Parsable};
use std::{
    collections::HashMap,
    ops::{AddAssign, SubAssign},
};

// Constants
const ALPHA: f64 = 7.4e-7;
const V: f64 = 1.;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct Chemical(String);
impl Chemical {
    fn get(&self) -> &str {
        &self.0
    }
}

impl From<String> for Chemical {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
struct Probability(f64);
impl Probability {
    fn new(x: f64) -> Option<Self> {
        if x >= 0. && x <= 1. {
            Some(Probability(x))
        } else {
            None
        }
    }
    // Needs better error result with kcat and km
    // Should be on parser directly
    fn calc_probability(km: f64, kcat: f64) -> (Self, Self, Self) {
        let p3 = kcat / 10000.;
        let p2 = p3 / 10.;
        let p1 = if kcat >= 300. && km <= 80. {
            1.
        } else {
            (p2 + p3) / (0.448 * (1. + (p2 + p3).powi(2)) * km)
        };
        (Probability(p1), Probability(p2), Probability(p3))
    }

    fn get(&self) -> f64 {
        self.0
    }
}

fn random_round(rng: &mut ThreadRng, x: f64) -> u32 {
    let m = x.fract();
    let random = rng.gen_range(0.0..=1.0);
    if m > random {
        x.ceil() as u32
    } else {
        x.floor() as u32
    }
}

#[derive(Debug)]
struct Reaction {
    solubes: Vec<Chemical>,
    results: Vec<Chemical>,
    probability: Probability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Operation {
    Incr(u32),
    Decr(u32),
}

impl Reaction {
    fn get_mutations(&self, state: &mut ElementsState, rng: &mut ThreadRng) {
        let c = if self.solubes.len() > 1 {
            ALPHA / V
                * (state.get(&self.solubes[0]).unwrap() * state.get(&self.solubes[0]).unwrap())
                    as f64
        } else {
            state.get(&self.solubes[0]).unwrap() as f64
        };
        if c == 0. {
            return ();
        }
        let n = c * self.probability.get();
        let n = random_round(rng, n);
        for chem in &self.solubes {
            state.apply_operation(chem, Operation::Decr(n));
        }
        for chem in &self.results {
            state.apply_operation(chem, Operation::Incr(n));
        }
    }
}

#[derive(Debug)]
struct ElementsState(HashMap<Chemical, u32>);

impl ElementsState {
    fn get(&self, chem: &Chemical) -> Option<u32> {
        self.0.get(chem).cloned()
    }
    fn apply_operation(&mut self, chem: &Chemical, value: Operation) {
        match self.0.get_mut(chem) {
            Some(n) => match value {
                Operation::Incr(v) => n.add_assign(v),
                Operation::Decr(v) => n.sub_assign(v),
            },
            None => (),
        }
    }
}

#[derive(Debug)]
struct Environment {
    reactions: Vec<Reaction>,
    elements: ElementsState,
    time: u32,
}

impl Environment {
    fn update_exact(&mut self, rng: &mut ThreadRng) {
        self.reactions.shuffle(rng);
        self.reactions
            .iter_mut()
            .for_each(|reaction| reaction.get_mutations(&mut self.elements, rng));
        self.time += 100;
    }

    fn get_csv_heading(&self) -> String {
        std::format!(
            "time,{}\n",
            self.elements
                .0
                .keys()
                .map(|key| key.0.to_owned())
                .intersperse(",".to_string())
                .collect::<String>()
        )
    }
    fn get_csv_line(&self) -> String {
        std::format!(
            "{},{}\n",
            self.time,
            self.elements
                .0
                .keys()
                .into_iter()
                .map(|k| self.elements.0[&k].to_string())
                .intersperse(",".to_string())
                .collect::<String>()
        )
    }
}

impl From<Ast> for Environment {
    fn from(ast: Ast) -> Self {
        ast.to_environment()
    }
}

trait ToEnvironment {
    fn to_environment(self) -> Environment;
}

impl ToEnvironment for Ast {
    fn to_environment(self) -> Environment {
        let Ast(exprs) = self;
        let mut element_states = HashMap::new();
        let mut reactions = vec![];
        for expr in exprs {
            match expr {
                simulation_parser::Expression::Reaction(reaction) => {
                    let (p1, p2, p3) =
                        Probability::calc_probability(reaction.km as f64, reaction.kcat as f64);
                    let reaction1 = Reaction {
                        probability: p1,
                        solubes: [reaction.enzhym.clone(), reaction.solubes.clone()]
                            .into_iter()
                            .map(Chemical)
                            .collect(),
                        results: vec![format!(
                            "{}---{}",
                            reaction.enzhym.clone(),
                            reaction.solubes.clone()
                        )]
                        .into_iter()
                        .map(Chemical)
                        .collect(),
                    };
                    let reaction2 = Reaction {
                        probability: p2,
                        solubes: [format!(
                            "{}---{}",
                            reaction.enzhym.clone(),
                            reaction.solubes.clone()
                        )]
                        .into_iter()
                        .map(Chemical)
                        .collect(),
                        results: vec![reaction.enzhym.clone(), reaction.solubes.clone()]
                            .into_iter()
                            .map(Chemical)
                            .collect(),
                    };
                    element_states.insert(
                        Chemical(format!(
                            "{}---{}",
                            reaction.enzhym.clone(),
                            reaction.solubes.clone()
                        )),
                        0,
                    );
                    element_states.insert(Chemical(reaction.enzhym.clone()), 0);
                    element_states.insert(Chemical(reaction.solubes.clone()), 0);
                    element_states.insert(Chemical(reaction.results.clone()), 0);
                    let reaction3 = Reaction {
                        probability: p3,
                        solubes: [format!(
                            "{}---{}",
                            reaction.enzhym.clone(),
                            reaction.solubes.clone()
                        )]
                        .into_iter()
                        .map(Chemical)
                        .collect(),
                        results: vec![reaction.enzhym, reaction.results]
                            .into_iter()
                            .map(Chemical)
                            .collect(),
                    };
                    reactions.push(reaction1);
                    reactions.push(reaction2);
                    reactions.push(reaction3);
                }
                simulation_parser::Expression::InitDeclaration(initialization) => {
                    element_states
                        .insert(Chemical(initialization.identifier), initialization.number);
                }
                _ => (),
            }
        }
        Environment {
            reactions,
            elements: ElementsState(element_states),
            time: 0,
        }
    }
}

fn main() {
    let file = include_str!("../../test_input.txt");
    let ast = Ast::parse(file.into()).unwrap().content;
    println!("Out Ast is :");
    println!("{:?}", ast);
    println!("Our Environment is:");
    let mut env = ast.to_environment();
    println!("{:?}", env);
    let mut rng = rand::thread_rng();
    let mut csv = env.get_csv_heading();
    while env.time < 60_000_000 {
        env.update_exact(&mut rng);
        if env.time % 100 == 0 {
            csv.push_str(&env.get_csv_line());
        }
    }
    std::fs::write("results.csv", csv).unwrap();

    // let n = 0.00000018;
    // let mut rng = thread_rng();
    // let count = (0..1_000_000_000).into_iter().fold(0, |count, _| {
    //     if rng.gen_range(0.0..=1.0) < n {
    //         count + 1
    //     } else {
    //         count
    //     }
    // });
    // println!("number of hits = {}", count)
}
