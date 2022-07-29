use crate::tree::MassData;
use nalgebra::Vector2;

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
        todo!("color based on mass?")
    }

    pub fn mass(&self) -> f32 {
        self.mass_point.mass
    }

    pub fn pos(&self) -> &Vector2<f32> {
        &self.mass_point.position
    }
}
