use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use procedural_terrain::core::attributes::TerrainAttributes;
use procedural_terrain::lem::generator::TerrainGenerator;
use procedural_terrain::models::surface::{builder::TerrainModel2DBulider, sites::Site2D};
extern crate procedural_terrain;

fn main() {
    // Bounding box to generate random sites and render terrain data to image
    let bound_min = Site2D { x: 0.0, y: 0.0 };
    let bound_max = Site2D {
        x: 200.0 * 1e3, // 200 km
        y: 200.0 * 1e3, // 200 km
    };

    let interval = 1.0 * 1e3;

    let x_size = (bound_max.x / interval) as usize;
    let y_size = (bound_max.y / interval) as usize;

    let sites = {
        let mut sites = Vec::new();
        for y in 0..y_size {
            for x in 0..x_size {
                sites.push(Site2D {
                    x: x as f64 * interval,
                    y: y as f64 * interval,
                });
            }
        }
        sites
    };

    let fbm = Fbm::<Perlin>::default().set_octaves(8);

    let num = sites.len();

    let base_level = sites
        .iter()
        .map(|site| {
            let value = fbm.get([site.x / bound_max.x, site.y / bound_max.y, 0.0]);
            value
        })
        .collect::<Vec<f64>>();

    let sea_level = 0.4;
    //let deposition_level = -0.0;

    let deposition_rate = sites
        .iter()
        .map(|site| {
            let value = fbm.get([site.x / bound_max.x * 3.0, site.y / bound_max.y * 3.0, 0.0]);
            value
        })
        .collect::<Vec<f64>>();

    // `TerrainModel` is a set of fundamental data required for genreating terrain.
    // This includes a set of sites and graph (created by delaunay triangulation).
    // When `build` method is called, the model is validated and the graph is constructed.
    // When `iterate_sites` method is called (after `set_bounding_box` method was called), the sites are relocated to apploximately evenly spaced positions using Lloyd's algorithm.
    let model = TerrainModel2DBulider::default()
        .set_sites(sites)
        .set_bounding_box(Some(bound_min), Some(bound_max))
        .set_custom_outlets(
            base_level
                .iter()
                .enumerate()
                .filter(|(_, &v)| v <= sea_level)
                .map(|(i, _)| i)
                .collect::<Vec<usize>>(),
        )
        .build()
        .unwrap();

    // `TerrainGenerator` generates a terrain from `TerrainModel`.
    // `TerrainGenerator` requires some paramaters to simulate landscape evolution for each site.
    let terrain = TerrainGenerator::default()
        .set_model(model)
        .set_attributes(
            (0..num)
                .map(|i| {
                    let erodibility = { deposition_rate[i] + 1.0 };
                    TerrainAttributes::new(0.0, 0.8, erodibility, None)
                })
                .collect::<_>(),
        )
        .generate()
        .unwrap();

    println!("Terrain was successfully generated.");

    // Render to image.
    // In this example, the terrain is represented by small rectangles, resulting in many voids between the rectangles.
    // The color of the rectangle is determined by the altitude of the site.
    let img_width = 500;
    let img_height = 500;

    let mut image_buf = image::RgbImage::new(img_width, img_height);
    let max_altitude = terrain
        .altitudes()
        .iter()
        .fold(std::f64::MIN, |acc, &n| n.max(acc));

    for imgx in 0..img_width {
        for imgy in 0..img_height {
            let x = bound_max.x * (imgx as f64 / img_width as f64);
            let y = bound_max.y * (imgy as f64 / img_height as f64);
            let site = Site2D { x, y };
            let altitude = terrain.get_altitude(&site);
            if let Some(altitude) = altitude {
                if altitude <= 0.0 {
                    image_buf.put_pixel(imgx as u32, imgy as u32, image::Rgb([0, 100, 150]));
                    continue;
                }
                let color = ((altitude / max_altitude) * 255.0) as u8;
                image_buf.put_pixel(imgx as u32, imgy as u32, image::Rgb([color, color, color]));
            }
        }
    }

    image_buf.save("image.png").unwrap();
}
