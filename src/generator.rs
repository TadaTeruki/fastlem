use delaunator::{triangulate, Point};
use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;

use crate::{
    model::TerrainModel,
    terrain::Terrain,
    stream_tree,
    units::{Altitude, Erodibility, Length, UpliftRate, Year},
};

/// Provides methods for generating terrain.
/// 
/// ### Required parameters
///  - `model`: the set of sites.
///  - `uplift_rates`: the uplift rates of sites.
///  - `erodibilities`: the erodibilities of sites. The value of erodibility depends on many factors, such as the lithology, vegetation, climate, and climate variability.
///  - `m_exp` and `n_exp` is the constants for calculating stream power. 
///
/// ### Optional parameters
///  - `base_altitudes`: the base altitudes of sites. If `None`, the base altitudes will be set to zero.
///  - `custom_outlets`: he custom outlets of sites. If `None`, the outlets will be computed from the convex hull of the sites.
///  - `year_step` is the time step of each iteration.
///  - `max_year` is the maximum time of the iteration. If `None`, the iteration will not stop until the altitudes of all sites are stable.
#[derive(Default)]
pub struct TerrainGenerator {
    model: Option<TerrainModel>,
    base_altitudes: Option<Vec<Altitude>>,
    uplift_rates: Option<Vec<UpliftRate>>,
    erodibilities: Option<Vec<Erodibility>>,
    custom_outlets: Option<Vec<usize>>,
    year_step: Option<Year>,
    max_year: Option<Year>,
    m_exp: Option<f64>,
    n_exp: Option<f64>,
}

impl TerrainGenerator {
    pub fn new() -> Self {
        Self::default()
    }

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

    /// Set the uplift rates of sites.
    pub fn set_uplift_rates(mut self, uplift_rates: Vec<UpliftRate>) -> Self {
        self.uplift_rates = Some(uplift_rates);
        self
    }

    /// Set the erodibilities of sites.
    pub fn set_erodibilities(mut self, erodibilities: Vec<Erodibility>) -> Self {
        self.erodibilities = Some(erodibilities);
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

    /// Set the exponent `n` for calculating stream power.
    pub fn set_exponent_n(mut self, n_exp: f64) -> Self {
        self.n_exp = Some(n_exp);
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
                    "You must set vetor map before generating terrain",
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

        let uplift_rates = {
            if let Some(uplift_rates) = &self.uplift_rates {
                uplift_rates
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "You must set uplift rates before generating terrain",
                )));
            }
        };

        let erodibilities = {
            if let Some(erodibilities) = &self.erodibilities {
                erodibilities
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "You must set erodibilities before generating terrain",
                )));
            }
        };

        let m_exp = {
            if let Some(m_exp) = &self.m_exp {
                m_exp.clone()
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "You must set m_exp before generating terrain",
                )));
            }
        };

        let n_exp = {
            if let Some(n_exp) = &self.n_exp {
                n_exp.clone()
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "You must set n_exp before generating terrain",
                )));
            }
        };

        let max_year = self.max_year;

        let year_step = {
            if let Some(year_step) = &self.year_step {
                year_step.clone()
            } else {
                if max_year.is_some() {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "If you specified `max_year` for TerrainBuilder, you must set `year_step` before generating terrain",
                    )));
                } else {
                    0.0
                }
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
                triangulation.hull.iter().map(|&i| i).collect()
            }
        };

        let graph: EdgeAttributedUndirectedGraph<Length> = {
            let mut graph = EdgeAttributedUndirectedGraph::new(sites.len());
            for triangle in triangulation.triangles.chunks_exact(3) {
                let a = triangle[0];
                let b = triangle[1];
                let c = triangle[2];

                if a < b {
                    graph.add_edge(
                        a,
                        b,
                        sites[a].distance(&sites[b]),
                    );
                }
                if b < c {
                    graph.add_edge(
                        b,
                        c,
                        sites[b].distance(&sites[c]),
                    );
                }
                if c < a {
                    graph.add_edge(
                        c,
                        a,
                        sites[c].distance(&sites[a]),
                    );
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

        let altitudes_: Vec<Altitude> = {
            let mut altitudes = base_altitudes;
            let mut year = 0.0;
            loop {
                let stream_tree =
                    stream_tree::StreamTree::build(&sites, &altitudes, &graph, &is_outlet);
                // calculate drainage area
                let drainage_areas = stream_tree.calculate_drainage_areas(&graph, &areas);
                let response_times = stream_tree.calculate_response_times(
                    &graph,
                    &drainage_areas,
                    &altitudes,
                    &erodibilities,
                    m_exp,
                    n_exp,
                );
                // update altitudes
                let mut changed = false;
                for i in 0..altitudes.len() {
                    let iroot = stream_tree.get_root(i);
                    let new_altitude = altitudes[iroot]
                        + uplift_rates[i] * (response_times[i] - response_times[iroot]);
                    if !changed {
                        changed |= new_altitude != altitudes[i];
                    }
                    altitudes[i] = new_altitude;
                }
                year += year_step;
                if let Some(max_year) = max_year {
                    if year > max_year {
                        break;
                    }
                } else {
                    if !changed {
                        break;
                    }
                }
            }

            altitudes
        };

        Ok(Terrain::new(
            sites.to_vec(),
            altitudes_,
        ))
    }
}

