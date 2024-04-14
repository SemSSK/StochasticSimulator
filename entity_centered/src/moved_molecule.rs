use crate::collided_molecule::CollidedMolecules;
use crate::molecule::Molecule;
use crate::reaction_registry::ReactionRegistry;
use crate::vector::Vector3d;

// use smallvec::SmallVec;

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct MovedMolecule {
    pub molecule: Molecule,
    pub next_position: Vector3d,
}

impl Eq for MovedMolecule {}

impl MovedMolecule {
    pub fn test_collision(self, other: &Self) -> CollidedMolecules {
        let distance = self.next_position.distance_pow2(&other.next_position);
        let radius_sum = self.molecule.kind.radius + other.molecule.kind.radius;
        if distance > radius_sum {
            CollidedMolecules::Mono(self)
        } else {
            CollidedMolecules::Bi(self, *other)
        }
    }
    pub fn process_collisions<'a, T>(
        self,
        others: T,
        reaction_registry: &ReactionRegistry,
    ) -> (Vec<Molecule>, Option<usize>)
    where
        T: Iterator<Item = (usize, &'a MovedMolecule)>,
    {
        for (i, m) in others {
            match self.test_collision(m) {
                col @ CollidedMolecules::Bi(_, _) => {
                    match reaction_registry.decide_collision(col) {
                        (mols, true) => {
                            return (mols, Some(i));
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }
        (
            reaction_registry
                .decide_collision(CollidedMolecules::Mono(self))
                .0,
            None,
        )
    }
}
