use fastlem::core::traits::Model;
use fastlem::models::surface::{builder::TerrainModel2DBulider, sites::Site2D};
use rand::Rng;
mod voronoi_visualizer;
use voronoi_visualizer::voronoi_visualizer::Visualizer;
extern crate fastlem;

#[test]
fn test_random_delaunay() {
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

    let sites = model.sites();

    let image = Visualizer::new(
        sites
            .iter()
            .map(|n| (Site2D { x: n.x, y: n.y }, rng.gen_range(0.0..1.0)))
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
