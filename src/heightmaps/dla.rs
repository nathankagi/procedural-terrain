use image::{Rgb, RgbImage};
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;

use std::collections::{HashMap, HashSet};
use std::ops::Mul;

#[derive(Clone)]
pub struct DiffusionLimitedAggregationParams {
    pub height: usize, // starting width
    pub width: usize, // starting height
    pub spawns: Vec<Point>,
    pub t: f32, // absorbtion coefficient parameter
    pub particles: u32,
    pub layers: u32, // number of layer scalings, each layer scales width/height by factor of 2
    pub density: f32,
}

#[derive(Clone, Copy)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone)]
pub struct Particle {
    pub linked: Vec<(u32, u32)>,
    pub point: Point,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn key(&self) -> (u32, u32) {
        (self.x, self.y)
    }
}

impl Mul for Point {
    type Output = Point;

    fn mul(self, other: Point) -> Point {
        Point {
            x: (self.x * other.x) as u32,
            y: (self.y * other.y) as u32,
        }
    }
}

impl Mul<u32> for Point {
    type Output = Point;

    fn mul(self, other: u32) -> Point {
        Point {
            x: (self.x * other) as u32,
            y: (self.y * other) as u32,
        }
    }
}


impl Particle {
    pub fn new(point: Point) -> Self {
        Self {
            linked: Vec::new(),
            point,
        }
    }

    pub fn link(&mut self, key: (u32, u32)) {
        self.linked.push(key);
    }

    pub fn height(&self, map: &HashMap<(u32, u32), Particle>) -> f32 {
        let mut m = 0.0;
        for a in self.linked.clone() {
            if let Some(t) = map.get(&a) {
                let h = t.height(map);
                if h > m {
                    m = h;
                }
            } else {
            }
        }

        return m + 1.0;
    }
}

pub fn generate(params: DiffusionLimitedAggregationParams) -> Vec<Vec<f32>> {
    // prepare
    println!("Preparing DLA generation");
    let default_scale_factor: u32 = 2;

    let mut point_map: HashMap<(u32, u32), Particle> =
        HashMap::with_capacity(params.particles as usize);

    // calculate the starting size based on number of layers
    // let base_factor = 2.0_f32.powf((params.layers - 1) as f32);
    // let base_width = (params.width as f32 / base_factor) as u32;
    // let base_height = (params.height as f32 / base_factor) as u32;

    // let mut img = RgbImage::new(basae_width, base_height);
    let mut img = RgbImage::new(params.width as u32, params.height as u32);
    let mut height_map: Vec<Vec<f32>> = vec![vec![0.0; params.width]; params.height];

    println!("Adding starting points to DLA image");
    for p in params.spawns.clone() {
        if point_map.contains_key(&p.key()) {
        } else {
            img.put_pixel(
                p.x as u32,
                p.y as u32,
                // (p.x as f32 / base_factor) as u32,
                // (p.y as f32 / base_factor) as u32,
                Rgb([255, 255, 255]),
            );
            point_map.insert(p.key(), Particle::new(p));
        }
    }

    // create particle map
    // TODO need to look at how many particles to generate

    // steps
    // 1. upscale heightmap
    // 2. upscale particle map
    // 3. add detail to particle map
    // 4. filter particle map
    // 5. add filtered particle map to heightmap
    for l in 0..params.layers {
        // 1. upscale heightmap
        let mut height_map = scale_heightmap(default_scale_factor, &height_map);

        // 2. upscale particle map
        let mut point_map: HashMap<(u32, u32), Particle> = scale_map(default_scale_factor, &point_map);

        // 3. add detail to particle map
        let particles = 10;
        for _ in 0..particles {
            let pos = random_particle(params.height as u32, params.width as u32, &point_map);
            walk(&pos, &params, &mut point_map);
        }

        // 4. filter particle ma
        let filterd = filter_map(&point_map);

        // 5. add filtered particle map to heightmap
        for (i, row) in height_map.iter_mut().enumerate() {
            for (j, val) in row.iter_mut().enumerate() {
                *val += filterd[i][j];
            }
        }

    }

    vec![vec![0.0; params.width]; params.height]
}

fn scale_map(factor: u32, map: &HashMap<(u32, u32), Particle>) -> HashMap<(u32, u32), Particle> {
    // scales the map by some u32 factor, typically 2
    // adds new particles between existing links

    let mut new_map: HashMap<(u32, u32), Particle> = HashMap::new();
    let mut set: HashSet<(u32, u32)> = HashSet::new();

    for k in map.values() {
        if set.contains(&k.point.key()) {
            continue;
        }

        // scale the position of the current point
        let p = Point::new(k.point.x, k.point.y) * factor;
        new_map.insert(p.key(), Particle::new(p));
    }

    return new_map;
}

fn scale_heightmap(factor: u32, map: &Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    vec![vec![0.0; 10]; 10]
}

fn filter_map(map: &HashMap<(u32, u32), Particle>) -> Vec<Vec<f32>> {
    vec![vec![0.0; 10]; 10]
}

