use crate::collided_molecule::CollidedMolecules;
use crate::element::Element;
use crate::molecule::Molecule;
use crate::probability::Probability;
use rustc_hash::FxHashMap;
use std::collections::HashMap;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum CollidedElements {
    Mono(Element),
    Bi(Element, Element),
}

#[derive(Debug)]
pub enum Outcome {
    One(Vec<Element>, Probability),
    Two((Vec<Element>, Probability), (Vec<Element>, Probability)),
}

#[derive(Debug)]
pub struct ReactionRegistry {
    register: FxHashMap<CollidedElements, Outcome>,
}

impl ReactionRegistry {
    pub fn new() -> Self {
        Self {
            register: HashMap::default(),
        }
    }
    pub fn insert(&mut self, k: CollidedElements, v: Outcome) {
        self.register.insert(k, v);
    }
    pub fn get(&self, k: &CollidedElements) -> Option<&Outcome> {
        self.register.get(k)
    }
    pub fn decide_collision(&self, collided_molecules: CollidedMolecules) -> (Vec<Molecule>, bool) {
        let roll = fastrand::f32();
        match self.get(&collided_molecules.get_elements()) {
            Some(Outcome::One(r, p)) if roll < p.get() => {
                // println!("Important collision found and executed");
                (
                    r.into_iter()
                        .map(|e| Molecule {
                            kind: *e,
                            position: collided_molecules.get_position(),
                        })
                        .collect(),
                    true,
                )
            }
            Some(Outcome::Two((r, p), _)) if roll < p.get() => {
                // println!("Important collision found and executed");
                (
                    r.into_iter()
                        .map(|e| Molecule {
                            kind: *e,
                            position: collided_molecules.get_position(),
                        })
                        .collect(),
                    true,
                )
            }
            Some(Outcome::Two((_, p1), (r, p2))) if roll < p1.get() + p2.get() => {
                // println!("Important collision found and executed");
                (
                    r.into_iter()
                        .map(|e| Molecule {
                            kind: *e,
                            position: collided_molecules.get_position(),
                        })
                        .collect(),
                    true,
                )
            }
            _ => (collided_molecules.roll_back(), false),
        }
    }
}