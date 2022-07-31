use crate::tree::Node;
use nalgebra::Vector2;
use palette::rgb::Rgb;
use rayon::prelude::*;

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

    pub fn color(&self) -> [f32; 3] {
        let rgb = ColorMap::STARS.get(self.mass() / 100.0);
        [rgb.red, rgb.green, rgb.blue]
    }

    pub fn mass(&self) -> f32 {
        self.mass_point.mass
    }

    pub fn pos(&self) -> &Vector2<f32> {
        &self.mass_point.position
    }
}

/// Represents a mass point in space.
#[derive(Copy, Clone, Debug)]
pub struct MassData {
    pub position: Vector2<f32>,
    pub mass: f32,
}

pub struct Simulation {
    pub stars: Vec<Star>,
}

impl Simulation {
    pub const THETA: f32 = 1.2;
    pub const GRAVITY: f32 = 6.67e-11;
    pub const TIME_STEP: f32 = 3600.0 * 24.0 * 7.0;

    pub fn new<I>(stars: I) -> Self
    where
        I: IntoIterator<Item = Star>,
    {
        Self {
            stars: stars.into_iter().collect(),
        }
    }

    pub fn update(&mut self) {
        const SCALE: f32 = 1500.0;

        let mut tree = Node::new_root(-Vector2::repeat(SCALE / 2.0), SCALE);

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
            });

        // integration step
        self.stars
            .iter_mut()
            .for_each(|star| star.mass_point.position += star.vel * Self::TIME_STEP);
    }
}

struct ColorMap<'a> {
    colors: &'a [(f32, f32, f32)],
}

impl<'a> ColorMap<'a> {
    pub const STARS: ColorMap<'static> = ColorMap {
        colors: &[
            (255.0, 181.0 / 255.0, 108.0 / 255.0),
            (255.0, 218.0 / 255.0, 181.0 / 255.0),
            (255.0, 237.0 / 255.0, 227.0 / 255.0),
            (249.0 / 255.0, 245.0 / 255.0, 255.0),
            (213.0 / 255.0, 224.0 / 255.0, 255.0),
            (162.0 / 255.0, 192.0 / 255.0, 255.0),
            (146.0 / 255.0, 181.0 / 255.0, 255.0),
        ],
    };

    pub fn get(&self, t: f32) -> Rgb {
        // this is the index of the "lower" color.
        let i = ((self.colors.len() - 1) as f32 * t).floor() as usize;

        Rgb::new(
            (1.0 - t) * self.colors[i].0 + t * self.colors[i + 1].0 * t,
            (1.0 - t) * self.colors[i].1 + t * self.colors[i + 1].1 * t,
            (1.0 - t) * self.colors[i].2 + t * self.colors[i + 1].2 * t,
        )
    }
}
