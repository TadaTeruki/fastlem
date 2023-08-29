use rand::Rng;
extern crate lem;

#[test]
fn test_landscape_evolution() {
    let num = 50000;
    let bound_min = lem::units::Site { x: 0.0, y: 0.0 };
    let bound_max = lem::units::Site {
        x: 2000.0 * 1e3, // 2000 km
        y: 2000.0 * 1e3, // 2000 km
    };

    let mut sites = Vec::with_capacity(num);
    let mut rng = rand::thread_rng();

    for _ in 0..num {
        let x = rng.gen_range(bound_min.x..bound_max.x);
        let y = rng.gen_range(bound_min.y..bound_max.y);
        sites.push(lem::units::Site { x, y });
    }

    let model = lem::model::TerrainModel::default()
        .set_sites(sites)
        .set_bounding_box(Some(bound_min), Some(bound_max))
        .unwrap()
        .iterate_sites(1)
        .unwrap();

    let terrain = lem::generator::TerrainGenerator::default()
        .set_model(model)
        .set_uplift_rate(1e-4 * 5.0)
        .set_erodibility(1e-7 * 5.61)
        .set_max_slope(3.14 * 0.03) // radian
        .set_exponent_m(0.5)
        .generate()
        .unwrap();

    let sites = terrain.sites;
    let altitudes = terrain.altitudes;

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
