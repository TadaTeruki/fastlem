use std::{error, io};

use delaunator::Point;
use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;
use voronoice::{BoundingBox, VoronoiBuilder};

use crate::units::{Area, Length, Site};

use super::{model::TerrainModel2D, sites::Site2D};

/// A set of sites for representing the terrain.
#[derive(Default, Clone)]
pub struct TerrainModel2DBulider {
    sites: Option<Vec<Site2D>>,
    bound_min: Option<Site2D>,
    bound_max: Option<Site2D>,
    custom_outlets: Option<Vec<usize>>,
}

impl TerrainModel2DBulider {
    /// Set sites.
    pub fn set_sites(mut self, sites: Vec<Site2D>) -> Self {
        self.sites = Some(sites);
        self
    }

    /// Set the bounding rectangle of the sites.
    ///
    /// If `bound_min` and `bound_max` is `None`, the bounding rectangle will be
    /// computed from the sites.
    pub fn set_bounding_box(
        mut self,
        bound_min: Option<Site2D>,
        bound_max: Option<Site2D>,
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
                    Site2D {
                        x: std::f64::MAX,
                        y: std::f64::MAX,
                    },
                    |acc, s| Site2D {
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
                    Site2D {
                        x: std::f64::MIN,
                        y: std::f64::MIN,
                    },
                    |acc, s| Site2D {
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
                    x: (b_max.x + b_min.x) / 2.0,
                    y: (b_max.y + b_min.y) / 2.0,
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
                    .map(|s| Site2D { x: s.x, y: s.y })
                    .collect::<Vec<Site2D>>(),
            );
        }

        Ok(self)
    }

    /// Set the custom outlets of sites.
    pub fn set_custom_outlets(mut self, custom_outlets: Vec<usize>) -> Self {
        self.custom_outlets = Some(custom_outlets);
        self
    }

    pub fn build(&self) -> Result<TerrainModel2D, Box<impl error::Error>> {
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
                    x: (b_max.x + b_min.x) / 2.0,
                    y: (b_max.y + b_min.y) / 2.0,
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

            let triangulation = voronoi.triangulation();

            let graph: EdgeAttributedUndirectedGraph<Length> = {
                let mut graph: EdgeAttributedUndirectedGraph<f64> =
                    EdgeAttributedUndirectedGraph::new(sites.len());
                for triangle in triangulation.triangles.chunks_exact(3) {
                    let a = triangle[0];
                    let b = triangle[1];
                    let c = triangle[2];

                    if a < b {
                        graph.add_edge(a, b, sites[a].distance(&sites[b]));
                    }
                    if b < c {
                        graph.add_edge(b, c, sites[b].distance(&sites[c]));
                    }
                    if c < a {
                        graph.add_edge(c, a, sites[c].distance(&sites[a]));
                    }
                }
                graph
            };

            let outlets: Vec<usize> = {
                if let Some(outlets) = &self.custom_outlets {
                    outlets.to_vec()
                } else {
                    triangulation.hull.to_vec()
                }
            };

            Ok(TerrainModel2D {
                sites: sites.to_vec(),
                areas,
                graph,
                outlets,
            })
        } else {
            Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Failed to calculate area",
            )))
        }
    }
}
