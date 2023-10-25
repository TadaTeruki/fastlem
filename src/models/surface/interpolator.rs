use naturalneighbor::{Interpolator, Point};

use crate::core::{traits::TerrainInterpolator, units::Altitude};

use super::sites::Site2D;

pub struct TerrainInterpolator2D {
    interpolator: Interpolator,
}

impl TerrainInterpolator2D {
    pub fn new(sites: &[Site2D]) -> Self {
        Self {
            interpolator: Interpolator::new(sites),
        }
    }
}

impl TerrainInterpolator<Site2D> for TerrainInterpolator2D {
    fn interpolate(&self, altitudes: &[Altitude], site: &Site2D) -> Option<Altitude> {
        self.interpolator.interpolate(
            altitudes,
            Point {
                x: site.x,
                y: site.y,
            },
        )
    }
}
