use image::{Rgb, RgbImage};
use rand::Rng;

use std::collections::HashMap;

#[derive(Clone)]
pub struct DiffusionLimitedAggregationParams {
    pub height: usize,
    pub width: usize,
    pub spawns: Vec<(u32, u32)>,
    pub t: f32, // absorbtion coefficient parameter
    pub particles: u32,
    pub layers: u32,
    pub density: f32,
}

#[derive(Clone)]
struct Point {
    // pub attached: Vec<&Point>,
    pub position: (u32, u32),
    height: f32,
}

impl Point {
    pub fn new(position: (u32, u32)) -> Self {
        Self {
            // attached: Vec::new(),
            position: position,
            height: 0.0,
        }
    }

    pub fn attach(&mut self, point: &Point) {
        // self.attached.push(point);
    }

    // pub fn height(&mut self, func: Fn(f32) -> f32) -> f32 {
    //     // return 1 - (1 / (1 + self.attached.len())) + 1;
    //     if self.height == 0.0 {
    //         self.height = func(self.attached.len() + 1);
    //     } else {
    //     }
    //     return self.height;
    // }

    pub fn clear_height(&mut self) {
        self.height = 0.0;
    }
}

pub fn generate(params: DiffusionLimitedAggregationParams) -> Vec<Vec<f32>> {
    let mut map = vec![vec![0.0; params.width]; params.height];
    let mut img = RgbImage::new(params.width as u32, params.height as u32);
    let mut point_map: HashMap<(u32, u32), Point> = HashMap::new();

    let d = min_max(params.density, 0.0, 1.0);
    // for p in params.spawns {
    //     if point_map.contains_key(&(p.x as u32, p.y as u32)) {
    //     } else {
    //         point_map.insert((p.x as u32, p.y as u32), Point::new(p));
    //         img.put_pixel(p.x as u32, p.y as u32, Rgb([255, 255, 255]));
    //     }
    // }
    for p in params.spawns {
        if point_map.contains_key(&(p.0 as u32, p.1 as u32)) {
        } else {
            point_map.insert((p.0 as u32, p.1 as u32), Point::new(p));
            img.put_pixel(p.0 as u32, p.1 as u32, Rgb([255, 255, 255]));
        }
    }

    let mut rng = rand::thread_rng();

    for _ in 0..params.particles {
        // determine border of aggregate and aggregate border limit

        // create a spawn point outside of the aggregate border plus scaler
        // let current = Vec2::new(
        //     rng.gen_range(0..params.width) as f32,
        //     rng.gen_range(0..params.height) as f32,
        // );

        let mut current = (
            rng.gen_range(0..params.width) as u32,
            rng.gen_range(0..params.height) as u32,
        );

        // if (point_map.keys().len() as f32 / params.particles as f32) > d && d > 0.0 {
        if false {
            // density is high enough
            return vec![vec![0.0; params.width]; params.height];
        }

        loop {
            // *   3   *
            // 1   x   2
            // *   0   *
            let dir = rng.gen_range(0..4) as u32;
            // let next: Vec2 = match dir {
            //     0 => current + Vec2::new(0.0, -1.0),
            //     1 => current + Vec2::new(-1.0, 0.0),
            //     2 => current + Vec2::new(1.0, 0.0),
            //     3 => current + Vec2::new(0.0, 1.0),
            //     _ => Vec2::new(0.0, 0.0),
            // };

            let c: (i32, i32) = (current.0 as i32, current.1 as i32);
            let next: (i32, i32) = match dir {
                0 => (c.0 + 0, c.1 - 1),
                1 => (c.0 - 1, c.1 + 0),
                2 => (c.0 + 1, c.1 + 0),
                3 => (c.0 + 0, c.1 + 1),
                _ => c,
            };
            let next = (next.0 as u32, next.1 as u32);

            // if (next.x < 0.0 || next.x >= params.width as f32)
            //     || (next.y < 0.0 || next.y >= params.height as f32)
            // {
            //     continue;
            // } else if point_map.contains_key(&(next.x as u32, next.y as u32)) {
            //     continue;
            // }
            if (next.0 < 0 || next.0 >= params.width as u32)
                || (next.1 < 0 || next.1 >= params.height as u32)
            {
                continue;
            } else if point_map.contains_key(&(next.0 as u32, next.1 as u32)) {
                continue;
            }

            // remove if particle is outside of border limit
            if false {
                break;
            }

            // move particle and check surroundings
            current = next.clone();

            // let positions = [
            //     current + Vec2::new(0.0, -1.0),
            //     current + Vec2::new(-1.0, 0.0),
            //     current + Vec2::new(1.0, 0.0),
            //     current + Vec2::new(0.0, 1.0),
            // ];

            let c: (i32, i32) = (current.0 as i32, current.1 as i32);
            let positions = [
                (c.0 + 0, c.1 - 1),
                (c.0 - 1, c.1 + 0),
                (c.0 + 1, c.1 + 0),
                (c.0 + 0, c.1 + 1),
            ];

            let mut p_cnt = 0;
            for p in positions {
                // if point_map.contains_key(&(p.x as u32, p.y as u32)) {
                if point_map.contains_key(&(p.0 as u32, p.1 as u32)) {
                    // println!("found ({}, {}) in keys", p.0, p.1);
                    p_cnt = p_cnt + 1;
                }
            }

            if p_cnt == 0 {
                continue;
            }

            let abs = absorbtion(params.t, p_cnt);
            let chance = rand::random::<f32>();

            if chance >= abs {
                let p = Point::new(current);
                // positions[0].attach(&p);
                // img.put_pixel(
                //     p.position.x as u32,
                //     p.position.y as u32,
                //     Rgb([255, 255, 255]),
                // );
                img.put_pixel(
                    p.position.0 as u32,
                    p.position.1 as u32,
                    Rgb([255, 255, 255]),
                );
                // point_map.insert((p.position.x as u32, p.position.y as u32), p);
                point_map.insert((p.position.0 as u32, p.position.1 as u32), p);
                // println!("({}, {}) attached", current.0, current.1);
                break;
            }
        }
    }

    let _ = img.save_with_format(
        "/Users/nathankagi/dev/procedural-terrain/img.jpg",
        image::ImageFormat::Jpeg,
    );

    vec![vec![0.0; params.width]; params.height]
}

// fn upscale_points() {}

// fn upscape_image() {}

fn inverse(x: f32) -> f32 {
    1.0 - (1.0 / (1.0 + x))
}

fn absorbtion(t: f32, b: u32) -> f32 {
    return min_max(t.powi((3 - b) as i32), 0.0, 1.0);
}

// fn filter(&map: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
//     let map_filt = vec![vec![0.0; map[0].len()]; map.len()];
// }

fn min_max(val: f32, min: f32, max: f32) -> f32 {
    return val.max(max).min(min);
}
