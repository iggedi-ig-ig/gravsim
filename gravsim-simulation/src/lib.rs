use crate::tree::Node;
use nalgebra::Vector2;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub mod tree;

#[derive(Copy, Clone, Debug)]
pub struct Star {
    pub mass_point: MassData,
    pub vel: Vector2<f32>,
}

impl Star {
    pub const DENSITY: f32 = 250.0;

    pub fn new(pos: Vector2<f32>, vel: Vector2<f32>, mass: f32) -> Self {
        Self {
            mass_point: MassData {
                position: pos,
                mass,
            },
            vel,
        }
    }

    pub fn radius(&self) -> f32 {
        (0.75 * self.mass_point.mass / Self::DENSITY).cbrt()
    }

    pub fn color(&self, _mass_dist: &MassDistribution) -> [f32; 3] {
        [1.0; 3]
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
    pub mass_dist: MassDistribution,
}

impl Simulation {
    pub const SCALE: f32 = 1500.0;
    pub const THETA: f32 = 1.1;
    pub const GRAVITY: f32 = 1e-4;

    pub fn new<I>(stars: I, mass_dist: MassDistribution) -> Self
    where
        I: IntoIterator<Item = Star>,
    {
        Self {
            stars: stars.into_iter().collect(),
            mass_dist,
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
            .iter_mut()
            .filter(|star| tree.contains(star.pos()))
            .for_each(|star| {
                let force = tree.force_on(&star.mass_point);
                star.vel += force / star.mass();
            });

        // integration step
        self.stars
            .iter_mut()
            .for_each(|star| star.mass_point.position += star.vel);
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