fn random_particle(height: u32, width: u32, map: &HashMap<(u32, u32), Particle>) -> Point {
    let mut rng = rand::thread_rng();

    loop {
        let current = Point::new(
            rng.gen_range(0..width) as u32,
            rng.gen_range(0..height) as u32,
        );

        if !map.contains_key(&current.key()) {
            return current;
        }
    }
}

fn walk(
    pos: &Point,
    params: &DiffusionLimitedAggregationParams,
    map: &mut HashMap<(u32, u32), Particle>,
) {
    let mut current = pos.clone();

    // *   3   *
    // 1   x   2
    // *   0   *
    let cords: [(i32, i32); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];

    let mut rng = rand::thread_rng();

    loop {
        // check for connections
        let mut moves: Vec<Point> = Vec::with_capacity(cords.len());
        let mut links: Vec<Point> = Vec::with_capacity(cords.len());
        let mut p_cnt = 0;

        for c in cords {
            let x = c.0 + current.x as i32;
            let y = c.1 + current.y as i32;

            if x >= 0 && x < params.width as i32 && y >= 0 && y < params.height as i32 {
                let p = Point::new(x as u32, y as u32);
                if map.contains_key(&p.key()) {
                    p_cnt = p_cnt + 1;
                    links.push(p);
                } else {
                    moves.push(p);
                }
            }
        }

        // insert if there is connection
        if p_cnt >= 3 {
            let new_particle: Particle = Particle::new(current.clone());
            if let Some(l) = map.get_mut(&links[0].key()) {
                l.link(new_particle.point.key());
            }

            map.insert(new_particle.point.key(), new_particle);
            break;
        }

        if p_cnt > 0 {
            let absorbtion_prob = absorbtion(params.t, p_cnt);
            let prob = rand::random::<f32>();
            if prob <= absorbtion_prob {
                let new_particle: Particle = Particle::new(current.clone());
                if let Some(l) = map.get_mut(&links[0].key()) {
                    l.link(new_particle.point.key());
                }

                map.insert(new_particle.point.key(), new_particle);
                break;
            } else {
            }
        }

        // move
        let pos = rng.gen_range(0..moves.len());
        current = moves[pos].clone();
    }
}

pub fn g(params: DiffusionLimitedAggregationParams) -> Vec<Vec<f32>> {
    let mut img = RgbImage::new(params.width as u32, params.height as u32);
    let mut point_set: HashSet<(u32, u32)> = HashSet::with_capacity(params.particles as usize);
    let mut point_map: HashMap<(u32, u32), Particle> =
        HashMap::with_capacity(params.particles as usize);

    // *   3   *
    // 1   x   2
    // *   0   *
    let cords: [(i32, i32); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];

    for p in params.spawns.clone() {
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

        // create valid start point
        loop {
            current = Point::new(
                rng.gen_range(0..params.width) as u32,
                rng.gen_range(0..params.height) as u32,
            );

            if !point_set.contains(&current.key()) {
                break;
            }
        }

        // walk
        loop {
            // check for connections
            let mut moves: Vec<Point> = Vec::with_capacity(cords.len());
            let mut links: Vec<Point> = Vec::with_capacity(cords.len());
            let mut p_cnt = 0;

            for c in cords {
                let x = c.0 + current.x as i32;
                let y = c.1 + current.y as i32;

                if x >= 0 && x < params.width as i32 && y >= 0 && y < params.height as i32 {
                    let p = Point::new(x as u32, y as u32);
                    if point_set.contains(&p.key()) {
                        p_cnt = p_cnt + 1;
                        links.push(p);
                    } else {
                        moves.push(p);
                    }
                }
            }

            // insert if there is connection
            if p_cnt >= 3 {
                point_set.insert(current.key());

                let new_particle: Particle = Particle::new(current.clone());
                if let Some(l) = point_map.get_mut(&links[0].key()) {
                    l.link(new_particle.point.key());
                }

                point_map.insert(new_particle.point.key(), new_particle);
                break;
            }

            if p_cnt > 0 {
                let absorbtion_prob = absorbtion(params.t, p_cnt);
                let prob = rand::random::<f32>();
                if prob <= absorbtion_prob {
                    point_set.insert(current.key());

                    let new_particle: Particle = Particle::new(current.clone());
                    if let Some(l) = point_map.get_mut(&links[0].key()) {
                        l.link(new_particle.point.key());
                    }

                    point_map.insert(new_particle.point.key(), new_particle);
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

    // let mut particle_heights: HashMap<(u32, u32), f32> = HashMap::with_capacity(point_map.len());

    // for (key, particle) in &point_map {
    //     let calculated_height = particle.height(&point_map);
    //     // particle_heights.insert(*key, calculated_height);
    // }

    // for (key, particle) in &mut point_map {
    //     if let Some(height) = particle_heights.get(key) {
    //         particle.height = *height;
    //     }
    // }

    match point_map.get(&params.spawns[0].key()) {
        Some(l) => println!(
            "height of first spawn is: {}",
            inverse(l.height(&point_map))
        ),
        None => println!("nothing"),
    }

    println!("creating image");
    for each in point_set.iter() {
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
