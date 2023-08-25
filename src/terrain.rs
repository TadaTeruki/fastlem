use crate::units::{Altitude, Site};

#[derive(Default)]
pub struct Terrain {
    sites: Vec<Site>,
    altitudes: Vec<Altitude>,
}

impl Terrain {
    pub fn new(sites: Vec<Site>, altitudes: Vec<Altitude>) -> Self {
        Self { sites, altitudes }
    }

    pub fn get_sites(&self) -> &Vec<Site> {
        &self.sites
    }

    pub fn get_altitudes(&self) -> &Vec<Altitude> {
        &self.altitudes
    }
}
