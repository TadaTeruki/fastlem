use naturalneighbor::Lerpable;

use super::units::{Elevation, Erodibility, Slope, UpliftRate};

/// The topographical parameters of sites.
/// The shape of the terrain will be determined by these parameters.
///
/// ### Properties
///  - `base_elevation` is the initial elevation (unit: L).
///     The default value is 0.0 which is recommended if you create a terrain from scratch.
///
///  - `erodibility` is the erodibility.
///     This is the main parameter to determine the shape of the terrain.
///
///  - `uplift_rate` is the uplift rate (unit: L/T).
///     The default value is 1.0. Configuring this value is not recommended.
///
///  - `is_outlet` is whether the site is an outlet or not.
///     The elevation will be always set 0.0 if the site is outlet.
///
///  - `max_slope` is the maximum slope (unit: rad). This value must be in the range of [0, π/2).
///     You can set `None` if you don't want to set the maximum slope.
#[derive(Debug, Clone)]
pub struct TopographicalParameters {
    pub(crate) base_elevation: Elevation,
    pub(crate) erodibility: Erodibility,
    pub(crate) uplift_rate: UpliftRate,
    pub(crate) is_outlet: bool,
    pub(crate) max_slope: Option<Slope>,
}

impl Default for TopographicalParameters {
    fn default() -> Self {
        Self {
            base_elevation: 0.0,
            erodibility: 1.0,
            uplift_rate: 1.0,
            is_outlet: false,
            max_slope: None,
        }
    }
}

impl TopographicalParameters {
    pub fn set_base_elevation(self, base_elevation: Elevation) -> Self {
        Self {
            base_elevation,
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

impl Lerpable for TopographicalParameters {
    fn lerp(&self, other: &Self, prop: f64) -> Self {
        let base_elevation = self.base_elevation * (1.0 - prop) + other.base_elevation * prop;
        let uplift_rate = self.uplift_rate * (1.0 - prop) + other.uplift_rate * prop;
        let erodibility = self.erodibility * (1.0 - prop) + other.erodibility * prop;
        let is_outlet = self.is_outlet || other.is_outlet;
        let max_slope = if let (Some(self_max_slope), Some(other_max_slope)) =
            (self.max_slope, other.max_slope)
        {
            Some(self_max_slope * (1.0 - prop) + other_max_slope * prop)
        } else if prop < 0.5 {
            self.max_slope
        } else {
            other.max_slope
        };
        TopographicalParameters {
            base_elevation,
            uplift_rate,
            erodibility,
            is_outlet,
            max_slope,
        }
    }
}
