use crate::element::Element;
use crate::moved_molecule::MovedMolecule;
use crate::vector::Vector3d;
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct Molecule {
    pub kind: Element,
    pub position: Vector3d,
}

impl Eq for Molecule {}

impl Molecule {
    pub fn apply_movement(self, direction: Vector3d) -> MovedMolecule {
        let dv = direction * self.kind.speed;
        // Add bound checking in position
        let next_position = self.position + dv;
        if next_position.distance(&Vector3d {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }) > 500.
        {
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
