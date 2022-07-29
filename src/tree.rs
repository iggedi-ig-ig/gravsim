use nalgebra::Vector2;
use num_enum::TryFromPrimitive;
use smallvec::{smallvec, SmallVec};

/// represents one quadrant of a node.
/// The corresponding u8 value is the index of the quadrant in the child list.
/// The bits of this value represent its coordinates with the constants
/// `Octant::X` and `Octant::Y` as bitmasks.
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
pub enum Quadrant {
    NorthWest = 0b00,
    NorthEast = 0b01,
    SouthWest = 0b10,
    SouthEast = 0b11,
}

impl Quadrant {
    pub const X: u8 = 0b01;
    pub const Y: u8 = 0b10;

    /// Returns the offset a child in this quadrant has to its parent node.
    /// This value has to be scaled by half of the scale of its parent node.
    pub fn offset(&self) -> Vector2<f32> {
        let bits = *self as u8;
        Vector2::new(
            (bits & Self::X > 0) as u8 as f32,
            (bits & Self::Y > 0) as u8 as f32,
        )
    }

    /// Returns the quadrant a point of the given offset to its
    /// parent node (which has the given scale) will fall into.
    pub fn from_offset(offset: &Vector2<f32>, scale: f32) -> Self {
        let bits_x = (offset.x > 0.5 * scale) as u8 * Self::X;
        let bits_y = (offset.y > 0.5 * scale) as u8 * Self::Y;

        (bits_x | bits_y).try_into().unwrap()
    }
}

/// Represents a mass point in space.
#[derive(Copy, Clone, Debug)]
pub struct MassData {
    pub position: Vector2<f32>,
    pub mass: f32,
}

#[derive(Clone, Debug)]
pub struct Node {
    pos: Vector2<f32>,
    scale: f32,

    center_of_mass: MassData,
    children: SmallVec<[Option<Box<Node>>; 4]>,
    depth: usize,
}

impl Node {
    const THETA: f32 = 1.25;
    const GRAVITY: f32 = 1e-4;

    pub fn new_root(pos: Vector2<f32>, scale: f32) -> Self {
        Self {
            pos,
            scale,
            center_of_mass: MassData {
                position: Default::default(),
                mass: 0.0,
            },
            children: smallvec![None; 4],
            depth: 0,
        }
    }

    pub fn new_child(parent: &Self, quadrant: Quadrant, mass_data: MassData) -> Box<Self> {
        Box::new(Self {
            pos: parent.pos + quadrant.offset() * parent.scale * 0.5,
            scale: parent.scale * 0.5,
            center_of_mass: mass_data,
            children: smallvec![None; 4],
            depth: parent.depth + 1,
        })
    }

    pub fn insert(&mut self, obj: &MassData) {
        if self.center_of_mass.mass == 0.0 {
            // if this is root node, don't subdivide
            self.center_of_mass = *obj;
            return;
        } else if obj.mass == 0.0
            || ((obj.position.x - self.center_of_mass.position.x).abs() < 1e-3
                && (obj.position.y - self.center_of_mass.position.y).abs() < 1e-3)
        {
            return;
        }

        if self.is_leaf() {
            // if this is a leaf, the center of mass is the star that was previously inserted.
            // this star has to be reinserted into the child nodes.
            let offset = self.center_of_mass.position - self.pos;
            let quadrant = Quadrant::from_offset(&offset, self.scale);

            self.children[quadrant as usize] =
                Some(Self::new_child(self, quadrant, self.center_of_mass))
        }

        // update center of mass
        self.center_of_mass.position = (self.center_of_mass.position * self.center_of_mass.mass
            + obj.position * obj.mass)
            / (self.center_of_mass.mass + obj.mass);
        self.center_of_mass.mass += obj.mass;

        let offset = obj.position - self.pos;
        let quadrant = Quadrant::from_offset(&offset, self.scale);

        if let Some(child) = &mut self.children[quadrant as usize] {
            // if there already exists a child in this quadrant,
            // insert into that node to subdivide eventually.
            child.insert(obj);
        } else {
            // if there isn't already a child node of that quadrant, create it / subdivide.
            self.children[quadrant as usize] = Some(Node::new_child(self, quadrant, *obj));
        }
    }

    pub fn force_on(&self, obj: &MassData) -> Vector2<f32> {
        let diff = self.center_of_mass.position - obj.position;
        let dist = (0.125 + diff.norm_squared()).sqrt();

        let q = self.scale / dist;
        if q < Self::THETA || (q.is_normal() && self.is_leaf()) {
            Self::GRAVITY * diff / dist.powi(3) * self.center_of_mass.mass * obj.mass
        } else {
            self.children
                .iter()
                .flatten()
                .map(|c| c.force_on(obj))
                .sum()
        }
    }

    pub fn is_leaf(&self) -> bool {
        !self.children.iter().any(|c| c.is_some())
    }

    pub fn contains(&self, pos: &Vector2<f32>) -> bool {
        self.pos
            .iter()
            .zip(pos.iter())
            .all(|(&a, &b)| b >= a && b < a + self.scale)
    }
}
