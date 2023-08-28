use crate::units::{Altitude, Site};

#[derive(Default)]
pub struct Terrain {
    pub sites: Vec<Site>,
    pub altitudes: Vec<Altitude>,
}

impl Terrain {
    pub fn new(sites: Vec<Site>, altitudes: Vec<Altitude>) -> Self {
        Self { sites, altitudes }
    }
}
