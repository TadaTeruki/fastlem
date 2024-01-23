use crate::core::units::Elevation;

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

    pub fn interpolate(&self, elevations: &[Elevation], site: &Site2D) -> Option<Elevation> {
        self.interpolator.interpolate(
            elevations,
            naturalneighbor::Point {
                x: site.x,
                y: site.y,
            },
        )
    }
}
