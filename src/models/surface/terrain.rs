use crate::core::units::Altitude;

use super::{interpolator::TerrainInterpolator2D, sites::Site2D};

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

    pub fn get_altitude(&self, site: &Site2D) -> Option<Altitude> {
        self.interpolator.interpolate(&self.altitudes, site)
    }
}
