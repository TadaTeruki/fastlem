use fastlem::core::{parameters::TopographicalParameters, traits::Model};
use fastlem::lem::generator::TerrainGenerator;
use fastlem::models::surface::{builder::TerrainModel2DBulider, sites::Site2D};
use noise::{NoiseFn, Perlin};
use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;

extern crate fastlem;

// An example for generating more complex (and wonderful) terrains.
// The result is shown in `images/out/terrain_generation_advanced.png`.

fn main() {
    // The bound_min and bound_max are the minimum and maximum coordinates of the terrain.
    // You can expand the terrain by changing these values.
    let bound_min = Site2D { x: 0.0, y: 0.0 };
    let bound_max = Site2D { x: 100.0, y: 100.0 };

    // Seed of the noise generator.
    // You can generate various terrains by changing the seed.
    let seed = 0;

    // Noise generator
    let perlin = Perlin::new(seed);

    // The value of N per 1.0x1.0.
    // N will be (bound_max.x - bound_min.x) * (bound_max.y - bound_min.y) * num_per_square
    let num_per_square = 8.0;

    let num = ((bound_max.x - bound_min.x) * (bound_max.y - bound_min.y) * num_per_square) as usize;
    let bound_range = Site2D {
        x: bound_max.x - bound_min.x,
        y: bound_max.y - bound_min.y,
    };

    println!("creating a model...");

    let model = TerrainModel2DBulider::from_random_sites(num, bound_min, bound_max)
        .relaxate_sites(1)
        .unwrap()
        .add_edge_sites(None, None)
        .unwrap()
        .build()
        .unwrap();

    println!("distributing params...");

    let sites = model.sites().to_vec();

    // fault
    let max_fault_radius = 35.0;
    let get_fault = |site: &Site2D| -> (f64, f64) {
        let scale = 100.0;

        let modulus = octaved_perlin(&perlin, site.x / scale, site.y / scale, 3, 0.5, 2.0).abs()
            * 2.0
            * max_fault_radius;
        let direction_x = octaved_perlin(
            &perlin,
            (site.x + bound_range.x) / scale,
            (site.y + bound_range.y) / scale,
            4,
            0.6,
            2.2,
        ) * 2.0;
        let direction_y = octaved_perlin(
            &perlin,
            (site.x - bound_range.x) / scale,
            (site.y - bound_range.y) / scale,
            4,
            0.6,
            2.2,
        ) * 2.0;
        (direction_x * modulus, direction_y * modulus)
    };

    // reef
    let max_reef_radius = 5.0;
    let get_reef = |site: &Site2D| -> (f64, f64) {
        let scale_scale = 800.0;
        let min_scale = 7.5;
        let max_scale = 20.0;
        let scale = octaved_perlin(
            &perlin,
            site.x / scale_scale,
            site.y / scale_scale,
            2,
            0.5,
            2.0,
        )
        .abs()
            * 2.0
            * (max_scale - min_scale)
            + min_scale;
        let modulus_rt =
            octaved_perlin(&perlin, site.x / scale, site.y / scale, 3, 0.5, 2.0).abs() * 2.0;
        let modulus = modulus_rt.powf(3.5) * max_reef_radius;
        let direction_x = octaved_perlin(
            &perlin,
            (site.x + bound_range.x) / scale,
            (site.y + bound_range.y) / scale,
            4,
            0.6,
            2.2,
        );
        let direction_y = octaved_perlin(
            &perlin,
            (site.x - bound_range.x) / scale,
            (site.y - bound_range.y) / scale,
            4,
            0.6,
            2.2,
        );

        (direction_x * modulus, direction_y * modulus)
    };
    let apply_fault = |site: &Site2D| -> Site2D {
        let fault = get_fault(site);
        let fault_x = site.x + fault.0;
        let fault_y = site.y + fault.1;
        Site2D {
            x: fault_x,
            y: fault_y,
        }
    };

    let apply_reef = |site: &Site2D| -> Site2D {
        let reef = get_reef(site);
        let reef_x = site.x + reef.0;
        let reef_y = site.y + reef.1;
        Site2D {
            x: reef_x,
            y: reef_y,
        }
    };

    let base_is_outlet = {
        sites
            .iter()
            .map(|site| {
                let site = apply_reef(&apply_fault(site));
                let persistence_scale = 50.;

                let plate_scale = 50.;
                let noise_persistence = octaved_perlin(
                    &perlin,
                    site.x / persistence_scale,
                    site.y / persistence_scale,
                    2,
                    0.5,
                    2.0,
                )
                .abs()
                    * 0.7
                    + 0.3;
                let noise_plate = octaved_perlin(
                    &perlin,
                    site.x / plate_scale,
                    site.y / plate_scale,
                    8,
                    noise_persistence,
                    2.4,
                ) * 0.5
                    + 0.5;
                let continent_scale = 200.;
                let noise_continent = octaved_perlin(
                    &perlin,
                    site.x / continent_scale,
                    site.y / continent_scale,
                    3,
                    0.5,
                    1.8,
                ) * 0.7
                    + 0.5;
                let ocean_bias = 0.035;
                noise_plate > noise_continent - ocean_bias
            })
            .collect::<Vec<bool>>()
    };

    let start_index = (num + 1..sites.len()).collect::<Vec<_>>();
    let graph = model.graph();

    let is_outlet = determine_outlets(&sites, base_is_outlet, start_index, graph).unwrap();

    println!("generating...");

    let parameters = {
        sites
            .iter()
            .enumerate()
            .map(|(i, site)| {
                let site = apply_reef(&apply_fault(site));
                let erodibility_scale = 75.0;
                let noise_erodibility = octaved_perlin(
                    &perlin,
                    site.x / erodibility_scale,
                    site.y / erodibility_scale,
                    5,
                    0.7,
                    2.2,
                )
                .abs()
                    * 4.0
                    + 0.1;
                TopographicalParameters::default()
                    .set_erodibility(noise_erodibility)
                    .set_is_outlet(is_outlet[i])
            })
            .collect::<Vec<TopographicalParameters>>()
    };

    let terrain = TerrainGenerator::default()
        .set_model(model)
        .set_parameters(parameters)
        .generate()
        .unwrap();

    println!("rendering...");

    let colormap_warm: (Vec<[u8; 3]>, Vec<f64>) = (
        vec![
            [50, 110, 150],
            [200, 200, 180],
            [100, 150, 70],
            [90, 120, 65],
            [210, 210, 210],
        ],
        vec![0.0, 0.2, 0.6, 25.0, 35.0],
    );

    let colormap_dry: (Vec<[u8; 3]>, Vec<f64>) = (
        vec![
            [50, 110, 150],
            [200, 200, 180],
            [200, 190, 140],
            [140, 110, 80],
            [60, 60, 30],
            [210, 210, 210],
        ],
        vec![0.0, 0.2, 0.6, 10.0, 25.0, 35.0],
    );
    let colormap_table = (vec![colormap_warm, colormap_dry], vec![0.35, 0.55]);

    // get color from elevation
    let get_color = |site: Site2D, elevation: f64| -> [u8; 3] {
        let climate_scale = 75.0;
        let site = apply_fault(&site);
        let noise_climate = octaved_perlin(
            &perlin,
            site.x / climate_scale,
            site.y / climate_scale,
            7,
            0.6,
            2.4,
        ) + 0.5;
        let (colormap_a, colormap_b) = get_surounding_index(&colormap_table.1, noise_climate);
        if colormap_a == colormap_b {
            return get_interporated_color(
                &colormap_table.0[colormap_a].0,
                &colormap_table.0[colormap_a].1,
                elevation,
            );
        } else {
            let (color_a, color_b) = (
                get_interporated_color(
                    &colormap_table.0[colormap_a].0,
                    &colormap_table.0[colormap_a].1,
                    elevation,
                ),
                get_interporated_color(
                    &colormap_table.0[colormap_b].0,
                    &colormap_table.0[colormap_b].1,
                    elevation,
                ),
            );
            let prop = (noise_climate - colormap_table.1[colormap_a])
                / (colormap_table.1[colormap_b] - colormap_table.1[colormap_a]);
            lerp_color(color_a, color_b, prop)
        }
    };

    // resolution of the image
    let img_width = 500;
    let img_height = (img_width as f64 * bound_range.y / bound_range.x) as u32;

    let shadow_dist = 0.3;
    let shadow_angle: f64 = 3.14 * 0.25;
    let shadow_dist_x = shadow_dist * shadow_angle.cos();
    let shadow_dist_y = shadow_dist * shadow_angle.sin();
    let shadow_elevation = 50.0;

    let mut image_buf = image::RgbImage::new(img_width, img_height);

    for imgx in 0..img_width {
        for imgy in 0..img_height {
            let x = (bound_range.x - shadow_dist_x) * ((imgx as f64 + 0.5) / img_width as f64)
                + bound_min.x;
            let y = (bound_range.y - shadow_dist_y) * ((imgy as f64 + 0.5) / img_height as f64)
                + bound_min.y;
            let site = Site2D { x, y };
            let site2 = Site2D {
                x: x + shadow_dist_x,
                y: y + shadow_dist_y,
            };
            let elevation = terrain.get_elevation(&site);
            let elevation2 = terrain.get_elevation(&site2);

            if let (Some(elevation), Some(elevation2)) = (elevation, elevation2) {
                let brightness = 1.0 - ((elevation - elevation2) / shadow_elevation).atan().sin();

                let color = apply_brightness(get_color(site, elevation), brightness);
                image_buf.put_pixel(imgx as u32, imgy as u32, image::Rgb(color));
            }
        }
    }

    image_buf.save("image.png").unwrap();
}

