use delaunator::{triangulate, Point};
use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;

use crate::{
    drainage_basin::DrainageBasin,
    model::TerrainModel,
    stream_tree,
    terrain::Terrain,
    units::{Altitude, Erodibility, Length, Site, UpliftRate, Year},
};

const DEFAULT_M_EXP: f64 = 0.5;

/// Provides methods for generating terrain.
///
/// ### Required parameters
///  - `model`: the set of sites.
///  - `uplift_rate_func`: the function that calculates uplift rates.
///  - `erodibility_func`: the function that calculates erodibilities.
/// ### Optional parameters
///  - `base_altitudes`: the base altitudes of sites. If `None`, the base altitudes will be set to zero.
///  - `custom_outlets`: he custom outlets of sites. If `None`, the outlets will be computed from the convex hull of the sites.
///  - `year_step` is the time step of each iteration.
///  - `max_year` is the maximum time of the iteration. If `None`, the iteration will not stop until the altitudes of all sites are stable.
///  - `m_exp` is the constants for calculating stream power. If `None`, the default value is 0.5.
#[derive(Default)]
pub struct TerrainGenerator {
    model: Option<TerrainModel>,
    base_altitudes: Option<Vec<Altitude>>,
    uplift_rate_func: Option<Box<dyn Fn(Year, Site) -> UpliftRate>>,
    erodibility_func: Option<Box<dyn Fn(Year, Site) -> Erodibility>>,
    custom_outlets: Option<Vec<usize>>,
    year_step: Option<Year>,
    max_year: Option<Year>,
    m_exp: Option<f64>,
}

impl TerrainGenerator {
    /// Set the model that contains the set of sites.
    pub fn set_model(mut self, model: TerrainModel) -> Self {
        self.model = Some(model);
        self
    }

    /// Set the base altitudes of sites.
    pub fn set_base_altitudes(mut self, base_altitudes: Vec<Altitude>) -> Self {
        self.base_altitudes = Some(base_altitudes);
        self
    }

    /// Set the base altitudes of sites by function.
    pub fn set_base_altitude_by_func(
        mut self,
        base_altitude: impl Fn(Site) -> Altitude,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let base_altitudes = {
            if let Some(model) = &self.model {
                let sites = model.get_sites().unwrap();
                sites.iter().map(|site| base_altitude(*site)).collect()
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "You must set `TerrainModel` before generating terrain",
                )));
            }
        };
        self.base_altitudes = Some(base_altitudes);
        Ok(self)
    }

    /// Set the function that calculates uplift rates.
    pub fn set_uplift_rate_func(
        mut self,
        uplift_rate_func: Box<dyn Fn(Year, Site) -> UpliftRate>,
    ) -> Self {
        self.uplift_rate_func = Some(uplift_rate_func);
        self
    }

    /// Set the function that calculates erodibilities.
    pub fn set_erodibility_func(
        mut self,
        erodibility_func: Box<dyn Fn(Year, Site) -> Erodibility>,
    ) -> Self {
        self.erodibility_func = Some(erodibility_func);
        self
    }

    /// Set the custom outlets of sites.
    pub fn set_custom_outlets(mut self, custom_outlets: Vec<usize>) -> Self {
        self.custom_outlets = Some(custom_outlets);
        self
    }

    /// Set the time step of each iteration.
    pub fn set_year_step(mut self, year_step: Year) -> Self {
        self.year_step = Some(year_step);
        self
    }

    /// Set the maximum time of the iteration.
    pub fn set_max_year(mut self, max_year: Year) -> Self {
        self.max_year = Some(max_year);
        self
    }

    /// Set the exponent `m` for calculating stream power.
    pub fn set_exponent_m(mut self, m_exp: f64) -> Self {
        self.m_exp = Some(m_exp);
        self
    }

    /// Generate terrain using Salève method.
    pub fn generate(&self) -> Result<Terrain, Box<dyn std::error::Error>> {
        let (sites, areas) = {
            if let Some(model) = &self.model {
                let sites = model.get_sites()?;
                let areas = model.calculate_areas()?;
                (sites, areas)
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "You must set `TerrainModel` before generating terrain",
                )));
            }
        };

        let base_altitudes = {
            if let Some(base_altitudes) = &self.base_altitudes {
                base_altitudes.to_vec()
            } else {
                vec![0.0; sites.len()]
            }
        };

        let uplift_rate_func = {
            if let Some(uplift_rate_func) = &self.uplift_rate_func {
                uplift_rate_func
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "You must set uplift rate function before generating terrain",
                )));
            }
        };

        let erodibility_func = {
            if let Some(erodibility_func) = &self.erodibility_func {
                erodibility_func
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "You must set erodibility function before generating terrain",
                )));
            }
        };

        let m_exp = {
            if let Some(m_exp) = &self.m_exp {
                *m_exp
            } else {
                DEFAULT_M_EXP
            }
        };

        let max_year = self.max_year;

        let year_step = {
            if let Some(year_step) = &self.year_step {
                *year_step
            } else if max_year.is_some() {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "If you specified `max_year` for TerrainBuilder, you must set `year_step` before generating terrain",
                )));
            } else {
                0.0
            }
        };

        let triangulation = {
            let points: Vec<Point> = sites
                .iter()
                .map(|site| Point {
                    x: site.x,
                    y: site.y,
                })
                .collect();
            triangulate(&points)
        };

        let outlets: Vec<usize> = {
            if let Some(outlets) = &self.custom_outlets {
                outlets.to_vec()
            } else {
                triangulation.hull
            }
        };

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

        let is_outlet = {
            let mut is_outlet = vec![false; sites.len()];
            outlets.iter().for_each(|&i| {
                is_outlet[i] = true;
            });
            is_outlet
        };

        let altitudes: Vec<Altitude> = {
            let mut altitudes = base_altitudes;
            let mut year = 0.0;
            loop {
                let stream_tree =
                    stream_tree::StreamTree::build(sites, &altitudes, &graph, &is_outlet);

                let mut drainage_areas = areas.to_vec();
                let mut response_times = vec![0.0; sites.len()];
                let mut changed = false;

                outlets.iter().for_each(|&outlet| {
                    let drainage_basin = DrainageBasin::build(outlet, &stream_tree, &graph);
                    // calculate drainage areas
                    drainage_basin.for_each_downstream(|i| {
                        let j = stream_tree.get_next(i);
                        if j != i {
                            drainage_areas[j] += drainage_areas[i];
                        }
                    });
                    // calculate response times
                    drainage_basin.for_each_upstream(|i| {
                        let j = stream_tree.get_next(i);
                        let distance: Length = {
                            let (ok, edge) = graph.has_edge(i, j);
                            if ok {
                                edge
                            } else {
                                0.0
                            }
                        };
                        let celerity =
                            erodibility_func(year, sites[i]) * drainage_areas[i].powf(m_exp);

                        response_times[i] += response_times[j] + 1.0 / celerity * distance;
                    });
                    // calculate altitudes
                    drainage_basin.for_each_upstream(|i| {
                        let new_altitude = altitudes[outlet]
                            + uplift_rate_func(year, sites[i])
                                * (response_times[i] - response_times[outlet]);
                        changed |= new_altitude != altitudes[i];
                        altitudes[i] = new_altitude;
                    });
                });
                year += year_step;

                if !changed {
                    break;
                }
                if let Some(max_year) = max_year {
                    if year > max_year {
                        break;
                    }
                }
            }

            altitudes
        };

        Ok(Terrain::new(sites.to_vec(), altitudes))
    }
}
