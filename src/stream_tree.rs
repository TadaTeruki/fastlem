use std::collections::BinaryHeap;
use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;

use crate::units::{Altitude, Area, Erodibility, Length, ResponseTime, Site};

/// Tree structure for representing the flow of water.
///  - `flow_into` is the next site of each site in the flow.
///  - `root` is the root of each site in the flow.
pub struct StreamTree {
    flow_into: Vec<usize>,
    root: Vec<usize>,
}

struct RidgeElement {
    index: usize,
    alt: Altitude,
}

impl PartialEq for RidgeElement {
    fn eq(&self, other: &Self) -> bool {
        self.alt == other.alt
    }
}

impl Eq for RidgeElement {}

impl Ord for RidgeElement {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.alt < other.alt {
            std::cmp::Ordering::Greater
        } else if self.alt > other.alt {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

impl PartialOrd for RidgeElement {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl StreamTree {
    pub fn build(
        sites: &Vec<Site>,
        altitudes: &Vec<Altitude>,
        graph: &EdgeAttributedUndirectedGraph<Length>,
        is_outlet: &Vec<bool>,
    ) -> Self {
        let mut subroot: Vec<Option<usize>> = vec![None; sites.len()];
        let mut next: Vec<usize> = vec![0; sites.len()];

        sites.iter().enumerate().for_each(|(i, _)| {
            if is_outlet[i] {
                subroot[i] = Some(i);
                next[i] = i;
                return;
            }

            subroot[i] = None;
            next[i] = i;

            let mut steepest_slope = 0.0;
            graph
                .neighbors_of(i)
                .iter()
                .enumerate()
                .for_each(|(_, ja)| {
                    let j = ja.0;
                    if altitudes[i] <= altitudes[j] {
                        return;
                    }
                    if let Some(distance) = graph.has_edge(i, j).1 {
                        let down_hill_slope = (altitudes[i] - altitudes[j]) / distance as f64;
                        if down_hill_slope > steepest_slope {
                            steepest_slope = down_hill_slope;
                            next[i] = j;
                        }
                    }
                });
        });

        // find roots
        let mut subflipgroup: Vec<usize> = Vec::with_capacity(sites.len());
        let mut has_lake = false;

        sites.iter().enumerate().for_each(|(i, _)| {
            if subroot[i].is_some() {
                return;
            }
            let mut iv = i;
            let mut ir = None;
            subflipgroup.push(iv);
            loop {
                if next[iv] == iv {
                    if !is_outlet[iv] {
                        has_lake = true;
                    }
                    ir = Some(iv);
                    break;
                }
                if subroot[iv].is_some() {
                    ir = Some(subroot[iv].unwrap());
                    break;
                }
                iv = next[iv];
                subflipgroup.push(iv);
            }
            subflipgroup.iter().for_each(|&is| {
                subroot[is] = ir;
            });

            subflipgroup.clear();
        });

        let subroot = subroot.iter().map(|&r| r.unwrap()).collect();

        // if there are no lakes, stream tree is already complete
        if !has_lake {
            return StreamTree {
                flow_into: next,
                root: subroot,
            };
        }

        // final roots of the stream tree
        let mut root: Vec<Option<usize>> = vec![None; sites.len()];
        let mut ridgestack: BinaryHeap<RidgeElement> = BinaryHeap::with_capacity(sites.len());

        sites.iter().enumerate().for_each(|(i, _)| {
            if is_outlet[i] {
                root[i] = Some(i);

                ridgestack.push(RidgeElement {
                    index: i,
                    alt: altitudes[i],
                });
            } else {
                root[i] = None;
            }
        });

        // remove lakes
        let mut visited: Vec<bool> = vec![false; sites.len()];

        loop {
            let element = {
                if let Some(element) = ridgestack.pop() {
                    element
                } else {
                    break;
                }
            };
            let i = element.index;

            if visited[i] {
                continue;
            }

            graph
                .neighbors_of(i)
                .iter()
                .enumerate()
                .for_each(|(_, ja)| {
                    let j = ja.0;
                    if visited[j] {
                        root[i] = root[subroot[i]];
                        return;
                    }

                    if root[subroot[j]].is_none() {
                        let mut k = j;
                        let mut nk = i;
                        loop {
                            if next[k] != k {
                                // flip flow
                                let tmp = next[k];
                                next[k] = nk;
                                nk = k;
                                k = tmp;
                            } else {
                                break;
                            }
                        }
                        next[k] = nk;
                        root[subroot[j]] = root[subroot[i]];
                    }

                    root[i] = root[subroot[i]];
                    if !visited[j] {
                        ridgestack.push(RidgeElement {
                            index: j,
                            alt: altitudes[j],
                        });
                    }
                });
            visited[i] = true;
        }

        StreamTree {
            flow_into: next,
            root: root.iter().map(|&r| r.unwrap()).collect(),
        }
    }

    pub fn calculate_drainage_areas(
        &self,
        graph: &EdgeAttributedUndirectedGraph<Length>,
        areas: &Vec<Area>,
    ) -> Vec<Area> {
        let mut drainage_areas: Vec<Area> = vec![0.0; graph.order()];

        self.root.iter().enumerate().for_each(|(i, _)| {
            let mut iv = i;
            loop {
                drainage_areas[iv] += areas[i];
                if self.flow_into[iv] == iv {
                    break;
                }
                iv = self.flow_into[iv];
            }
        });

        drainage_areas
    }

    pub fn calculate_response_times(
        &self,
        graph: &EdgeAttributedUndirectedGraph<Length>,
        drainage_areas: &Vec<Area>,
        altitudes: &Vec<Altitude>,
        erodibilities: &Vec<Erodibility>,
        m_exp: f64,
        n_exp: f64,
    ) -> Vec<ResponseTime> {
        let celerities = altitudes
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let j = self.get_flow_into(i);
                let slope = {
                    if i == j {
                        1.0
                    } else {
                        let distance = graph.has_edge(i, j).1.unwrap();
                        (altitudes[i] - altitudes[j]) / distance
                    }
                };
                erodibilities[i] * drainage_areas[i].powf(m_exp) * slope.powf(n_exp - 1.0)
            })
            .collect::<Vec<_>>();

        let mut response_times: Vec<ResponseTime> = vec![0.0; altitudes.len()];

        self.root.iter().enumerate().for_each(|(i, _)| {
            let mut iv = i;
            loop {
                response_times[i] += 1.0 / celerities[iv];
                if self.flow_into[iv] == iv {
                    break;
                }
                iv = self.flow_into[iv];
            }
        });

        response_times
    }

    pub fn get_flow_into(&self, i: usize) -> usize {
        self.flow_into[i]
    }

    pub fn get_root(&self, i: usize) -> usize {
        self.root[i]
    }
}
