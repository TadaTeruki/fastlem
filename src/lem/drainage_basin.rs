use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;

use crate::{core::units::Length, lem::stream_tree};

/// Represents the drainage basin.
/// This enables to iterate over the sites in the drainage basin with no duplication.
pub struct DrainageBasin {
    traversal: Vec<usize>,
}

impl DrainageBasin {
    /// Construct a new `DrainageBasin` from the given outlet.
    pub fn construct(
        outlet: usize,
        stream_tree: &stream_tree::StreamTree,
        graph: &EdgeAttributedUndirectedGraph<Length>,
    ) -> Self {
        let mut traversal: Vec<usize> = Vec::new();
        traversal.push(outlet);
        let mut i = 0;
        loop {
            let it = traversal[i];
            graph.neighbors_of(it).iter().for_each(|ja| {
                let jt = ja.0;
                if stream_tree.next[jt] == it {
                    traversal.push(jt);
                }
            });
            i += 1;
            if i >= traversal.len() {
                break;
            }
        }

        Self { traversal }
    }

    /// Iterates over the sites in the drainage basin from the outlet to the upstream.
    pub fn for_each_upstream(&self, mut f: impl FnMut(usize)) {
        self.traversal.iter().for_each(|i| f(*i));
    }

    /// Iterates over the sites in the drainage basin from the top of the stream to the downstream.
    pub fn for_each_downstream(&self, mut f: impl FnMut(usize)) {
        self.traversal.iter().rev().for_each(|i| f(*i));
    }
}
