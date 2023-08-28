use std::collections::BinaryHeap;
use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;

use crate::units::{Altitude, Length, Site};

/// Tree structure for representing the flow of water.
///  - `next` is the next site of each site in the flow.
///  - `root` is the root(outlet) of each site in the flow.
pub struct StreamTree {
    next: Vec<usize>,
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
        other.alt.partial_cmp(&self.alt).unwrap()
    }
}

impl PartialOrd for RidgeElement {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

struct StreamOriginElement {
    stream_order: usize,
}

impl PartialEq for StreamOriginElement {
    fn eq(&self, other: &Self) -> bool {
        self.stream_order == other.stream_order
    }
}

impl Eq for StreamOriginElement {}

impl PartialOrd for StreamOriginElement {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StreamOriginElement {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.stream_order.cmp(&self.stream_order)
    }
}

impl StreamTree {
    pub fn build(
        sites: &Vec<Site>,
        altitudes: &[Altitude],
        graph: &EdgeAttributedUndirectedGraph<Length>,
        is_outlet: &[bool],
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
            graph.neighbors_of(i).iter().for_each(|ja| {
                let j = ja.0;
                if altitudes[i] > altitudes[j] {
                    let distance = ja.1;
                    let down_hill_slope = (altitudes[i] - altitudes[j]) / distance;
                    if down_hill_slope > steepest_slope {
                        steepest_slope = down_hill_slope;
                        next[i] = j;
                    }
                }
            });
        });

        // find roots
        let mut has_lake = false;

        sites.iter().enumerate().for_each(|(i, _)| {
            if subroot[i].is_some() {
                return;
            }
            let mut iv = i;
            while subroot[iv].is_none() && iv != next[iv] {
                iv = next[iv];
            }

            let ir = {
                if subroot[iv].is_none() {
                    has_lake = true;
                    Some(iv)
                } else {
                    subroot[iv]
                }
            };

            let mut iv = i;
            while subroot[iv].is_none() && iv != next[iv] {
                subroot[iv] = ir;
                iv = next[iv];
            }
            subroot[iv] = ir;
        });

        // if there are no lakes, stream tree is already complete
        if !has_lake {
            return StreamTree { next };
        }

        let subroot: Vec<usize> = subroot.iter().map(|&r| r.unwrap()).collect();

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
        let mut subaltitudes = altitudes.to_owned();

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
                        return;
                    }

                    if root[subroot[j]].is_none() {
                        let mut k = j;
                        let mut nk = i;
                        loop {
                            if next[k] != k {
                                // flip flow
                                let tmp = next[k];
                                subaltitudes[k] = subaltitudes[i];
                                next[k] = nk;
                                nk = k;
                                k = tmp;
                            } else {
                                break;
                            }
                        }
                        next[k] = nk;
                        subaltitudes[k] = subaltitudes[i];
                        root[subroot[j]] = root[subroot[i]];
                    }

                    let distance = ja.1;
                    ridgestack.push(RidgeElement {
                        index: j,
                        alt: (subaltitudes[j] - subaltitudes[i]) * distance,
                    });
                });
            root[i] = root[subroot[i]];
            visited[i] = true;
        }

        StreamTree { next }
    }

    pub fn get_next(&self, i: usize) -> usize {
        self.next[i]
    }
}
