use rand::{rngs::StdRng, Rng, SeedableRng};
use std::marker::PhantomData;
use thiserror::Error;

use crate::{
    core::{
        attributes::TerrainAttributes,
        traits::{Model, Site},
        units::{Altitude, Length, Step},
    },
    lem::drainage_basin::DrainageBasin,
    lem::stream_tree,
};

/// The default value of the exponent `m` for calculating stream power.
const DEFAULT_M_EXP: f64 = 0.5;

#[derive(Error, Debug)]
pub enum GenerationError {
    #[error("The number of attributes must be equal to the number of sites")]
    InvalidNumberOfAttributes,
    #[error("You must set attributes before generating terrain")]
    AttributesNotSet,
    #[error("You must set `TerrainModel` before generating terrain")]
    ModelNotSet,
}

/// Provides methods for generating terrain.
///
/// ### Required parameters
///  - `model` is the vector representation of the terrain network.
///  - `attributes` is the attributes of sites. Each attribute contains the uplift rates, erodibilities, base altitudes and maximum slopes (see [TerrainAttributes] for details).
/// ### Optional parameters
///  - `max_iteration` is the maximum number of iterations. If not set, the iterations will be repeated until the altitudes of all sites are stable.
///
pub struct TerrainGenerator<S, M, T>
where
    S: Site,
    M: Model<S, T>,
{
    model: Option<M>,
    attributes: Option<Vec<TerrainAttributes>>,
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
            attributes: None,
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

    /// Set the attributes of sites.
    pub fn set_attributes(self, attributes: Vec<TerrainAttributes>) -> Self {
        Self {
            attributes: Some(attributes),
            ..self
        }
    }

    /// Set the maximum number of iterations.
    ///
    /// The iteration(loop) for calculating altitudes will be stopped when the number of iterations reaches `max_iteration`.
    /// If not set, the iterations will be repeated until the altitudes of all sites are stable.
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

        let attributes = {
            if let Some(attributes) = &self.attributes {
                if attributes.len() != num {
                    return Err(GenerationError::InvalidNumberOfAttributes);
                }
                attributes
            } else {
                return Err(GenerationError::AttributesNotSet);
            }
        };

        let m_exp = DEFAULT_M_EXP;

        let outlets = {
            let outlets = attributes
                .iter()
                .enumerate()
                .filter(|(_, attribute)| attribute.is_outlet)
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            if outlets.is_empty() {
                default_outlets.to_vec()
            } else {
                outlets
            }
        };

        let mut rng: StdRng = SeedableRng::from_seed([0u8; 32]);

        let altitudes: Vec<Altitude> = {
            let mut altitudes = attributes
                .iter()
                .map(|a| a.base_altitude + rng.gen::<f64>() * f64::EPSILON)
                .collect::<Vec<_>>();
            let mut step = 0;

            loop {
                let stream_tree =
                    stream_tree::StreamTree::construct(sites, &altitudes, graph, &outlets);

                let mut drainage_areas = areas.to_vec();
                let mut response_times = vec![0.0; num];
                let mut changed = false;

                // calculate altitudes for each drainage basin
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
                        let celerity = attributes[i].erodibility * drainage_areas[i].powf(m_exp);
                        response_times[i] += response_times[j] + 1.0 / celerity * distance;
                    });

                    // calculate altitudes
                    drainage_basin.for_each_upstream(|i| {
                        let mut new_altitude = altitudes[outlet]
                            + attributes[i].uplift_rate
                                * (response_times[i] - response_times[outlet]).max(0.0);

                        // check if the slope is too steep
                        // if max_slope_func is not set, the slope is not checked
                        if let Some(max_slope) = attributes[i].max_slope {
                            let j = stream_tree.next[i];
                            let distance: Length = {
                                let (ok, edge) = graph.has_edge(i, j);
                                if ok {
                                    edge
                                } else {
                                    1.0
                                }
                            };
                            let distance = distance * distance;
                            let max_slope = max_slope.tan();
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

        Ok(model.create_terrain_from_result(&altitudes))
    }
}
