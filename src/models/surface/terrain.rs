use crate::core::units::Elevation;

use super::{interpolator::TerrainInterpolator2D, sites::Site2D};

/// Represents the result of terrain generation includeing the pair of sites and result Elevations.
/// Terrain2D also provides a method for query the interpolated elevations.
#[derive(Clone)]
pub struct Terrain2D {
    sites: Vec<Site2D>,
    elevations: Vec<Elevation>,
    interpolator: TerrainInterpolator2D,
}

impl Terrain2D {
    pub fn new(
        sites: Vec<Site2D>,
        elevations: Vec<Elevation>,
        interpolator: TerrainInterpolator2D,
    ) -> Self {
        Self {
            sites,
            elevations,
            interpolator,
        }
    }

    pub fn sites(&self) -> &[Site2D] {
        &self.sites
    }

    pub fn elevations(&self) -> &[Elevation] {
        &self.elevations
    }

    /// Get interpolated elevation.
    pub fn get_elevation(&self, site: &Site2D) -> Option<Elevation> {
        self.interpolator.interpolate(&self.elevations, site)
    }
}
