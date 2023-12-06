use fastlem::core::parameters::TopographicalParameters;
use fastlem::lem::generator::TerrainGenerator;
use fastlem::models::surface::{builder::TerrainModel2DBulider, sites::Site2D};
use rand::Rng;
mod voronoi_visualizer;
use voronoi_visualizer::voronoi_visualizer::Visualizer;
extern crate fastlem;

#[test]
fn test_landscape_evolution() {
    let num = 10000;
    let bound_min = Site2D { x: 0.0, y: 0.0 };
    let bound_max = Site2D { x: 200.0, y: 100.0 };

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
        .relaxate_sites(1)
        .unwrap()
        .build()
        .unwrap();

    let terrain = TerrainGenerator::default()
        .set_model(model)
        .set_parameters(
            (0..num)
                .map(|_| {
                    TopographicalParameters::default()
                        .set_base_altitude(0.0)
                        .set_erodibility(1.0)
                        .set_uplift_rate(1.0)
                        .set_is_outlet(false)
                        .set_max_slope(Some(3.14 * 0.1))
                })
                .collect::<_>(),
        )
        .generate()
        .unwrap();

    let sites = terrain.sites();
    let altitudes = terrain.altitudes();

    let image = Visualizer::new(
        sites
            .iter()
            .enumerate()
            .map(|(i, n)| (Site2D { x: n.x, y: n.y }, altitudes[i]))
            .collect::<Vec<(Site2D, f64)>>(),
    )
    .set_x_range(bound_min.x, bound_max.x)
    .set_y_range(bound_min.y, bound_max.y);

    image
        .render_image(Some(500), None)
        .unwrap()
        .save("image.png")
        .unwrap();
}
