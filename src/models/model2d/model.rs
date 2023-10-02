use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;

use crate::core::{
    traits::Model,
    units::{Area, Length},
};

use super::{interpolation::TerrainInterpolator2D, sites::Site2D};

/// A set of fundamental data required for genreating terrain.
///
/// ### Properties
/// - `sites` is the set of sites.
/// - `areas` is the areas of each site.
/// - `graph` is the graph representing the conecctions between sites.
/// - `outlets` is the set of outlets.
/// - `triangles` is the set of triangles created by delaunay triangulation.
pub struct TerrainModel2D {
    pub sites: Vec<Site2D>,
    pub areas: Vec<Area>,
    pub graph: EdgeAttributedUndirectedGraph<Length>,
    pub outlets: Vec<usize>,
    pub triangles: Vec<[usize; 3]>,
}

impl Model<Site2D, TerrainInterpolator2D> for TerrainModel2D {
    fn num(&self) -> usize {
        self.graph.order()
    }

    fn sites(&self) -> &[Site2D] {
        &self.sites
    }

    fn areas(&self) -> &[Area] {
        &self.areas
    }

    fn outlets(&self) -> &[usize] {
        &self.outlets
    }

    fn graph(&self) -> &EdgeAttributedUndirectedGraph<Length> {
        &self.graph
    }

    fn create_interpolator(&self) -> TerrainInterpolator2D {
        let mut collection = TerrainInterpolator2D::new(&self.sites);
        self.triangles.iter().for_each(|t| {
            collection.insert(*t);
        });
        collection
    }
}
