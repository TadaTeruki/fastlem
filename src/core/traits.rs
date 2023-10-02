use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;

use super::units::{Area, Length};

pub trait Site: Copy + Clone + Default {
    /// Calculate the distance between two sites.
    fn distance(&self, other: &Self) -> Length;

    /// Calculate the squared distance between two sites.
    fn squared_distance(&self, other: &Self) -> Length;
}

pub trait Model<S: Site, TC: TriangleCollection<S>> {
    fn num(&self) -> usize;
    fn sites(&self) -> &[S];
    fn areas(&self) -> &[Area];
    fn outlets(&self) -> &[usize];
    fn graph(&self) -> &EdgeAttributedUndirectedGraph<Length>;
    fn create_triangle_collection(&self) -> TC;
}

pub trait TriangleCollection<S: Site> {
    fn search(&self, site: &S) -> Option<[usize; 3]>;
    fn interpolate(&self, triangle: [usize; 3], site: &S) -> [f64; 3];
}
