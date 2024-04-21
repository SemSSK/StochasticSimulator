use crate::element::Element;
use crate::moved_molecule::MovedMolecule;
use crate::vector::{self, Vector3d};
use std::hash::Hash;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Molecule {
    pub kind: Element,
    pub position: Vector3d,
}

impl Hash for Molecule {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl Eq for Molecule {}

impl Molecule {
    pub fn apply_movement(self, direction: Vector3d) -> MovedMolecule {
        let dv = direction * self.kind.speed;
        let next_position = self.position + dv;
        if next_position.distance(&vector::VECTOR_ZERO) > 500. {
            MovedMolecule {
                next_position: self.position,
                molecule: self,
            }
        } else {
            MovedMolecule {
                next_position,
                molecule: self,
            }
        }
    }
}
