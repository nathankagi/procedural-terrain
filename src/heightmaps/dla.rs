use image::{Rgb, RgbImage};
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;

use std::{
    collections::{HashMap, HashSet},
    ops::Add,
};

#[derive(Clone)]
pub struct DiffusionLimitedAggregationParams {
    pub height: usize,
    pub width: usize,
    pub spawns: Vec<Point>,
    pub t: f32, // absorbtion coefficient parameter
    pub particles: u32,
    pub layers: u32,
    pub density: f32,
}

#[derive(Clone)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone)]
pub struct Pixel {
    pub attached: Vec<Point>,
    pub point: Point,
    height: f32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn key(&self) -> (u32, u32) {
        (self.x, self.y)
    }
}

impl<'a, 'b> Add<&'b Point> for &'a Point {
    type Output = Point;

    fn add(self, other: &'b Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Pixel {
    pub fn new(point: Point) -> Self {
        Self {
            attached: Vec::new(),
            point,
            height: 0.0,
        }
    }

    pub fn attach(&mut self, point: &Point) {
        self.attached.push(point.clone());
        self.height == 0.0;
    }

    pub fn height(&mut self) -> f32 {
        if self.height == 0.0 {
            self.height = inverse((self.attached.len() + 1) as f32);
        }
        return self.height;
    }

    pub fn clear_height(&mut self) {
        self.height = 0.0;
    }
}

pub fn generate(params: DiffusionLimitedAggregationParams) -> Vec<Vec<f32>> {
    let mut img = RgbImage::new(params.width as u32, params.height as u32);
    let mut point_map: HashSet<(u32, u32)> = HashSet::with_capacity(params.particles as usize);

    // *   3   *
    // 1   x   2
    // *   0   *
    let cords: [(i32, i32); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];

    for p in params.spawns {
        if point_map.contains(&p.key()) {
        } else {
            point_map.insert(p.key());
            img.put_pixel(p.x as u32, p.y as u32, Rgb([255, 255, 255]));
        }
    }

    let mut rng = rand::thread_rng();

    let bar = ProgressBar::new(params.particles as u64);
    let style = ProgressStyle::with_template(
        "{bar:40} {percent}% | eta: {eta} elapsed: {elapsed} {pos:>7}/{len:7}",
    )
    .unwrap();
    bar.set_style(style);

    let mut current = Point::new(0, 0);

    for _ in 0..params.particles {
        bar.inc(1);

        loop {
            current = Point::new(
                rng.gen_range(0..params.width) as u32,
                rng.gen_range(0..params.height) as u32,
            );

            if point_map.contains(&current.key()) {
            } else {
                break;
            }
        }

        loop {
            // check for connections
            let mut moves: Vec<Point> = Vec::with_capacity(cords.len());
            let mut p_cnt = 0;

            for c in cords {
                let x = c.0 + current.x as i32;
                let y = c.1 + current.y as i32;

                if x >= 0 && x < params.width as i32 && y >= 0 && y < params.height as i32 {
                    let p = Point::new(x as u32, y as u32);
                    if point_map.contains(&p.key()) {
                        p_cnt = p_cnt + 1;
                    } else {
                        moves.push(p);
                    }
                }
            }

            // insert if there is connection
            if p_cnt > 0 {
                point_map.insert(current.key());
                break;
            }

            // move
            let pos = rng.gen_range(0..moves.len());
            current = moves[pos].clone();
        }
    }

    bar.finish();

    println!("creating image");
    for each in point_map {
        img.put_pixel(each.0 as u32, each.1 as u32, Rgb([255, 255, 255]));
    }
    let _ = img.save_with_format(
        "C:/projects/procedural-terrain/img.jpg",
        image::ImageFormat::Jpeg,
    );

    vec![vec![0.0; params.width]; params.height]
}

fn inverse(x: f32) -> f32 {
    1.0 - (1.0 / (1.0 + x))
}

fn absorbtion(t: f32, b: u32) -> f32 {
    return min_max(t.powi((3 - b) as i32), 0.0, 1.0);
}

fn min_max(val: f32, min: f32, max: f32) -> f32 {
    return val.max(max).min(min);
}
