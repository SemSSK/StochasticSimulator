use std::time::{Duration, Instant};

use crate::moved_molecule::MovedMolecule;
use crate::reaction_registry::ReactionRegistry;
use crate::vector::VectorInt3d;
use crate::Environment;
use crate::{molecule::Molecule, vector::Vector3d};
use hashbrown::HashMap;
use indicatif::ProgressBar;
use itertools::Itertools;
use rayon::prelude::*;

fn detect_collision(
    moved_molecules: HashMap<VectorInt3d, Vec<MovedMolecule>>,
    reg: &ReactionRegistry,
    molecules: &mut Vec<Molecule>,
) {
    molecules.extend(
        moved_molecules
            .values()
            .flat_map(|e| {
                let mut ignored = Vec::with_capacity(e.len());
                let res = e
                    .into_iter()
                    .enumerate()
                    .filter_map(move |(i, m1)| {
                        if ignored.contains(&i) {
                            None
                        } else {
                            let (results, ignore) =
                                m1.process_collisions(e.iter().enumerate().skip(i + 1), reg);
                            ignore.iter().for_each(|j| ignored.push(*j));
                            Some(results)
                        }
                    })
                    .collect_vec();
                res
            })
            .flatten(),
    );
}

fn group<K, V, I>(iter: I) -> HashMap<K, Vec<V>>
where
    K: Eq + std::hash::Hash,
    I: Iterator<Item = (K, V)>,
{
    let mut hash_map = HashMap::with_capacity(iter.try_len().unwrap_or(0));

    for (key, value) in iter {
        hash_map
            .entry(key)
            .or_insert_with(|| Vec::with_capacity(5))
            .push(value)
    }

    hash_map
}

fn simulation(reg: &ReactionRegistry, molecules: &mut Vec<Molecule>) {
    let movedmols = group(
        molecules
            .into_iter()
            .map(|m| m.apply_movement(Vector3d::get_random_unitary()))
            .map(|m| (m.next_position.into_vectorint(), m)),
    );
    molecules.clear();

    detect_collision(movedmols, reg, molecules);
}

pub fn run(environment: Environment) -> String {
    let Environment {
        mut board,
        registry,
        mut molecules,
    } = environment;

    let iterations = 9_000_000 * 8;
    let bar = ProgressBar::new(iterations as u64);
    for t in 1..=iterations {
        molecules.reverse();
        simulation(&registry, &mut molecules);
        if t % 500 == 0 {
            board.add_entry(&molecules, t);
        }
        bar.inc(1);
    }
    bar.finish();

    board.convert_to_csv()
}
