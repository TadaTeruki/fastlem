use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;

use super::units::{Altitude, Area, Length};

pub trait Site: Copy + Clone + Default {
    /// Calculate the distance between two sites.
    fn distance(&self, other: &Self) -> Length;

    /// Calculate the squared distance between two sites.
    fn squared_distance(&self, other: &Self) -> Length;
}

pub trait Model<S: Site, I: TerrainInterpolator<S>> {
    fn num(&self) -> usize;
    fn sites(&self) -> &[S];
    fn areas(&self) -> &[Area];
    fn outlets(&self) -> &[usize];
    fn graph(&self) -> &EdgeAttributedUndirectedGraph<Length>;
    fn create_interpolator(&self) -> I;
}

pub trait TerrainInterpolator<S: Site> {
    fn interpolate(&self, altitudes: &[Altitude], site: &S) -> Option<Altitude>;
}