fn octaved_perlin(
    perlin: &Perlin,
    x: f64,
    y: f64,
    octaves: usize,
    persistence: f64,
    lacunarity: f64,
) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += perlin.get([x * frequency, y * frequency, 0.0]) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    value / max_value
}

fn determine_outlets(
    sites: &Vec<Site2D>,
    base_is_outlet: Vec<bool>,
    start_index: Vec<usize>,
    graph: &EdgeAttributedUndirectedGraph<f64>,
) -> Option<Vec<bool>> {
    let mut queue = start_index
        .into_iter()
        .filter(|i| base_is_outlet[*i])
        .collect::<Vec<_>>();
    if queue.is_empty() {
        return None;
    }
    let mut outlets = vec![false; sites.len()];
    while let Some(i) = queue.pop() {
        if outlets[i] {
            continue;
        }
        outlets[i] = true;
        graph.neighbors_of(i).iter().for_each(|(j, _)| {
            if !outlets[*j] && base_is_outlet[*j] {
                queue.push(*j);
            }
        });
    }

    let is_outlet = outlets.iter().map(|&b| b).collect::<Vec<_>>();
    Some(is_outlet)
}

fn lerp_color(c1: [u8; 3], c2: [u8; 3], prop: f64) -> [u8; 3] {
    [
        (c1[0] as f64 + (c2[0] as f64 - c1[0] as f64) * prop) as u8,
        (c1[1] as f64 + (c2[1] as f64 - c1[1] as f64) * prop) as u8,
        (c1[2] as f64 + (c2[2] as f64 - c1[2] as f64) * prop) as u8,
    ]
}

fn apply_brightness(color: [u8; 3], brightness: f64) -> [u8; 3] {
    [
        (color[0] as f64 * brightness) as u8,
        (color[1] as f64 * brightness) as u8,
        (color[2] as f64 * brightness) as u8,
    ]
}

fn get_surounding_index(props: &Vec<f64>, target: f64) -> (usize, usize) {
    if target <= props[0] {
        (0, 0)
    } else if target >= props[props.len() - 1] {
        (props.len() - 1, props.len() - 1)
    } else {
        let mut index = 1;
        for i in 1..props.len() {
            if target < props[i] {
                index = i;
                break;
            }
        }
        (index - 1, index)
    }
}

fn get_interporated_color(colors: &Vec<[u8; 3]>, elevations: &Vec<f64>, elevation: f64) -> [u8; 3] {
    let (index_a, index_b) = get_surounding_index(elevations, elevation);
    let color_a = colors[index_a];
    let color_b = colors[index_b];
    let prop = (elevation - elevations[index_a]) / (elevations[index_b] - elevations[index_a]);
    lerp_color(color_a, color_b, prop.min(1.0).max(0.0))
}
