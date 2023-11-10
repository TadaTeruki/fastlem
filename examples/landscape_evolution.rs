use fastlem::core::attributes::TerrainAttributes;
use fastlem::lem::generator::TerrainGenerator;
use fastlem::models::surface::{builder::TerrainModel2DBulider, sites::Site2D};
use rand::{Rng, SeedableRng};
extern crate fastlem;

fn main() {
    // Number of sites
    let num = 30000;

    // Bounding box to generate random sites and render terrain data to image
    let bound_min = Site2D { x: 0.0, y: 0.0 };
    let bound_max = Site2D {
        x: 200.0, // 200 km
        y: 200.0, // 200 km
    };

    // Generate random sites
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);

    let sites = (0..num)
        .map(|_| {
            let x = rng.gen_range(bound_min.x..bound_max.x);
            let y = rng.gen_range(bound_min.y..bound_max.y);
            Site2D { x, y }
        })
        .collect::<Vec<Site2D>>();

    // `TerrainModel` is a set of fundamental data required for genreating terrain.
    // This includes a set of sites and graph (created by delaunay triangulation).
    // When `build` method is called, the model is validated and the graph is constructed.
    // When `iterate_sites` method is called (after `set_bounding_box` method was called), the sites are relocated to apploximately evenly spaced positions using Lloyd's algorithm.
    let model = TerrainModel2DBulider::default()
        .set_sites(sites)
        .set_bounding_box(Some(bound_min), Some(bound_max))
        .iterate_sites(1)
        .unwrap()
        .build()
        .unwrap();

    // `TerrainGenerator` generates a terrain from `TerrainModel`.
    // `TerrainGenerator` requires some paramaters to simulate landscape evolution for each site.
    let terrain = TerrainGenerator::default()
        .set_model(model)
        .set_attributes(
            (0..num)
                .map(|_| TerrainAttributes::new(0.0, 1., 1., None))
                .collect::<_>(),
        )
        .generate()
        .unwrap();

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
                let color = ((altitude / max_altitude) * 255.0) as u8;
                image_buf.put_pixel(imgx as u32, imgy as u32, image::Rgb([color, color, color]));
            }
        }
    }

    image_buf.save("image.png").unwrap();
}
