use image::{Rgb, RgbImage};
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;

use std::{
    collections::{hash_map, HashMap, HashSet},
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
pub struct Particle <'a> {
    pub linked: Vec<&'a Particle<'a>>,
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

impl<'a> Particle<'a> {
    pub fn new(point: Point) -> Self {
        Self {
            linked: Vec::new(),
            point,
            height: 0.0,
        }
    }

    pub fn link(&mut self, particle: &'a Particle<'a>) {
        self.linked.push(particle);
    }

    pub fn height(&self) -> f32 {
        let mut m = 0.0;
        for a in self.linked.iter() {
            if a.height() > m {
                m = a.height();
            }
        }

        return self.height;
    }
}

pub fn generate(params: DiffusionLimitedAggregationParams) -> Vec<Vec<f32>> {
    let mut img = RgbImage::new(params.width as u32, params.height as u32);
    let mut point_set: HashSet<(u32, u32)> = HashSet::with_capacity(params.particles as usize);
    let mut point_map: HashMap<(u32, u32), Particle> = HashMap::with_capacity(params.particles as usize);

    // *   3   *
    // 1   x   2
    // *   0   *
    let cords: [(i32, i32); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];

    for p in params.spawns {
        if point_set.contains(&p.key()) {
        } else {
            point_set.insert(p.key());
            img.put_pixel(p.x as u32, p.y as u32, Rgb([255, 255, 255]));
            point_map.insert(p.key(), Particle::new(p));
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

            if point_set.contains(&current.key()) {
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
                    if point_set.contains(&p.key()) {
                        p_cnt = p_cnt + 1;
                    } else {
                        moves.push(p);
                    }
                }
            }

            // insert if there is connection
            if p_cnt >= 3 {
                point_set.insert(current.key());
                let mut new_particle: Particle<'_> = Particle::new(current.clone());
                
                if let Some(l) = point_map.get(&current.key()) {
                    new_particle.link(l);
                }

                point_map.insert(new_particle.point.key(), new_particle);
                break;
            }

            if p_cnt > 0 {
                let absorbtion_prob = absorbtion(params.t, p_cnt);
                let prob = rand::random::<f32>();
                if prob <= absorbtion_prob {
                    point_set.insert(current.key());
                    break;
                } else {
                }
            }

            // move
            let pos = rng.gen_range(0..moves.len());
            current = moves[pos].clone();
        }
    }

    bar.finish();

    println!("creating image");
    for each in point_set {
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
    return t.powi((3 - b) as i32);
}

fn scale_points() -> HashMap<(u32, u32), Point> {
    let map: HashMap<(u32, u32), Point> = HashMap::new();

    return map;
}

fn scale_image() {}

fn filter() {}