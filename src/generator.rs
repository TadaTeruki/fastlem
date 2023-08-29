use delaunator::{triangulate, Point};
use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;

use crate::{
    drainage_basin::DrainageBasin,
    model::TerrainModel,
    stream_tree,
    terrain::Terrain,
    units::{Altitude, Erodibility, Length, Site, Slope, Step, UpliftRate},
};

/// The default value of the exponent `m` for calculating stream power.
const DEFAULT_M_EXP: f64 = 0.5;

/// Provides methods for generating terrain.
///
/// ### Required parameters
///  - `model` is the set of sites.
///  - `uplift_rate_func` is the function that calculates uplift rates.
///  - `erodibility_func` is the function that calculates erodibilities.
/// ### Optional parameters
///  - `base_altitudes` is the base altitudes of sites. If `None`, the base altitudes will be set to zero.
///  - `custom_outlets` is he custom outlets of sites. If `None`, the outlets will be computed from the convex hull of the sites.
///  - `max_slope_func` is the function that calculates maximum slopes. If `None`, the slopes will not be checked. The return value should be always between 0 and pi/2.
///  - `max_iteration` is the maximum number of iterations. If `None`, the iterations will be repeated until the altitudes of all sites are stable.
///  - `m_exp` is the constants for calculating stream power. If `None`, the default value is 0.5.
#[derive(Default)]
pub struct TerrainGenerator {
    model: Option<TerrainModel>,
    base_altitudes: Option<Vec<Altitude>>,
    uplift_rate_func: Option<Box<dyn Fn(Step, Site) -> UpliftRate>>,
    erodibility_func: Option<Box<dyn Fn(Step, Site) -> Erodibility>>,
    max_slope_func: Option<Box<dyn Fn(Step, Site) -> Slope>>,
    custom_outlets: Option<Vec<usize>>,
    max_iteration: Option<Step>,
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
                let sites = model.get_sites()?;
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

    /// Set the constant uplift rates.
    pub fn set_uplift_rate(mut self, uplift_rate: UpliftRate) -> Self {
        self.uplift_rate_func = Some(Box::new(move |_, _| uplift_rate));
        self
    }

    /// Set the function that calculates uplift rates.
    pub fn set_uplift_rate_func(
        mut self,
        uplift_rate_func: Box<dyn Fn(Step, Site) -> UpliftRate>,
    ) -> Self {
        self.uplift_rate_func = Some(uplift_rate_func);
        self
    }

    /// Set the constant erodibilities.
    pub fn set_erodibility(mut self, erodibility: Erodibility) -> Self {
        self.erodibility_func = Some(Box::new(move |_, _| erodibility));
        self
    }

    /// Set the function that calculates erodibilities.
    pub fn set_erodibility_func(
        mut self,
        erodibility_func: Box<dyn Fn(Step, Site) -> Erodibility>,
    ) -> Self {
        self.erodibility_func = Some(erodibility_func);
        self
    }

    /// Set the constant maximum slopes.
    /// The slope should be between 0 and pi/2;
    pub fn set_max_slope(mut self, max_slope: Slope) -> Self {
        self.max_slope_func = Some(Box::new(move |_, _| max_slope));
        self
    }

    /// Set the function that calculates maximum slopes.
    /// The slope should be always between 0 and pi/2;
    pub fn set_max_slope_func(mut self, max_slope_func: Box<dyn Fn(Step, Site) -> Slope>) -> Self {
        self.max_slope_func = Some(max_slope_func);
        self
    }

    /// Set the maximum number of iterations.
    pub fn set_max_iteration(mut self, max_iteration: Step) -> Self {
        self.max_iteration = Some(max_iteration);
        self
    }

    /// Set the custom outlets of sites.
    pub fn set_custom_outlets(mut self, custom_outlets: Vec<usize>) -> Self {
        self.custom_outlets = Some(custom_outlets);
        self
    }

    /// Set the exponent `m` for calculating stream power.
    pub fn set_exponent_m(mut self, m_exp: f64) -> Self {
        self.m_exp = Some(m_exp);
        self
    }

    /// Generate terrain.
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

        let altitudes: Vec<Altitude> = {
            let mut altitudes = base_altitudes;
            let mut step = 0;
            loop {
                let stream_tree =
                    stream_tree::StreamTree::construct(sites, &altitudes, &graph, &outlets);

                let mut drainage_areas = areas.to_vec();
                let mut response_times = vec![0.0; sites.len()];
                let mut changed = false;

                // calculate altitudes for each drainage basin
                outlets.iter().for_each(|&outlet| {
                    // construct drainage basin
                    let drainage_basin = DrainageBasin::construct(outlet, &stream_tree, &graph);

                    // calculate drainage areas
                    drainage_basin.for_each_downstream(|i| {
                        let j = stream_tree.next[i];
                        if j != i {
                            drainage_areas[j] += drainage_areas[i];
                        }
                    });

                    // calculate response times
                    drainage_basin.for_each_upstream(|i| {
                        let j = stream_tree.next[i];
                        let distance: Length = {
                            let (ok, edge) = graph.has_edge(i, j);
                            if ok {
                                edge
                            } else {
                                0.0
                            }
                        };
                        let celerity =
                            erodibility_func(step, sites[i]) * drainage_areas[i].powf(m_exp);

                        response_times[i] += response_times[j] + 1.0 / celerity * distance;
                    });

                    // calculate altitudes
                    drainage_basin.for_each_upstream(|i| {
                        let mut new_altitude = altitudes[outlet]
                            + uplift_rate_func(step, sites[i])
                                * (response_times[i] - response_times[outlet]);

                        // check if the slope is too steep
                        // if max_slope_func is not set, the slope is not checked
                        if let Some(max_slope_func) = &self.max_slope_func {
                            let j = stream_tree.next[i];
                            let distance: Length = {
                                let (ok, edge) = graph.has_edge(i, j);
                                if ok {
                                    edge
                                } else {
                                    1.0
                                }
                            };
                            let max_slope = max_slope_func(step, sites[i]).tan();
                            let slope = (new_altitude - altitudes[j]) / distance;
                            if slope > max_slope {
                                new_altitude = altitudes[j] + max_slope * distance;
                            }
                        }

                        changed |= new_altitude != altitudes[i];
                        altitudes[i] = new_altitude;
                    });
                });

                // if the altitudes of all sites are stable, break
                if !changed {
                    break;
                }
                step += 1;
                if let Some(max_iteration) = &self.max_iteration {
                    if step >= *max_iteration {
                        break;
                    }
                }
            }

            altitudes
        };

        Ok(Terrain::new(sites.to_vec(), altitudes))
    }
}
