use crate::tree::Node;
use nalgebra::{Vector2, Vector3};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub mod tree;

#[derive(Copy, Clone, Debug)]
pub struct Star {
    pub mass_point: MassData,
    pub vel: Vector2<f32>,
    pub color: [f32; 3],
}

impl Star {
    pub const DENSITY: f32 = 250.0;

    pub fn new(pos: Vector2<f32>, vel: Vector2<f32>, color: [f32; 3], mass: f32) -> Self {
        Self {
            mass_point: MassData {
                position: pos,
                mass,
            },
            vel,
            color,
        }
    }

    pub fn radius(&self) -> f32 {
        (0.75 * self.mass_point.mass / (Self::DENSITY * std::f32::consts::PI)).cbrt()
    }

    pub fn color(&self) -> [f32; 3] {
        self.color
    }

    pub fn mass(&self) -> f32 {
        self.mass_point.mass
    }

    pub fn pos(&self) -> &Vector2<f32> {
        &self.mass_point.position
    }
}

/// Represents a mass point in space.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct MassData {
    pub position: Vector2<f32>,
    pub mass: f32,
}

pub struct Simulation {
    pub stars: Vec<Star>,
}

impl Simulation {
    pub const SCALE: f32 = 5000.0;
    pub const THETA: f32 = 1.25;
    pub const GRAVITY: f32 = 1e-4;

    pub fn new<I>(stars: I) -> Self
    where
        I: IntoIterator<Item = Star>,
    {
        Self {
            stars: stars.into_iter().collect(),
        }
    }

    pub fn update(&mut self) {
        let mut tree = Node::new_root(-Vector2::repeat(Self::SCALE / 2.0), Self::SCALE);

        // insert stars into tree
        for star in &self.stars {
            if tree.contains(star.pos()) {
                tree.insert(&star.mass_point);
            }
        }

        // calculate force on stars
        self.stars
            .par_iter_mut()
            .filter(|star| tree.contains(star.pos()))
            .for_each(|star| {
                let force = tree.force_on(&star.mass_point);
                star.vel += force / star.mass();

                // integration can be done here because tree doesn't change
                star.mass_point.position += star.vel;
            });

        self.stars
            .iter_mut()
            .filter(|star| !tree.contains(star.pos()))
            .for_each(|star| star.mass_point.position = Vector2::from_element(f32::NAN))
    }
}

pub struct Galaxy {
    /// `stars[0]` is the center
    stars: Vec<Star>,
}

impl Galaxy {
    pub fn new(
        center: Star,
        num_stars: usize,
        radius: f32,
        mass_distribution: &MassDistribution,
        color: [f32; 3],
    ) -> Self {
        let mut rng = XorShiftRng::from_entropy();

        Self {
            stars: [center]
                .into_iter()
                .chain((0..num_stars).map(|_| {
                    let a = rng.gen::<f32>() * std::f32::consts::TAU;
                    let d = rng.gen::<f32>().sqrt() * radius;

                    let relative_pos = Vector2::new(a.sin(), a.cos()) * d;
                    let n = Vector3::cross(
                        &*Vector3::z_axis(),
                        &Vector3::new(relative_pos.x, relative_pos.y, 0.0),
                    );
                    let velocity = (Simulation::GRAVITY * center.mass() / d).sqrt();

                    Star::new(
                        center.pos() + relative_pos,
                        center.vel + n.xy().normalize() * velocity,
                        color,
                        1.0 + mass_distribution.sample(rng.gen()),
                    )
                }))
                .collect(),
        }
    }

    pub fn stars(&self) -> &Vec<Star> {
        &self.stars
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MassDistribution {
    alpha: f32,
    max_mass: f32,
}

impl MassDistribution {
    pub fn new(alpha: f32, max_mass: f32) -> Self {
        Self { alpha, max_mass }
    }
}

impl MassDistribution {
    pub fn sample(&self, t: f32) -> f32 {
        self.max_mass * ((self.alpha * t).exp_m1() / self.alpha.exp_m1()).min(1.0)
    }

    pub fn eval_inv(&self, x: f32) -> f32 {
        (self.alpha.exp_m1() * x / self.max_mass + 1.0).ln() / self.alpha
    }
}
