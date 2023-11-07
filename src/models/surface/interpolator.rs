use crate::core::units::Altitude;

use super::sites::Site2D;

pub struct TerrainInterpolator2D {
    interpolator: naturalneighbor::Interpolator,
}

impl TerrainInterpolator2D {
    pub fn new(sites: &[Site2D]) -> Self {
        Self {
            interpolator: naturalneighbor::Interpolator::new(sites),
        }
    }

    pub fn interpolate(&self, altitudes: &[Altitude], site: &Site2D) -> Option<Altitude> {
        self.interpolator.interpolate(
            altitudes,
            naturalneighbor::Point {
                x: site.x,
                y: site.y,
            },
        )
    }
}
