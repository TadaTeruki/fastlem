use super::units::{Altitude, Erodibility, Slope, UpliftRate};

/// Attributes of sites.
/// The shape of terrain will be determined by these attributes.
/// ### Attributes
///  - `base_altitude` is the base (initial) altitude (unit: L).
///     If you create a terrain from scratch, 0.0 is recommended.
///  - `erodibility` is the erodibility.
///  - `uplift_rate` is the uplift rate (unit: L/T).
///  - `is_outlet` is whether the site is an outlet or not.
///  - `max_slope` is the maximum slope (unit: rad). This value must be in the range of [0, π/2).
///     If you don't want to set the maximum slope, set `None`.
#[derive(Debug)]
pub struct TerrainAttributes {
    pub base_altitude: Altitude,
    pub erodibility: Erodibility,
    pub uplift_rate: UpliftRate,
    pub is_outlet: bool,
    pub max_slope: Option<Slope>,
}

impl Default for TerrainAttributes {
    fn default() -> Self {
        Self {
            base_altitude: rand::random::<f64>() * f64::EPSILON,
            erodibility: 1.0,
            uplift_rate: 1.0,
            is_outlet: false,
            max_slope: None,
        }
    }
}

impl TerrainAttributes {
    pub fn set_base_altitude(self, base_altitude: Altitude) -> Self {
        Self {
            base_altitude,
            ..self
        }
    }

    pub fn set_erodibility(self, erodibility: Erodibility) -> Self {
        Self {
            erodibility,
            ..self
        }
    }

    pub fn set_uplift_rate(self, uplift_rate: UpliftRate) -> Self {
        Self {
            uplift_rate,
            ..self
        }
    }

    pub fn set_is_outlet(self, is_outlet: bool) -> Self {
        Self { is_outlet, ..self }
    }

    pub fn set_max_slope(self, max_slope: Option<Slope>) -> Self {
        Self { max_slope, ..self }
    }
}
