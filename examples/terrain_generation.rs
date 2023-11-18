use fastlem::core::{attributes::TerrainAttributes, traits::Model};
use fastlem::lem::generator::TerrainGenerator;
use fastlem::models::surface::{builder::TerrainModel2DBulider, sites::Site2D};
use noise::{NoiseFn, Perlin};

extern crate fastlem;

fn octaved_perlin(perlin: &Perlin, x: f64, y: f64, octaves: usize) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += perlin.get([x * frequency, y * frequency, 0.0]) * amplitude;
        max_value += amplitude;
        amplitude /= 2.0;
        frequency *= 2.0;
    }

    value / max_value
}

fn main() {
    let num = 100000;

    let bound_min = Site2D { x: 0.0, y: 0.0 };
    let bound_max = Site2D { x: 100.0, y: 100.0 };

    let model = TerrainModel2DBulider::from_random_sites(num, bound_min, bound_max)
        .relaxate_sites(1)
        .unwrap()
        .add_edge_sites(None)
        .unwrap()
        .build()
        .unwrap();

    let sites = model.sites().to_vec();

    let perlin = Perlin::new(100);

    let terrain = TerrainGenerator::default()
        .set_model(model)
        .set_attributes(
            (0..sites.len())
                .map(|i| {
                    let site = sites[i];
                    let octaves = 10;
                    let x = site.x / bound_max.x;
                    let y = site.y / bound_max.y;
                    let noise_erodibility =
                        octaved_perlin(&perlin, x * 5.0, y * 5.0, octaves) * 0.5 + 0.5;
                    let noise_is_outlet =
                        octaved_perlin(&perlin, x * 2.0, y * 2.0, octaves) * 0.5 + 0.5;
                    TerrainAttributes::default()
                        .set_erodibility(noise_erodibility)
                        .set_is_outlet(noise_is_outlet > 0.48)
                })
                .collect::<_>(),
        )
        .generate()
        .unwrap();

    // (color: [u8; 3], altitude: f64)
    let color_table: Vec<([u8; 3], f64)> = vec![
        ([70, 150, 200], 0.0),
        ([240, 240, 210], 0.5),
        ([190, 200, 120], 1.0),
        ([25, 100, 25], 18.0),
        ([15, 60, 15], 30.0),
    ];

    // get color from altitude
    let get_color = |altitude: f64| -> [u8; 3] {
        let color_index = {
            let mut i = 0;
            while i < color_table.len() {
                if altitude < color_table[i].1 {
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

            let prop = (altitude - prop_a) / (prop_b - prop_a);

            let color = [
                (color_a.0[0] as f64 + (color_b.0[0] as f64 - color_a.0[0] as f64) * prop) as u8,
                (color_a.0[1] as f64 + (color_b.0[1] as f64 - color_a.0[1] as f64) * prop) as u8,
                (color_a.0[2] as f64 + (color_b.0[2] as f64 - color_a.0[2] as f64) * prop) as u8,
            ];

            color
        }
    };

    let img_width = 500;
    let img_height = 500;

    let mut image_buf = image::RgbImage::new(img_width, img_height);

    for imgx in 0..img_width {
        for imgy in 0..img_height {
            let x = bound_max.x * (imgx as f64 / img_width as f64);
            let y = bound_max.y * (imgy as f64 / img_height as f64);
            let site = Site2D { x, y };
            let altitude = terrain.get_altitude(&site);
            if let Some(altitude) = altitude {
                let color = get_color(altitude);
                image_buf.put_pixel(imgx as u32, imgy as u32, image::Rgb(color));
            }
        }
    }

    image_buf.save("image.png").unwrap();
}
