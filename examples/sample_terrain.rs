use fastlem::core::{attributes::TerrainAttributes, traits::Model};
use fastlem::lem::generator::TerrainGenerator;
use fastlem::models::surface::{builder::TerrainModel2DBulider, sites::Site2D};
use naturalneighbor::{Interpolator, Lerpable, Point};
use serde::Deserialize;
extern crate fastlem;

#[derive(Debug, Deserialize, Clone)]
struct SampleNode {
    x: f64,
    y: f64,
    erodibility: f64,
    is_outlet: bool,
}

impl Into<Point> for SampleNode {
    fn into(self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }
}

impl Lerpable for SampleNode {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        Self {
            x: self.x.lerp(&other.x, t),
            y: self.y.lerp(&other.y, t),
            erodibility: self.erodibility.lerp(&other.erodibility, t),
            is_outlet: {
                if t < 0.5 {
                    self.is_outlet
                } else {
                    other.is_outlet
                }
            },
        }
    }
}

fn main() {
    let num = 30000;

    let bound_min = Site2D { x: 0.0, y: 0.0 };
    let bound_max = Site2D { x: 500.0, y: 500.0 };

    let model = TerrainModel2DBulider::from_random_sites(num, bound_min, bound_max)
        .relaxate_sites(1)
        .unwrap()
        .build()
        .unwrap();

    let seed_nodes = serde_json::from_str::<Vec<SampleNode>>(
        &std::fs::read_to_string("examples/data/sample.json").unwrap(),
    )
    .unwrap();

    let edge_nodes_num = 5;

    let corners = vec![
        Point {
            x: bound_min.x,
            y: bound_min.y,
        },
        Point {
            x: bound_min.x,
            y: bound_max.y,
        },
        Point {
            x: bound_max.x,
            y: bound_max.y,
        },
        Point {
            x: bound_max.x,
            y: bound_min.y,
        },
    ];

    let nearest_node = |point: &Point| -> SampleNode {
        let mut min_dist = std::f64::MAX;
        let mut nearest_node = None;
        for node in &seed_nodes {
            let dist = (point.x - node.x).powi(2) + (point.y - node.y).powi(2);
            if dist < min_dist {
                min_dist = dist;
                nearest_node = Some(node);
            }
        }
        nearest_node.unwrap().clone()
    };

    let edge_nodes = corners
        .iter()
        .enumerate()
        .flat_map(|(i, corner)| {
            let next = &corners[(i + 1) % corners.len()];
            let mut nodes = vec![];
            for j in 0..edge_nodes_num {
                let t = j as f64 / edge_nodes_num as f64;
                let point = Point {
                    x: corner.x.lerp(&next.x, t),
                    y: corner.y.lerp(&next.y, t),
                };
                let nearest_node = nearest_node(&point);
                nodes.push(SampleNode {
                    x: point.x,
                    y: point.y,
                    erodibility: nearest_node.erodibility,
                    is_outlet: nearest_node.is_outlet,
                });
            }
            nodes
        })
        .collect::<Vec<_>>();

    let seed_nodes = seed_nodes.into_iter().chain(edge_nodes).collect::<Vec<_>>();

    let sites = model.sites().to_vec();
    let interpolator = Interpolator::new(&seed_nodes);

    let terrain = TerrainGenerator::default()
        .set_model(model)
        .set_attributes(
            (0..num)
                .map(|i| {
                    let point = Point {
                        x: sites[i].x,
                        y: sites[i].y,
                    };
                    let sample = interpolator.interpolate(&seed_nodes, point).unwrap();
                    TerrainAttributes::default()
                        .set_erodibility(sample.erodibility)
                        .set_is_outlet(sample.is_outlet)
                })
                .collect::<_>(),
        )
        .generate()
        .unwrap();

    let color_table: Vec<([u8; 3], f64)> = vec![
        ([70, 150, 200], 0.0),
        ([240, 240, 210], 0.05),
        ([190, 200, 120], 0.75),
        ([25, 100, 25], 18.0),
        ([15, 60, 15], 30.0),
    ];

    let img_width = 500;
    let img_height = 500;

    let mut image_buf = image::RgbImage::new(img_width, img_height);

    for imgx in 0..img_width {
        for imgy in 0..img_height {
            let x = (bound_max.x - bound_min.x) * (imgx as f64 / img_width as f64) + bound_min.x;
            let y = (bound_max.y - bound_min.y) * (imgy as f64 / img_height as f64) + bound_min.y;
            let site = Site2D { x, y };
            let altitude = terrain.get_altitude(&site);
            if let Some(altitude) = altitude {
                let prop = altitude;
                let color = {
                    let color_index = {
                        let mut i = 0;
                        while i < color_table.len() {
                            if prop < color_table[i].1 {
                                break;
                            }
                            i += 1;
                        }
                        i
                    };

                    if color_index == 0 {
                        color_table[0].0
                    } else if color_index == color_table.len() {
                        color_table[color_table.len() - 1].0
                    } else {
                        let color_a = color_table[color_index - 1];
                        let color_b = color_table[color_index];

                        let prop_a = color_a.1;
                        let prop_b = color_b.1;

                        let prop = (prop - prop_a) / (prop_b - prop_a);

                        let color = [
                            (color_a.0[0] as f64
                                + (color_b.0[0] as f64 - color_a.0[0] as f64) * prop)
                                as u8,
                            (color_a.0[1] as f64
                                + (color_b.0[1] as f64 - color_a.0[1] as f64) * prop)
                                as u8,
                            (color_a.0[2] as f64
                                + (color_b.0[2] as f64 - color_a.0[2] as f64) * prop)
                                as u8,
                        ];

                        color
                    }
                };

                image_buf.put_pixel(imgx as u32, imgy as u32, image::Rgb(color));
            }
        }
    }

    image_buf.save("image.png").unwrap();
}
