use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;

use crate::{stream_tree, units::Length};

pub struct DrainageBasin {
    traversal: Vec<usize>,
}

impl DrainageBasin {
    pub fn build(
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
                if stream_tree.get_next(jt) == it {
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

    pub fn for_each_upstream(&self, mut f: impl FnMut(usize)) {
        self.traversal.iter().for_each(|i| f(*i));
    }

    pub fn for_each_downstream(&self, mut f: impl FnMut(usize)) {
        self.traversal.iter().rev().for_each(|i| f(*i));
    }
}
