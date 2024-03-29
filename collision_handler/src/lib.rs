use itertools::Itertools;
use rayon::prelude::*;
#[derive(Debug, PartialEq, PartialOrd)]
pub struct CollisionIds(usize, usize);

impl<T> SphereCollidable for &T
where
    T: SphereCollidable,
{
    fn get_position(&self) -> (f32, f32) {
        (*self).get_position()
    }

    fn get_radius(&self) -> f32 {
        (*self).get_radius()
    }
}

pub trait SphereCollidable {
    fn get_position(&self) -> (f32, f32);
    fn get_radius(&self) -> f32;
    fn collide(&self, other: &Self) -> bool {
        let rad_sum = self.get_radius() + other.get_radius();
        let (selfx, selfy) = self.get_position();
        let (otherx, othery) = other.get_position();
        let distance = ((selfx - otherx).powi(2) + (selfy - othery).powi(2)).sqrt();
        if distance < rad_sum {
            true
        } else {
            false
        }
    }
    fn get_collisions(t: &[impl SphereCollidable]) -> Vec<CollisionIds> {
        t.iter()
            .enumerate()
            .flat_map(|(i, o)| {
                t.iter()
                    .enumerate()
                    .skip(i + 1)
                    .filter(|(_, other)| o.collide(other))
                    .map(|(i_other, _)| CollisionIds(i, i_other))
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {

    use std::time::{Duration, Instant};

    use rand::Rng;

    use super::*;

    #[derive(Clone, Copy)]
    struct Ball {
        x: f32,
        y: f32,
        radius: f32,
    }
    impl SphereCollidable for Ball {
        fn get_position(&self) -> (f32, f32) {
            (self.x, self.y)
        }

        fn get_radius(&self) -> f32 {
            self.radius
        }
    }

    fn par_efficient_collisions(targets: &[Ball]) -> Vec<CollisionIds> {
        let grid = targets.into_iter().into_group_map_by(|obj| {
            let (x, y) = obj.get_position();
            let grid_x = x.floor() / 10.;
            let grid_y = y.floor() / 10.;
            (grid_x as i32, grid_y as i32)
        });
        let cols = grid
            .into_par_iter()
            .flat_map(|(_, objs)| Ball::get_collisions(&objs))
            .collect();
        cols
    }

    #[test]
    fn collides() {
        let b1 = Ball {
            x: 0.,
            y: 0.,
            radius: 2.,
        };
        let b2 = Ball {
            x: 2.,
            y: 2.,
            radius: 3.,
        };
        assert!(b1.collide(&b2))
    }

    #[test]
    fn doesnt_collides() {
        let b1 = Ball {
            x: 0.,
            y: 0.,
            radius: 2.,
        };
        let b2 = Ball {
            x: 2.,
            y: 3.,
            radius: 1.,
        };
        assert!(!b1.collide(&b2))
    }

    #[test]
    fn collision_group() {
        let mut balls = vec![
            Ball {
                x: 0.,
                y: 2.,
                radius: 0.7,
            },
            Ball {
                x: 3.,
                y: -2.,
                radius: 1.,
            },
            Ball {
                x: -2.,
                y: 1.,
                radius: 0.5,
            },
            Ball {
                x: 1.,
                y: 1.,
                radius: 0.8,
            },
        ];
        balls.par_sort_unstable_by(|a, b| {
            let (x0, y0) = ((a.x.floor() / 10.) as i32, (a.x.floor() / 10.) as i32);
            let (x1, y1) = ((b.x.floor() / 10.) as i32, (b.x.floor() / 10.) as i32);
            x0.cmp(&x1).then(y0.cmp(&y1))
        });
        let collision_list = balls
            .into_iter()
            .group_by(|obj| {
                let (x, y) = obj.get_position();
                let grid_x = x.floor() / 10.;
                let grid_y = y.floor() / 10.;
                (grid_x as i32, grid_y as i32)
            })
            .into_iter()
            .flat_map(|(_, grp)| Ball::get_collisions(&grp.collect::<Vec<_>>()))
            .collect::<Vec<_>>();
        // let collision_list: Vec<CollisionIds> = par_efficient_collisions(&balls);
        assert_eq!(&Vec::<CollisionIds>::new(), &collision_list);
    }

    #[test]
    fn big_collision_group() {
        let mut rng = rand::thread_rng();
        let mut balls = Vec::new();
        let now = Instant::now();
        for _ in 0..1_00_000 {
            balls.push(Ball {
                x: rng.gen_range(-500.0..500.0),
                y: rng.gen_range(-500.0..500.0),
                radius: rng.gen_range(0.2..1.0),
            });
        }
        println!(
            "Creation of random balls took {}ms",
            Instant::now().duration_since(now).as_millis()
        );
        let now = Instant::now();
        let collisions = par_efficient_collisions(&balls);
        println!(
            "hashmap took {}ms for just one round calculated {} collisions",
            Instant::now().duration_since(now).as_millis(),
            collisions.len()
        );
        let now = Instant::now();
        balls.par_sort_unstable_by(|a, b| {
            let (x0, y0) = ((a.x / 10.).floor() as i32, (a.y / 10.).floor() as i32);
            let i0 = x0 + y0 * 1000;
            let (x1, y1) = ((b.x / 10.).floor() as i32, (b.y / 10.).floor() as i32);
            let i1 = x1 + y1 * 1000;
            i0.cmp(&i1)
        });
        let collisions = balls
            .iter()
            .group_by(|b| {
                let grid_x = b.x.floor() / 10.;
                let grid_y = b.y.floor() / 10.;
                (grid_x as i32, grid_y as i32)
            })
            .into_iter()
            .flat_map(|(_, grp)| Ball::get_collisions(&grp.collect::<Vec<_>>()))
            .collect::<Vec<_>>();
        println!(
            "sort and group took {}ms for just one round calculated {} collisions",
            Instant::now().duration_since(now).as_millis(),
            collisions.len()
        );
        let now = Instant::now();
        let collisions = Ball::get_collisions(&balls);
        println!(
            "naive took {}ms for just one round calculated {} collisions",
            Instant::now().duration_since(now).as_millis(),
            collisions.len()
        );
    }
}
