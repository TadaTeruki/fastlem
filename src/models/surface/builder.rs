use rand::Rng;
use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;
use thiserror::Error;
use voronoice::{BoundingBox, VoronoiBuilder};

use crate::core::{
    traits::Site,
    units::{Area, Length},
};

use super::{model::TerrainModel2D, sites::Site2D};

/// Default margin for bounding box.
/// This value is used when the bounding box is calculated from the minimum and maximum values of the sites.

#[derive(Error, Debug)]
pub enum ModelBuilderError {
    #[error("You must set sites using `set_sites` before calculating area")]
    SitesNotSet,
    #[error("Failed to calculate voronoi diagram")]
    VoronoiError,
}

/// Provides methods to construct a `TerrainModel2D`.
///
/// ### Required parameters
/// - `sites` is the set of sites.
/// ### Optional parameters
/// - `bound_min` and `bound_max` are the bounding rectangle of the sites. If not set, the bounding rectangle will be computed from the sites.
///    This parameter is used to calculate the area or to relocate the sites to apploximately evenly spaced positions using Lloyd's algorithm.
#[derive(Default, Clone)]
pub struct TerrainModel2DBulider {
    sites: Option<Vec<Site2D>>,
    bound_min: Option<Site2D>,
    bound_max: Option<Site2D>,
}

impl TerrainModel2DBulider {
    pub fn from_random_sites(num: usize, bound_min: Site2D, bound_max: Site2D) -> Self {
        let mut rng = rand::thread_rng();
        let sites = (0..num)
            .map(|_| {
                let x = rng.gen_range(bound_min.x..bound_max.x);
                let y = rng.gen_range(bound_min.y..bound_max.y);
                Site2D { x, y }
            })
            .collect::<Vec<Site2D>>();
        Self {
            sites: Some(sites),
            bound_min: Some(bound_min),
            bound_max: Some(bound_max),
        }
    }

    /// Set sites.
    pub fn set_sites(self, sites: Vec<Site2D>) -> Self {
        Self {
            sites: Some(sites),
            ..self
        }
    }

    /// Set the bounding rectangle of the sites.
    ///
    /// If `bound_min` and `bound_max` is `None`, the bounding rectangle will be
    /// computed from the sites.
    pub fn set_bounding_box(self, bound_min: Option<Site2D>, bound_max: Option<Site2D>) -> Self {
        Self {
            bound_min,
            bound_max,
            ..self
        }
    }

    /// Relocate the sites to apploximately evenly spaced positions using Lloyd's algorithm.
    /// The number of times for Lloyd's algorithm is specified by `times`.
    pub fn relaxate_sites(self, times: usize) -> Result<Self, ModelBuilderError> {
        if times == 0 {
            return Ok(self);
        }

        let (bound_min, bound_max) = (self.query_bound_min()?, self.query_bound_max()?);

        let sites = {
            if let Some(sites) = &self.sites {
                sites
            } else {
                return Err(ModelBuilderError::SitesNotSet);
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
            .set_lloyd_relaxation_iterations(times)
            .build();

        if let Some(voronoi) = voronoi_opt {
            return Ok(Self {
                sites: Some(
                    voronoi
                        .sites()
                        .iter()
                        .map(|s| Site2D { x: s.x, y: s.y })
                        .collect::<Vec<Site2D>>(),
                ),
                ..self
            });
        }

        Ok(self)
    }

    pub fn build(&self) -> Result<TerrainModel2D, ModelBuilderError> {
        let sites = {
            if let Some(sites) = &self.sites {
                sites
            } else {
                return Err(ModelBuilderError::SitesNotSet);
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

            let default_outlets = triangulation.hull.to_vec();

            Ok(TerrainModel2D::new(
                sites.to_vec(),
                areas,
                graph,
                default_outlets,
            ))
        } else {
            Err(ModelBuilderError::VoronoiError)
        }
    }

    fn query_bound_min(&self) -> Result<Site2D, ModelBuilderError> {
        if let Some(bound_min) = self.bound_min {
            Ok(bound_min)
        } else if let Some(sites) = &self.sites {
            let bound_min = sites.iter().fold(
                Site2D {
                    x: std::f64::MAX,
                    y: std::f64::MAX,
                },
                |acc, s| Site2D {
                    x: acc.x.min(s.x),
                    y: acc.y.min(s.y),
                },
            );
            Ok(bound_min)
        } else {
            Err(ModelBuilderError::SitesNotSet)
        }
    }

    fn query_bound_max(&self) -> Result<Site2D, ModelBuilderError> {
        if let Some(bound_max) = self.bound_max {
            Ok(bound_max)
        } else if let Some(sites) = &self.sites {
            let bound_max = sites.iter().fold(
                Site2D {
                    x: std::f64::MIN,
                    y: std::f64::MIN,
                },
                |acc, s| Site2D {
                    x: acc.x.max(s.x),
                    y: acc.y.max(s.y),
                },
            );
            Ok(bound_max)
        } else {
            Err(ModelBuilderError::SitesNotSet)
        }
    }
}
