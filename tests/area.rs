use fastlem::core::traits::Model;
use fastlem::models::surface::{builder::TerrainModel2DBulider, sites::Site2D};
mod voronoi_visualizer;
use voronoi_visualizer::voronoi_visualizer::Visualizer;
extern crate fastlem;

#[test]
fn test_terrain_generation() {
    let num = 10000;
    let bound_min = Site2D { x: 0.0, y: 0.0 };
    let bound_max = Site2D { x: 200.0, y: 100.0 };

    let model = TerrainModel2DBulider::from_random_sites(num, bound_min, bound_max)
        .relaxate_sites(1)
        .unwrap()
        .build()
        .unwrap();

    let sites = model.sites();
    let areas = model.areas();

    let image = Visualizer::new(
        sites
            .iter()
            .enumerate()
            .map(|(i, n)| (Site2D { x: n.x, y: n.y }, areas[i]))
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
