use std::{error, io};

use delaunator::Point;
use voronoice::{BoundingBox, VoronoiBuilder};

use crate::units::{Area, Site};

/// A set of sites.
#[derive(Default)]
pub struct TerrainModel {
    sites: Option<Vec<Site>>,
    bound_min: Option<Site>,
    bound_max: Option<Site>,
}

impl TerrainModel {
    /// Set sites.
    pub fn set_sites(mut self, sites: Vec<Site>) -> Self {
        self.sites = Some(sites);
        self
    }

    /// Set the bounding rectangle of the sites.
    ///
    /// If `bound_min` and `bound_max` is `None`, the bounding rectangle will be
    /// computed from the sites.
    pub fn set_bounding_box(
        mut self,
        bound_min: Option<Site>,
        bound_max: Option<Site>,
    ) -> Result<Self, Box<impl error::Error>> {
        let sites = {
            if let Some(sites) = &mut self.sites {
                sites
            } else {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "You must set sites using `set_sites` before iterating",
                )));
            }
        };

        let b_min = {
            if let Some(b_min) = bound_min {
                b_min
            } else {
                sites.iter().fold(
                    Site {
                        x: std::f64::MAX,
                        y: std::f64::MAX,
                    },
                    |acc, s| Site {
                        x: acc.x.min(s.x),
                        y: acc.y.min(s.y),
                    },
                )
            }
        };
        let b_max = {
            if let Some(b_max) = bound_max {
                b_max
            } else {
                sites.iter().fold(
                    Site {
                        x: std::f64::MIN,
                        y: std::f64::MIN,
                    },
                    |acc, s| Site {
                        x: acc.x.max(s.x),
                        y: acc.y.max(s.y),
                    },
                )
            }
        };
        self.bound_max = Some(b_max);
        self.bound_min = Some(b_min);
        Ok(self)
    }

    /// Relocate the sites to apploximately evenly spaced positions using Lloyd's algorithm.
    /// The number of iterations for Lloyd's algorithm is specified by `iterations`.
    pub fn iterate_sites(mut self, iterations: usize) -> Result<Self, Box<impl error::Error>> {
        if iterations == 0 {
            return Ok(self);
        }

        let sites = {
            if let Some(sites) = &mut self.sites {
                sites
            } else {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "You must set sites using `set_sites` before iterating",
                )));
            }
        };

        let (b_min, b_max) = {
            if let (Some(b_min), Some(b_max)) = (&self.bound_min, &self.bound_max) {
                (b_min, b_max)
            } else {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "You must set the bounding box using `set_bounding_box` before iterating",
                )));
            }
        };

        let voronoi_opt = VoronoiBuilder::default()
            .set_sites(
                sites
                    .iter()
                    .map(|s| voronoice::Point { x: s.x, y: s.y })
                    .collect(),
            )
            .set_bounding_box(BoundingBox::new(
                voronoice::Point {
                    x: (b_max.x - b_min.x) / 2.0,
                    y: (b_max.y - b_min.y) / 2.0,
                },
                b_max.x - b_min.x,
                b_max.y - b_min.y,
            ))
            .set_lloyd_relaxation_iterations(iterations)
            .build();

        if let Some(voronoi) = voronoi_opt {
            self.sites = Some(
                voronoi
                    .sites()
                    .iter()
                    .map(|s| Site { x: s.x, y: s.y })
                    .collect::<Vec<Site>>(),
            );
        }

        Ok(self)
    }

    /// Calculate the area of each site.
    /// The area is calculated using the Voronoi diagram.
    pub fn calculate_areas(&self) -> Result<Vec<Area>, Box<impl error::Error>> {
        let sites = {
            if let Some(sites) = &self.sites {
                sites
            } else {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "You must set sites using `set_sites` before calculating area",
                )));
            }
        };

        let (b_min, b_max) = {
            if let (Some(b_min), Some(b_max)) = (&self.bound_min, &self.bound_max) {
                (b_min, b_max)
            } else {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "You must set the bounding box using `set_bounding_box` before calculating area",
                )));
            }
        };

        let voronoi_opt = VoronoiBuilder::default()
            .set_sites(
                sites
                    .iter()
                    .map(|s| voronoice::Point { x: s.x, y: s.y })
                    .collect(),
            )
            .set_bounding_box(BoundingBox::new(
                voronoice::Point {
                    x: (b_max.x - b_min.x) / 2.0,
                    y: (b_max.y - b_min.y) / 2.0,
                },
                b_max.x - b_min.x,
                b_max.y - b_min.y,
            ))
            .build();

        if let Some(voronoi) = voronoi_opt {
            let areas: Vec<Area> = voronoi
                .iter_cells()
                .map(|cell| {
                    let vertices = cell.iter_vertices().collect::<Vec<&Point>>();
                    let mut area = 0.0;
                    for i in 0..vertices.len() {
                        let j = (i + 1) % vertices.len();
                        area += vertices[i].x * vertices[j].y - vertices[j].x * vertices[i].y;
                    }
                    area.abs() / 2.0
                })
                .collect();
            Ok(areas)
        } else {
            Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Failed to calculate area",
            )))
        }
    }

    /// Get the reference to the sites.
    /// Even if the sites are set using `set_sites`,
    /// the values may be changed after doing some operations like `iterate_sites`.
    pub fn get_sites(&self) -> Result<&Vec<Site>, Box<impl error::Error>> {
        if let Some(sites) = &self.sites {
            Ok(sites)
        } else {
            Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "You must set sites using `set_sites` before getting sites",
            )))
        }
    }
}
