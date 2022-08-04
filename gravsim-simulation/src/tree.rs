use crate::MassData;
use crate::Simulation;
use nalgebra::Vector2;
use num_enum::TryFromPrimitive;
use smallvec::{smallvec, SmallVec};
use std::collections::VecDeque;

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

#[derive(Clone, Debug)]
pub struct Node {
    pos: Vector2<f32>,
    scale: f32,

    center_of_mass: MassData,
    children: SmallVec<[Option<Box<Node>>; 4]>,
    leaf: bool,
}

impl Node {
    pub fn new_root(pos: Vector2<f32>, scale: f32) -> Self {
        Self {
            pos,
            scale,
            center_of_mass: MassData {
                position: Default::default(),
                mass: 0.0,
            },
            children: smallvec![None; 4],
            leaf: true,
        }
    }

    pub fn new_child(parent: &Self, quadrant: Quadrant, mass_data: MassData) -> Box<Self> {
        Box::new(Self {
            pos: parent.pos + quadrant.offset() * parent.scale * 0.5,
            scale: parent.scale * 0.5,
            center_of_mass: mass_data,
            children: smallvec![None; 4],
            leaf: true,
        })
    }

    pub fn insert(&mut self, obj: &MassData) {
        if self.center_of_mass.mass == 0.0 {
            // if this is the root node, don't subdivide
            self.center_of_mass = *obj;
            return;
        } else if obj.mass == 0.0 {
            return;
        }

        if self.is_leaf() {
            // if this is a leaf, the center of mass is the star that was previously inserted.
            // this star has to be reinserted into the child nodes.
            let offset = self.center_of_mass.position - self.pos;
            let quadrant = Quadrant::from_offset(&offset, self.scale);

            self.insert_into(quadrant, &self.center_of_mass.clone())
        }

        // update center of mass
        self.center_of_mass.position = (self.center_of_mass.position * self.center_of_mass.mass
            + obj.position * obj.mass)
            / (self.center_of_mass.mass + obj.mass);
        self.center_of_mass.mass += obj.mass;

        let offset = obj.position - self.pos;
        let quadrant = Quadrant::from_offset(&offset, self.scale);

        self.insert_into(quadrant, obj);
    }

    fn insert_into(&mut self, quadrant: Quadrant, obj: &MassData) {
        self.leaf = false;
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
        const EPSILON: f32 = 0.05;

        // factor out G and obj.mass
        let mut force_part = Vector2::zeros();

        // bfs
        let mut queue = VecDeque::from([self]);
        while let Some(node) = queue.pop_front() {
            let diff = node.center_of_mass.position - obj.position;
            let dist = (EPSILON + diff.norm_squared()).sqrt();

            let q = node.scale / dist;
            if q < Simulation::THETA || node.is_leaf() {
                force_part += diff / dist.powi(3) * node.center_of_mass.mass;
            } else {
                queue.extend(node.children.iter().flatten().map(|n| n.as_ref()));
            }
        }
        Simulation::GRAVITY * obj.mass * force_part
    }

    pub fn is_leaf(&self) -> bool {
        self.leaf
    }

    pub fn contains(&self, pos: &Vector2<f32>) -> bool {
        self.pos
            .iter()
            .zip(pos.iter())
            .all(|(&a, &b)| b >= a && b < a + self.scale)
    }
}
