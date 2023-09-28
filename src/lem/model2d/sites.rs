use crate::lem::units::{Length, Site};

/// A 2D point in the plane.
#[derive(Clone, Copy, Debug, Default)]
pub struct Site2D {
    pub x: Length,
    pub y: Length,
}

impl Site2D {
    pub fn new(x: Length, y: Length) -> Self {
        Self { x, y }
    }
}

impl Site for Site2D {
    fn distance(&self, other: &Self) -> Length {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    fn squared_distance(&self, other: &Self) -> Length {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
}
