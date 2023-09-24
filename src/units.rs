use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;

/// Length (unit: L);
pub type Length = f64;

/// Altitude (unit: L).
pub type Altitude = f64;

/// Uplift rate (unit: L/T).
pub type UpliftRate = f64;

/// Erodibility.
pub type Erodibility = f64;

/// Area (unit: L^2).
pub type Area = f64;

/// Slope (unit: rad).
pub type Slope = f64;

/// Iteration step.
pub type Step = u32;

/// Response Time.
pub type ResponseTime = f64;

pub trait Site: Copy + Clone + Default {
    /// Calculate the distance between two sites.
    fn distance(&self, other: &Self) -> Length;

    /// Calculate the squared distance between two sites.
    fn squared_distance(&self, other: &Self) -> Length;
}

pub trait Model<S: Site> {
    fn num(&self) -> usize;
    fn sites(&self) -> &[S];
    fn areas(&self) -> &[Area];
    fn outlets(&self) -> &[usize];
    fn graph(&self) -> &EdgeAttributedUndirectedGraph<Length>;
    fn triangles(&self) -> &[[usize; 3]];
}
