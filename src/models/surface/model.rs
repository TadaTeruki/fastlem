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
/// - `default_outlets` is the set of indices of sites that are set as outlets by default.
pub struct TerrainModel2D {
    sites: Vec<Site2D>,
    areas: Vec<Area>,
    graph: EdgeAttributedUndirectedGraph<Length>,
    default_outlets: Vec<usize>,
}

impl TerrainModel2D {
    pub(super) fn new(
        sites: Vec<Site2D>,
        areas: Vec<Area>,
        graph: EdgeAttributedUndirectedGraph<Length>,
        default_outlets: Vec<usize>,
    ) -> Self {
        Self {
            sites,
            areas,
            graph,
            default_outlets,
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

    fn default_outlets(&self) -> &[usize] {
        &self.default_outlets
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
