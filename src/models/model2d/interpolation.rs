use rtree_rs::{RTree, Rect};

use crate::core::traits::TerrainInterpolator;

use super::sites::Site2D;

pub struct TerrainInterpolator2D {
    sites: Vec<Site2D>,
    tree: RTree<2, f64, [usize; 3]>,
}

impl TerrainInterpolator2D {
    pub fn new(sites: &[Site2D]) -> Self {
        Self {
            sites: sites.to_vec(),
            tree: RTree::new(),
        }
    }

    pub fn insert(&mut self, triangle: [usize; 3]) {
        let (a, b, c) = (triangle[0], triangle[1], triangle[2]);
        let min_x = self.sites[a].x.min(self.sites[b].x).min(self.sites[c].x);
        let max_x = self.sites[a].x.max(self.sites[b].x).max(self.sites[c].x);
        let min_y = self.sites[a].y.min(self.sites[b].y).min(self.sites[c].y);
        let max_y = self.sites[a].y.max(self.sites[b].y).max(self.sites[c].y);
        self.tree.insert(
            Rect::new([min_x, min_y], [max_x, max_y]),
            [triangle[0], triangle[1], triangle[2]],
        );
    }

    fn get_collision(&self, site: &Site2D) -> Option<[usize; 3]> {
        for item in self.tree.search(Rect::new_point([site.x, site.y])) {
            let triangle = item.data;
            let (a, b, c) = (triangle[0], triangle[1], triangle[2]);
            let sites = [self.sites[a], self.sites[b], self.sites[c]];
            if Self::is_inside_triangle(site, &sites) {
                return Some(*triangle);
            }
        }
        None
    }

    fn is_inside_triangle(site: &Site2D, triangle: &[Site2D; 3]) -> bool {
        let a = &triangle[0];
        let b = &triangle[1];
        let c = &triangle[2];

        let denom = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);

        let lambda1 = ((b.y - c.y) * (site.x - c.x) + (c.x - b.x) * (site.y - c.y)) / denom;
        if !(0.0..=1.0).contains(&lambda1) {
            return false;
        }

        let lambda2 = ((c.y - a.y) * (site.x - c.x) + (a.x - c.x) * (site.y - c.y)) / denom;
        if !(0.0..=1.0).contains(&lambda2) {
            return false;
        }

        let lambda3 = 1.0 - lambda1 - lambda2;
        if !(0.0..=1.0).contains(&lambda3) {
            return false;
        }

        true
    }
}

impl TerrainInterpolator<Site2D> for TerrainInterpolator2D {
    fn search(&self, site: &Site2D) -> Option<[usize; 3]> {
        self.get_collision(site)
    }

    fn interpolate(&self, triangle: [usize; 3], site: &Site2D) -> [f64; 3] {
        let (a, b, c) = (triangle[0], triangle[1], triangle[2]);
        let sites = [self.sites[a], self.sites[b], self.sites[c]];
        let area = |a: &Site2D, b: &Site2D, c: &Site2D| {
            (a.x - c.x) * (b.y - c.y) - (a.y - c.y) * (b.x - c.x)
        };
        let s = area(&sites[0], &sites[1], &sites[2]);
        let s1 = area(site, &sites[1], &sites[2]);
        let s2 = area(&sites[0], site, &sites[2]);
        let s3 = area(&sites[0], &sites[1], site);
        let w1 = s1 / s;
        let w2 = s2 / s;
        let w3 = s3 / s;
        [w1, w2, w3]
    }
}
