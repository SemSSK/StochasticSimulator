use rand::{distributions::Uniform, Rng};

use crate::probability::Probability;
use std::hash::Hash;

const ALPHA: f32 = 7.4e-7;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Element {
    pub uuid: u64,
}

impl Eq for Element {}

impl Hash for Element {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum CollidedElements {
    Mono(Element),
    Bi(Element, Element),
}

impl CollidedElements {
    fn calculate_consontration(&self, state: &[i32]) -> f32 {
        (match self {
            CollidedElements::Mono(e) => state[e.uuid as usize] as f32,
            CollidedElements::Bi(e1, e2) => {
                ALPHA * (state[e1.uuid as usize] * state[e2.uuid as usize]) as f32
            }
        })
    }
}

#[derive(Debug)]
pub struct ReactionRegistry {
    register: Vec<(CollidedElements, (Vec<Element>, Probability))>,
}

impl ReactionRegistry {
    pub fn new() -> Self {
        Self {
            register: Vec::default(),
        }
    }
    pub fn insert(&mut self, k: CollidedElements, v: (Vec<Element>, Probability)) {
        self.register.push((k, v));
    }
    pub fn get_rate_of_all_reaction(&self, state: &[i32]) -> f32 {
        self.register.iter().fold(0., |r, (collision, (_, p))| {
            collision.calculate_consontration(state) * p.get() + r
        })
    }
    pub fn get_rate_vector(&self, state: &[i32]) -> Vec<f32> {
        self.register
            .iter()
            .map(|(collision, (_, p))| collision.calculate_consontration(state) * p.get())
            .collect()
    }
    pub fn calc_tau_vector(&self, state: &[i32]) -> Vec<f32> {
        self.get_rate_vector(state)
            .iter()
            .map(|&rate| {
                let dist: Uniform<f32> = rand::distributions::Uniform::new(0., 1.);
                if rate != 0. {
                    -rand::thread_rng().sample(dist).ln() / rate
                } else {
                    f32::INFINITY
                }
            })
            .collect()
    }
    pub fn calc_update_vector_and_tau(&self, state: &[i32]) -> (Vec<i32>, f32) {
        let mut v = vec![0; state.len()];
        match self
            .register
            .iter()
            .zip(self.calc_tau_vector(state))
            .min_by(|(_, t1), (_, t2)| t1.total_cmp(t2))
        {
            Some(((collision, outcome), t)) if t.is_finite() => {
                match collision {
                    CollidedElements::Mono(e) => v[e.uuid as usize] = -1,
                    CollidedElements::Bi(e1, e2) => {
                        v[e1.uuid as usize] = -1;
                        v[e2.uuid as usize] = -1;
                    }
                }
                for e in &outcome.0 {
                    v[e.uuid as usize] = 1;
                }
                (v, t)
            }
            _ => (v, 0.),
        }
    }
}
