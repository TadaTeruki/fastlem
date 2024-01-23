use rand::{rngs::StdRng, Rng, SeedableRng};
use std::marker::PhantomData;
use thiserror::Error;

use crate::{
    core::{
        parameters::TopographicalParameters,
        traits::{Model, Site},
        units::{Elevation, Length, Step},
    },
    lem::drainage_basin::DrainageBasin,
    lem::stream_tree,
};

/// The default value of the exponent `m` for calculating stream power.
const DEFAULT_M_EXP: f64 = 0.5;

#[derive(Error, Debug)]
pub enum GenerationError {
    #[error("The number of topographical parameters must be equal to the number of sites")]
    InvalidNumberOfParameters,
    #[error("You must set topographical parameters before generating terrain")]
    ParametersNotSet,
    #[error("You must set `TerrainModel` before generating terrain")]
    ModelNotSet,
}

/// Provides methods for generating terrain.
///
/// ### Required properties
///  - `model` is the vector representation of the terrain network.
///  - `parameters` is the topographical parameters of sites. Each parameter contains the uplift rates, erodibilities, base elevations and maximum slopes (see [TopographicalParameters] for details).
/// ### Optional properties
///  - `max_iteration` is the maximum number of iterations. If not set, the iterations will be repeated until the elevations of all sites are stable.
///
pub struct TerrainGenerator<S, M, T>
where
    S: Site,
    M: Model<S, T>,
{
    model: Option<M>,
    parameters: Option<Vec<TopographicalParameters>>,
    max_iteration: Option<Step>,
    _phantom: PhantomData<(S, T)>,
}

impl<S, M, T> Default for TerrainGenerator<S, M, T>
where
    S: Site,
    M: Model<S, T>,
{
    fn default() -> Self {
        Self {
            model: None,
            parameters: None,
            max_iteration: None,
            _phantom: PhantomData,
        }
    }
}

impl<S, M, T> TerrainGenerator<S, M, T>
where
    S: Site,
    M: Model<S, T>,
{
    /// Set the model that contains the set of sites.
    pub fn set_model(self, model: M) -> Self {
        Self {
            model: Some(model),
            ..self
        }
    }

    /// Set the topographical parameters of sites. See [TopographicalParameters] about the parameters.
    pub fn set_parameters(self, parameters: Vec<TopographicalParameters>) -> Self {
        Self {
            parameters: Some(parameters),
            ..self
        }
    }

    /// Set the maximum number of iterations.
    ///
    /// The iteration(loop) for calculating elevations will be stopped when the number of iterations reaches `max_iteration`.
    /// If not set, the iterations will be repeated until the elevations of all sites are stable.
    pub fn set_max_iteration(self, max_iteration: Step) -> Self {
        Self {
            max_iteration: Some(max_iteration),
            ..self
        }
    }

    /// Generate terrain.
    pub fn generate(self) -> Result<T, GenerationError> {
        let model = {
            if let Some(model) = &self.model {
                model
            } else {
                return Err(GenerationError::ModelNotSet);
            }
        };

        let (num, sites, areas, graph, default_outlets) = (
            model.num(),
            model.sites(),
            model.areas(),
            model.graph(),
            model.default_outlets(),
        );

        let parameters = {
            if let Some(parameters) = &self.parameters {
                if parameters.len() != num {
                    return Err(GenerationError::InvalidNumberOfParameters);
                }
                parameters
            } else {
                return Err(GenerationError::ParametersNotSet);
            }
        };

        let m_exp = DEFAULT_M_EXP;

        let outlets = {
            let outlets = parameters
                .iter()
                .enumerate()
                .filter(|(_, param)| param.is_outlet)
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            if outlets.is_empty() {
                default_outlets.to_vec()
            } else {
                outlets
            }
        };

        let mut rng: StdRng = SeedableRng::from_seed([0u8; 32]);

        let elevations: Vec<Elevation> = {
            let mut elevations = parameters
                .iter()
                .map(|a| a.base_elevation + rng.gen::<f64>() * f64::EPSILON)
                .collect::<Vec<_>>();
            let mut step = 0;

            loop {
                let stream_tree =
                    stream_tree::StreamTree::construct(sites, &elevations, graph, &outlets);

                let mut drainage_areas = areas.to_vec();
                let mut response_times = vec![0.0; num];
                let mut changed = false;

                // calculate elevations for each drainage basin
                outlets.iter().for_each(|&outlet| {
                    // construct drainage basin
                    let drainage_basin = DrainageBasin::construct(outlet, &stream_tree, graph);

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
                                1.0
                            }
                        };
                        let celerity = parameters[i].erodibility * drainage_areas[i].powf(m_exp);
                        response_times[i] += response_times[j] + 1.0 / celerity * distance;
                    });

                    // calculate elevations
                    drainage_basin.for_each_upstream(|i| {
                        let mut new_elevation = elevations[outlet]
                            + parameters[i].uplift_rate
                                * (response_times[i] - response_times[outlet]).max(0.0);

                        // check if the slope is too steep
                        // if max_slope_func is not set, the slope is not checked
                        if let Some(max_slope) = parameters[i].max_slope {
                            let j = stream_tree.next[i];
                            let distance: Length = {
                                let (ok, edge) = graph.has_edge(i, j);
                                if ok {
                                    edge
                                } else {
                                    1.0
                                }
                            };
                            let max_slope = max_slope.tan();
                            let slope = (new_elevation - elevations[j]) / distance;
                            if slope > max_slope {
                                new_elevation = elevations[j] + max_slope * distance;
                            }
                        }

                        changed |= new_elevation != elevations[i];
                        elevations[i] = new_elevation;
                    });
                });

                // if the elevations of all sites are stable, break
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

            elevations
        };

        Ok(model.create_terrain_from_result(&elevations))
    }
}
