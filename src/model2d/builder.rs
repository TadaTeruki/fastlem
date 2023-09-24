use std::{error, io};

use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;
use voronoice::{BoundingBox, VoronoiBuilder};

use crate::units::{Area, Length, Site};

use super::{model::TerrainModel2D, sites::Site2D};

/// Default margin for bounding box.
/// This value is used when the bounding box is calculated from the minimum and maximum values of the sites.
///
/// **TODO**: This value should be deprecated but it is still defined to avoid a bug. The reason of the bug should be investigated.
const DEFAULT_BOUDNING_BOX_MARGIN: f64 = 1e-9;

/// Provides methods to construct a `TerrainModel2D`.
///
/// ### Required parameters
/// - `sites` is the set of sites.
/// ### Optional parameters
/// - `bound_min` and `bound_max` are the bounding rectangle of the sites. If not set, the bounding rectangle will be computed from the sites.
///    This parameter is used to calculate the area or to relocate the sites to apploximately evenly spaced positions using Lloyd's algorithm.
/// - `custom_outlets` is the set of outlets. If not set, the convex hull of the sites will be used as the outlets.
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
    ) -> Result<Self, Box<dyn error::Error>> {
        self.bound_min = bound_min;
        self.bound_max = bound_max;
        Ok(self)
    }

    /// Relocate the sites to apploximately evenly spaced positions using Lloyd's algorithm.
    /// The number of iterations for Lloyd's algorithm is specified by `iterations`.
    pub fn iterate_sites(mut self, iterations: usize) -> Result<Self, Box<dyn error::Error>> {
        if iterations == 0 {
            return Ok(self);
        }

        let (bound_min, bound_max) = (self.query_bound_min()?, self.query_bound_max()?);

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

        let voronoi_opt = VoronoiBuilder::default()
            .set_sites(
                sites
                    .iter()
                    .map(|s| voronoice::Point { x: s.x, y: s.y })
                    .collect(),
            )
            .set_bounding_box(BoundingBox::new(
                voronoice::Point {
                    x: (bound_max.x + bound_min.x) / 2.0,
                    y: (bound_max.y + bound_min.y) / 2.0,
                },
                bound_max.x - bound_min.x,
                bound_max.y - bound_min.y,
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

    pub fn build(&self) -> Result<TerrainModel2D, Box<dyn error::Error>> {
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

        let (bound_min, bound_max) = (self.query_bound_min()?, self.query_bound_max()?);

        let voronoi_opt = VoronoiBuilder::default()
            .set_sites(
                sites
                    .iter()
                    .map(|s| voronoice::Point { x: s.x, y: s.y })
                    .collect(),
            )
            .set_bounding_box(BoundingBox::new(
                voronoice::Point {
                    x: (bound_max.x + bound_min.x) / 2.0,
                    y: (bound_max.y + bound_min.y) / 2.0,
                },
                bound_max.x - bound_min.x,
                bound_max.y - bound_min.y,
            ))
            .build();

        if let Some(voronoi) = voronoi_opt {
            let areas: Vec<Area> = voronoi
                .iter_cells()
                .map(|cell| {
                    let vertices = cell.iter_vertices().collect::<Vec<_>>();
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
                    let (a, b, c) = (triangle[0], triangle[1], triangle[2]);

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
                "Failed to calculate voronoi diagram",
            )))
        }
    }

    fn query_bound_min(&self) -> Result<Site2D, Box<dyn error::Error>> {
        if let Some(bound_min) = self.bound_min {
            Ok(bound_min)
        } else if let Some(sites) = &self.sites {
            let bound_min = sites.iter().fold(
                Site2D {
                    x: std::f64::MAX,
                    y: std::f64::MAX,
                },
                |acc, s| Site2D {
                    x: acc.x.min(s.x) - DEFAULT_BOUDNING_BOX_MARGIN,
                    y: acc.y.min(s.y) - DEFAULT_BOUDNING_BOX_MARGIN,
                },
            );
            Ok(bound_min)
        } else {
            Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "You must set sites using `set_sites` before calculating area",
            )))
        }
    }

    fn query_bound_max(&self) -> Result<Site2D, Box<dyn error::Error>> {
        if let Some(bound_max) = self.bound_max {
            Ok(bound_max)
        } else if let Some(sites) = &self.sites {
            let bound_max = sites.iter().fold(
                Site2D {
                    x: std::f64::MIN,
                    y: std::f64::MIN,
                },
                |acc, s| Site2D {
                    x: acc.x.max(s.x) + DEFAULT_BOUDNING_BOX_MARGIN,
                    y: acc.y.max(s.y) + DEFAULT_BOUDNING_BOX_MARGIN,
                },
            );
            Ok(bound_max)
        } else {
            Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "You must set sites using `set_sites` before calculating area",
            )))
        }
    }
}
