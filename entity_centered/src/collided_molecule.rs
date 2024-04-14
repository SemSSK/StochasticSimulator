use crate::molecule::Molecule;
use crate::moved_molecule::MovedMolecule;
use crate::reaction_registry::CollidedElements;
use crate::vector::Vector3d;
#[derive(Debug, Hash)]
pub enum CollidedMolecules {
    Mono(MovedMolecule),              // No collision
    Bi(MovedMolecule, MovedMolecule), // Collided with another entity
}

impl CollidedMolecules {
    pub fn get_position(&self) -> Vector3d {
        match self {
            CollidedMolecules::Mono(m) => m.next_position,
            CollidedMolecules::Bi(m1, m2) => 0.5 * (m1.next_position + m2.next_position),
        }
    }
    pub fn get_elements(&self) -> CollidedElements {
        match self {
            CollidedMolecules::Mono(m) => CollidedElements::Mono(m.molecule.kind),
            CollidedMolecules::Bi(m1, m2) => {
                CollidedElements::Bi(m1.molecule.kind, m2.molecule.kind)
            }
        }
    }
    pub fn roll_back(self) -> Vec<Molecule> {
        match self {
            CollidedMolecules::Mono(m) => {
                let mut mol = m.molecule;
                mol.position = m.next_position;
                vec![mol]
            }
            CollidedMolecules::Bi(m1, m2) => {
                vec![m1.molecule, m2.molecule]
            }
        }
    }
}
