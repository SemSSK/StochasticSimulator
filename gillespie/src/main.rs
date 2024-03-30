use itertools::Itertools;
use parser::{Ast, Parsable};
use rand::prelude::*;
use std::{
    collections::HashMap,
    ops::{AddAssign, SubAssign},
};

// Constants
const ALPHA: f32 = 0.00000074;
const V: f32 = 1.;

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
struct Probability(f32);
impl Probability {
    fn new(x: f32) -> Option<Self> {
        if x >= 0. && x <= 1. {
            Some(Probability(x))
        } else {
            None
        }
    }
    // Needs better error result with kcat and km
    // Should be on parser directly
    fn calc_probability(km: f32, kcat: f32) -> (Self, Self, Self) {
        let p3 = kcat / 10000.;
        let p2 = p3 / 10.;
        let p1 = if kcat >= 300. && km <= 80. {
            1.
        } else {
            (p2 + p3) / (0.448 * (1. + (p2 + p3).powi(2)) * km)
        };
        (Probability(p1), Probability(p2), Probability(p3))
    }
}

#[derive(Debug)]
struct Reaction {
    solubes: Vec<Chemical>,
    results: Vec<Chemical>,
    probability: Probability,
}

impl Reaction {
    fn calc_tau(&self, state: &ElementsState) -> f32 {
        if self.solubes.len() > 1 {
            (ALPHA / V)
                * (self
                    .solubes
                    .iter()
                    .fold(1, |prod, solube| prod * state.get(solube).unwrap())
                    as f32)
                * self.probability.0
        } else {
            (self
                .solubes
                .iter()
                .fold(1, |prod, solube| prod * state.get(solube).unwrap()) as f32)
                * self.probability.0
        }
    }
}

#[derive(Debug)]
struct ElementsState(HashMap<Chemical, u32>);

impl ElementsState {
    fn get(&self, chem: &Chemical) -> Option<u32> {
        self.0.get(chem).cloned()
    }
    fn add_assign(&mut self, chem: &Chemical, value: u32) {
        match self.0.get_mut(chem) {
            Some(n) => n.add_assign(value),
            None => (),
        }
    }
    fn sub_assign(&mut self, chem: &Chemical, value: u32) {
        match self.0.get_mut(chem) {
            Some(n) => n.sub_assign(value),
            None => (),
        }
    }
}

#[derive(Debug)]
struct Environment {
    reactions: Vec<Reaction>,
    elements: ElementsState,
    time: f32,
}

impl Environment {
    fn update_exact(&mut self, rng: &mut ThreadRng) {
        let Some((tau, reaction)) = self
            .reactions
            .iter()
            .map(|reaction| (reaction.calc_tau(&self.elements), reaction))
            .filter(|(a, _)| *a > 0.)
            .map(|(an, reaction)| (-rng.gen_range(0.0f32..=1.0f32).log10() / an, reaction))
            .min_by(|x, y| x.0.total_cmp(&y.0))
        else {
            return ();
        };
        for solube in &reaction.solubes {
            self.elements.sub_assign(&solube, 1);
        }
        for result in &reaction.results {
            self.elements.add_assign(&result, 1);
        }
        self.time += tau;
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
                parser::Expression::Reaction(reaction) => {
                    let (p1, p2, p3) = Probability::calc_probability(reaction.km, reaction.kcat);
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
                parser::Expression::InitDeclaration(initialization) => {
                    element_states
                        .insert(Chemical(initialization.identifier), initialization.number);
                }
                _ => (),
            }
        }
        Environment {
            reactions,
            elements: ElementsState(element_states),
            time: 0.,
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
    for i in 0..100_000 {
        env.update_exact(&mut rng);
        if i % 100 == 0 {
            csv.push_str(&env.get_csv_line());
        }
    }
    std::fs::write("results.csv", csv).unwrap();
}
