use super::units::{Altitude, Erodibility, Slope, UpliftRate};

/// Attributes of sites.
/// The shape of terrain will be determined by these attributes.
/// ### Attributes
///  - `uplift_rate` is the uplift rate (unit: L/T).
///  - `erodibility` is the erodibility.
///  - `base_altitude` is the base altitude (unit: L). If the base altitude is set to 0, the lowest altitude will be 0.
///  - `max_slope` is the maximum slope (unit: rad). This value must be in the range of [0, π/2).
///     If you don't want to set the maximum slope, set `None`.
pub struct TerrainAttributes {
    pub uplift_rate: UpliftRate,
    pub erodibility: Erodibility,
    pub base_altitude: Altitude,
    pub max_slope: Option<Slope>,
}

impl TerrainAttributes {
    pub fn new(
        base_altitude: Altitude,
        uplift_rate: UpliftRate,
        erodibility: Erodibility,
        max_slope: Option<Slope>,
    ) -> Self {
        Self {
            base_altitude,
            uplift_rate,
            erodibility,
            max_slope,
        }
    }
}
