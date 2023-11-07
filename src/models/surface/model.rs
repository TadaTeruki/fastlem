use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;

use crate::core::{
    traits::Model,
    units::{Altitude, Area, Length},
};

use super::{interpolator::TerrainInterpolator2D, sites::Site2D, terrain::Terrain2D};

/// A set of fundamental data required for genreating terrain.
///
/// ### Properties
/// - `sites` is the set of sites.
/// - `areas` is the areas of each site.
/// - `graph` is the graph representing the conecctions between sites.
/// - `outlets` is the set of outlets.
/// - `triangles` is the set of triangles created by delaunay triangulation.
pub struct TerrainModel2D {
    sites: Vec<Site2D>,
    areas: Vec<Area>,
    graph: EdgeAttributedUndirectedGraph<Length>,
    outlets: Vec<usize>,
}

impl TerrainModel2D {
    pub(super) fn new(
        sites: Vec<Site2D>,
        areas: Vec<Area>,
        graph: EdgeAttributedUndirectedGraph<Length>,
        outlets: Vec<usize>,
    ) -> Self {
        Self {
            sites,
            areas,
            graph,
            outlets,
        }
    }
}

impl Model<Site2D, Terrain2D> for TerrainModel2D {
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

    fn create_terrain_from_result(&self, altitudes: &[Altitude]) -> Terrain2D {
        Terrain2D::new(
            self.sites.clone(),
            altitudes.to_vec(),
            TerrainInterpolator2D::new(&self.sites),
        )
    }
}
