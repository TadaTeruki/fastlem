use crate::core::units::Altitude;

use super::{interpolator::TerrainInterpolator2D, sites::Site2D};

/// Represents the result of terrain generation includeing the pair of sites and result altitudes.
/// Terrain2D also provides a method for query the interpolated altitudes.
pub struct Terrain2D {
    sites: Vec<Site2D>,
    altitudes: Vec<Altitude>,
    interpolator: TerrainInterpolator2D,
}

impl Terrain2D {
    pub fn new(
        sites: Vec<Site2D>,
        altitudes: Vec<Altitude>,
        interpolator: TerrainInterpolator2D,
    ) -> Self {
        Self {
            sites,
            altitudes,
            interpolator,
        }
    }

    pub fn sites(&self) -> &[Site2D] {
        &self.sites
    }

    pub fn altitudes(&self) -> &[Altitude] {
        &self.altitudes
    }

    /// Get interpolated altitude.
    pub fn get_altitude(&self, site: &Site2D) -> Option<Altitude> {
        self.interpolator.interpolate(&self.altitudes, site)
    }
}
