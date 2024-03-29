use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;

use super::units::{Area, Elevation, Length};

pub trait Site: Copy + Clone + Default {
    /// Calculate the distance between two sites.
    fn distance(&self, other: &Self) -> Length;

    /// Calculate the squared distance between two sites.
    fn squared_distance(&self, other: &Self) -> Length;
}

pub trait Model<S: Site, T> {
    fn num(&self) -> usize;
    fn sites(&self) -> &[S];
    fn areas(&self) -> &[Area];
    fn default_outlets(&self) -> &[usize];
    fn graph(&self) -> &EdgeAttributedUndirectedGraph<Length>;
    fn create_terrain_from_result(&self, elevation: &[Elevation]) -> T;
}
