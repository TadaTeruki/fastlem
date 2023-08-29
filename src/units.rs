/// Length (unit: L);
pub type Length = f64;

/// Altitude (unit: L).
pub type Altitude = f64;

/// Uplift rate (unit: L/T).
pub type UpliftRate = f64;

/// Erodibility.
pub type Erodibility = f64;

/// Area (unit: L^2).
pub type Area = f64;

/// Slope (unit: rad).
pub type Slope = f64;

/// Iteration step.
pub type Step = u32;

/// Response Time.
pub type ResponseTime = f64;

/// A 2D point in the plane.
#[derive(Clone, Copy, Debug)]
pub struct Site {
    pub x: Length,
    pub y: Length,
}

impl Site {
    pub fn new(x: Length, y: Length) -> Self {
        Self { x, y }
    }

    /// Calculate the distance between two sites.
    pub fn distance(&self, other: &Site) -> Length {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// Calculate the squared distance between two sites.
    pub fn squared_distance(&self, other: &Site) -> Length {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
}
