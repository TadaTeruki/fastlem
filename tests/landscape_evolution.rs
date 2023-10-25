use rand::Rng;
use terrain::core::attributes::TerrainAttributes;
use terrain::lem::generator::TerrainGenerator;
use terrain::models::surface::{builder::TerrainModel2DBulider, sites::Site2D};
extern crate terrain;

#[test]
fn test_landscape_evolution() {
    let num = 10000;
    let bound_min = Site2D { x: 0.0, y: 0.0 };
    let bound_max = Site2D {
        x: 200.0 * 1e3, // 200 km
        y: 100.0 * 1e3, // 100 km
    };

    let mut sites = Vec::with_capacity(num);
    let mut rng = rand::thread_rng();

    for _ in 0..num {
        let x = rng.gen_range(bound_min.x..bound_max.x);
        let y = rng.gen_range(bound_min.y..bound_max.y);
        sites.push(Site2D { x, y });
    }

    let model = TerrainModel2DBulider::default()
        .set_sites(sites)
        .set_bounding_box(Some(bound_min), Some(bound_max))
        .unwrap()
        .iterate_sites(1)
        .unwrap()
        .build()
        .unwrap();

    let terrain = TerrainGenerator::default()
        .set_model(model)
        .set_attributes(
            (0..num)
                .map(|_| TerrainAttributes::new(1e-4 * 5.0, 1e-7 * 5.61, 0.0, Some(3.14 * 0.1)))
                .collect::<_>(),
        )
        .set_exponent_m(0.5)
        .generate()
        .unwrap();

    let sites = terrain.sites();
    let altitudes = terrain.altitudes();

    let image = terrain_visualizer::Visualizer::new(
        sites
            .iter()
            .enumerate()
            .map(|(i, n)| (terrain_visualizer::Site { x: n.x, y: n.y }, altitudes[i]))
            .collect::<Vec<(terrain_visualizer::Site, f64)>>(),
    )
    .set_x_range(bound_min.x, bound_max.x)
    .set_y_range(bound_min.y, bound_max.y);

    image
        .render_image(Some(1000), None, |weight_rate: f64| {
            let c = (weight_rate * 220.0 + 30.0) as u8;
            image::Rgb([c, c, c])
        })
        .unwrap()
        .save("image.png")
        .unwrap();
}
