use rand::Rng;
use terrain::generator::TerrainGenerator;
use terrain::model2d::{builder::TerrainModel2DBulider, sites::Site2D};
extern crate terrain;

fn main() {
    // Number of sites
    let num = 10000;

    // Bounding box to generate random sites and render terrain data to image
    let bound_min = Site2D { x: 0.0, y: 0.0 };
    let bound_max = Site2D {
        x: 200.0 * 1e3, // 200 km
        y: 200.0 * 1e3, // 200 km
    };

    // Generate random sites
    let mut rng = rand::thread_rng();
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
        .unwrap()
        .iterate_sites(1)
        .unwrap()
        .build()
        .unwrap();

    // `TerrainGenerator` generates a terrain from `TerrainModel`.
    // `TerrainGenerator` requires some paramaters to simulate landscape evolution for each site.
    let terrain = TerrainGenerator::default()
        .set_model(model)
        .set_uplift_rate(1e-4 * 5.0)
        .set_erodibility(1e-7 * 5.61)
        .set_max_slope(3.14 * 0.1) // radian
        .set_exponent_m(0.5)
        .generate()
        .unwrap();

    // Render to image.
    // In this example, the terrain is represented by small rectangles, resulting in many voids between the rectangles.
    // The color of the rectangle is determined by the altitude of the site.
    let img_width = 500;
    let img_height = 500;
    let rect_width = 5;

    let mut image_buf = image::RgbImage::new(img_width, img_height);
    let max_altitude = terrain
        .altitudes
        .iter()
        .fold(std::f64::MIN, |acc, &n| n.max(acc));

    terrain.sites.iter().enumerate().for_each(|(i, n)| {
        let x = img_width as f64 * (n.x / bound_max.x);
        let y = img_height as f64 * (n.y / bound_max.y);
        let color = (terrain.altitudes[i] / (max_altitude) * 255.0) as u8;
        for dx in 0..rect_width {
            for dy in 0..rect_width {
                let x = x as i32 + dx;
                let y = y as i32 + dy;
                if x < 0 || x >= img_width as i32 || y < 0 || y >= img_height as i32 {
                    continue;
                }

                image_buf.put_pixel(x as u32, y as u32, image::Rgb([color, color, color]));
            }
        }
    });

    image_buf.save("image.png").unwrap();
}
