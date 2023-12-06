use std::{collections::BinaryHeap, error, io};

use fastlem::models::surface::sites::Site2D;

/// A struct to provide visualization of the terrain data.
pub struct Visualizer {
    x_range: Option<(f64, f64)>,
    y_range: Option<(f64, f64)>,
    weight_range: Option<(f64, f64)>,
    nodes: Vec<(Site2D, f64)>,
}

/// A pixel in image.
///
/// The pixel is eventually colored according to the weight of the node.
/// For searching the nearest node from the pixel, `VisualizerPixel` holds the negative squared distance
/// between the pixel and the root pixel as `negative_squared_distance`.
#[derive(Debug, PartialEq)]
struct VisualizerPixel {
    // coordinates of the pixel
    x: u32,
    y: u32,

    // index of the root node
    root_node_i: usize,

    // negative squared distance between the pixel and the root pixel
    negative_squared_distance: f64,
}

// `Eq` is implemented for `BinaryHeap` to work.
impl Eq for VisualizerPixel {}

// `Ord` is implemented for `BinaryHeap` to work.
impl Ord for VisualizerPixel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.negative_squared_distance
            .partial_cmp(&other.negative_squared_distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

// `PartialOrd` is implemented for `BinaryHeap` to work.
impl PartialOrd for VisualizerPixel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Visualizer {
    /// Creates a new `Visualizer`.
    pub fn new(nodes: Vec<(Site2D, f64)>) -> Visualizer {
        Visualizer {
            x_range: None,
            y_range: None,
            weight_range: None,
            nodes,
        }
    }

    /// Sets the range of x coordinates.
    pub fn set_x_range(mut self, x_min: f64, x_max: f64) -> Self {
        self.x_range = Some((x_min, x_max));
        self
    }

    /// Sets the range of y coordinates.
    pub fn set_y_range(mut self, y_min: f64, y_max: f64) -> Self {
        self.y_range = Some((y_min, y_max));
        self
    }

    /// Render terrain data into image buffer.
    ///
    /// `width` and `height` are the size of the image.
    ///  - If both width and height are specified, the image will be expanded to fit the width and height.
    ///  - If only width is specified, the image will be scaled to fit the width keeping the aspect ratio.
    ///    (This is also the same for height.)
    ///  - If both width and height are None, an error will be returned.
    pub fn render_image(
        &self,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<image::RgbImage, Box<impl error::Error>> {
        let (min_x, max_x) = {
            if let Some(x_range) = self.x_range {
                x_range
            } else {
                // calculate x range from nodes
                let mut min_x = std::f64::MAX;
                let mut max_x = std::f64::MIN;
                for node in self.nodes.iter() {
                    let x = node.0.x;
                    if x < min_x {
                        min_x = x;
                    }
                    if x > max_x {
                        max_x = x;
                    }
                }
                (min_x, max_x)
            }
        };

        let (min_y, max_y) = {
            if let Some(y_range) = self.y_range {
                y_range
            } else {
                // calculate y range from nodes
                let mut min_y = std::f64::MAX;
                let mut max_y = std::f64::MIN;
                for node in self.nodes.iter() {
                    let y = node.0.y;
                    if y < min_y {
                        min_y = y;
                    }
                    if y > max_y {
                        max_y = y;
                    }
                }
                (min_y, max_y)
            }
        };

        // calculate image width and height
        let (img_width, img_height) = {
            let aspect_ratio = (max_x - min_x) / (max_y - min_y);
            if let Some(w) = width {
                if let Some(h) = height {
                    // both width and height are specified
                    (w, h)
                } else {
                    // only width is specified
                    (w, (w as f64 / aspect_ratio) as u32)
                }
            } else if let Some(h) = height {
                // only height is specified
                ((h as f64 * aspect_ratio) as u32, h)
            } else {
                // both width and height are None
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Cannot create an image; both width and height are None",
                )));
            }
        };

        // priority queue that stores pixels
        let mut priority_queue = BinaryHeap::new();

        // a function to push a pixel to the priority queue
        let add_pixel = |priority_queue_ref: &mut BinaryHeap<VisualizerPixel>,
                         pixel_x: u32,
                         pixel_y: u32,
                         pixel_root_node_i: usize| {
            // squared distance between target pixel and root pixel
            let dx = (pixel_x as f64 / img_width as f64) * (max_x - min_x) + min_x
                - self.nodes[pixel_root_node_i].0.x;
            let dy = (pixel_y as f64 / img_height as f64) * (max_y - min_y) + min_y
                - self.nodes[pixel_root_node_i].0.y;
            let squared_distance = dx * dx + dy * dy;

            priority_queue_ref.push(VisualizerPixel {
                x: pixel_x,
                y: pixel_y,
                root_node_i: pixel_root_node_i,
                negative_squared_distance: -squared_distance,
            });
        };

        // set initial pixels which are the nearest pixels from each node
        for (i, node) in self.nodes.iter().enumerate() {
            let pixel_x = ((node.0.x - min_x) / (max_x - min_x) * img_width as f64) as u32;
            let pixel_y = ((node.0.y - min_y) / (max_y - min_y) * img_height as f64) as u32;
            priority_queue.push(VisualizerPixel {
                x: pixel_x,
                y: pixel_y,
                root_node_i: i,
                negative_squared_distance: 0.,
            });
        }

        // table that stores root node index of each pixel
        // if a pixel has no root yet, the value is None
        let mut root_table: Vec<Vec<Option<usize>>> =
            vec![vec![None; img_width as usize]; img_height as usize];

        // determine root node of each pixel
        while let Some(pixel) = priority_queue.pop() {
            // if the pixel is out of the image, skip
            if pixel.y >= img_height || pixel.x >= img_width {
                continue;
            }

            // if the pixel already has an index of root node, skip
            if root_table[pixel.y as usize][pixel.x as usize].is_some() {
                continue;
            }

            // set pixel
            root_table[pixel.y as usize][pixel.x as usize] = Some(pixel.root_node_i);

            // add neighbors as candidates for next pixels
            if pixel.x > 0 {
                add_pixel(&mut priority_queue, pixel.x - 1, pixel.y, pixel.root_node_i);
            }
            if pixel.x < img_width - 1 {
                add_pixel(&mut priority_queue, pixel.x + 1, pixel.y, pixel.root_node_i);
            }
            if pixel.y > 0 {
                add_pixel(&mut priority_queue, pixel.x, pixel.y - 1, pixel.root_node_i);
            }
            if pixel.y < img_height - 1 {
                add_pixel(&mut priority_queue, pixel.x, pixel.y + 1, pixel.root_node_i);
            }
        }

        // get weight range
        let (min_weight, max_weight) = {
            if let Some(weight_range) = self.weight_range {
                weight_range
            } else {
                // calculate weight range from nodes
                let mut min_weight = std::f64::MAX;
                let mut max_weight = std::f64::MIN;
                for node in self.nodes.iter() {
                    let weight = node.1;
                    if weight < min_weight {
                        min_weight = weight;
                    }
                    if weight > max_weight {
                        max_weight = weight;
                    }
                }
                (min_weight, max_weight)
            }
        };

        // create an image
        let mut image_buf = image::RgbImage::new(img_width, img_height);

        // render pixels
        for y in 0..img_height {
            for x in 0..img_width {
                if let Some(root_i) = root_table[y as usize][x as usize] {
                    let score = (self.nodes[root_i].1 - min_weight) / (max_weight - min_weight);
                    image_buf.put_pixel(x, y, {
                        let c = (score * 255.0) as u8;
                        image::Rgb([c, c, c])
                    })
                }
            }
        }

        Ok(image_buf)
    }
}
