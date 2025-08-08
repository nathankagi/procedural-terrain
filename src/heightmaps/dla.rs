use image::{Rgb, RgbImage};
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::ops::{Mul, Sub};

#[derive(Clone)]
pub struct DiffusionLimitedAggregationParams {
    pub height: usize, // starting width
    pub width: usize,  // starting height
    pub spawns: Vec<Point>,
    pub t: f32, // absorbtion coefficient parameter
    pub particles: u32,
    pub layers: u32, // number of layer scalings, each layer scales width/height by factor of 2
    pub density: f32,
}

#[derive(Copy, Clone)]
pub enum DiffusionLimitedAggregationMode {
    Particle,
    Image,
}

pub enum ParticleSpawnPattern {
    Random,
    Edge,
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

    pub fn mid(&self, other: &Point) -> Point {
        let x = self.x as i32 * 2 - (self.x as i32 - other.x as i32);
        let y = self.y as i32 * 2 - (self.y as i32 - other.y as i32);

        Point::new(x as u32, y as u32)
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

impl Sub<Point> for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point {
            x: (self.x - other.x) as u32,
            y: (self.y - other.y) as u32,
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
    let scale_factor: u32 = 2;

    let mut point_map: HashMap<(u32, u32), Particle> =
        HashMap::with_capacity(params.particles as usize);
    let mut height_map: Vec<Vec<f32>> = vec![vec![0.0; params.width]; params.height];

    println!("Adding starting points to DLA image");
    for p in params.spawns.clone() {
        if point_map.contains_key(&p.key()) {
        } else {
            point_map.insert(p.key(), Particle::new(p));
        }
    }

    for layer in 0..params.layers {
        let layer_params = DiffusionLimitedAggregationParams {
            height: params.height * 2_u32.pow(layer) as usize,
            width: params.width * 2_u32.pow(layer) as usize,
            spawns: params.spawns.clone(),
            t: params.t,
            // particles: params.particles * 2_u32.pow(2 * layer),
            particles: (0.08
                * params.height as f32
                * 2_u32.pow(layer) as f32
                * params.width as f32
                * 2_u32.pow(layer) as f32) as u32,
            layers: params.layers,
            density: params.density,
        };

        println!(
            "populating layer {}/{} with dimensions {}x{}",
            layer + 1,
            params.layers,
            layer_params.width,
            layer_params.height
        );

        let bar = ProgressBar::new(layer_params.particles as u64);
        let style = ProgressStyle::with_template(
            "dla layer: {bar:40} {percent}% | eta: {eta} elapsed: {elapsed} {pos:>7}/{len:7}",
        )
        .unwrap();
        bar.set_style(style);

        for _ in 0..layer_params.particles {
            let pos = &random_particle(
                layer_params.height as u32,
                layer_params.width as u32,
                &point_map,
            );
            walk(pos, &layer_params, &mut point_map);
            bar.inc(1);
        }

        bar.finish();

        // println!("creating image");
        // let mut img = RgbImage::new(layer_params.width as u32, layer_params.height as u32);
        // for each in point_map.keys() {
        //     img.put_pixel(each.0 as u32, each.1 as u32, Rgb([255, 255, 255]));
        // }
        // let name = format!("./outputs/img_{}.jpg", layer);
        // let _ = img.save_with_format(name, image::ImageFormat::Jpeg);

        if layer == (params.layers - 1) {
            break;
        }

        point_map = scale_particle_map(scale_factor, &point_map);
    }

    println!("Complted final layer with {} particles", point_map.len());

    vec![vec![0.0; params.width]; params.height]
}

fn scale_particle_map(
    factor: u32,
    map: &HashMap<(u32, u32), Particle>,
) -> HashMap<(u32, u32), Particle> {
    fn mid(a: (u32, u32), b: (u32, u32)) -> (u32, u32) {
        let x = a.0 as i32 * 2 - (a.0 as i32 - b.0 as i32);
        let y = a.1 as i32 * 2 - (a.1 as i32 - b.1 as i32);

        (x as u32, y as u32)
    }

    fn scale_recursive(
        factor: u32,
        particle: &Particle,
        old_map: &HashMap<(u32, u32), Particle>,
        new_map: &mut HashMap<(u32, u32), Particle>,
        set: &mut HashSet<(u32, u32)>,
    ) {
        // particle already updated
        if set.contains(&particle.point.key()) {
            return;
        }

        // scale the position of the current point
        let scaled_point = Point::new(particle.point.x, particle.point.y) * factor;

        set.insert(particle.point.key()); // track old keys that have been updated
        let mut scaled_particle = Particle::new(scaled_point);

        for link in particle.linked.clone() {
            let link_scaled = Point::new(link.0, link.1) * factor; // scale the linked particle
            let middle = mid(particle.point.key(), link); // find the mid point between particle and link

            let x_diff: i32 = link_scaled.x as i32 - middle.0 as i32;
            let y_diff: i32 = link_scaled.y as i32 - middle.1 as i32;

            let mut x_move: i32 = 0;
            let mut y_move: i32 = 0;
            let mut rng = rand::thread_rng();
            let num = rng.gen_range(0..3);

            if x_diff == 0 {
                x_move = num - 1;
            }
            if y_diff == 0 {
                y_move = num - 1;
            }

            let mid_point = Point::new(
                (middle.0 as i32 + x_move) as u32,
                (middle.1 as i32 + y_move) as u32,
            );

            // TODO randomly shift the mid point by one position

            let mut mid_particle = Particle::new(mid_point); // create mid point particle
            mid_particle.link(link_scaled.key()); // link scaled link to mid particle
            new_map.insert(mid_particle.point.key(), mid_particle); // insert the new mid particle

            scaled_particle.link(mid_point.key()); // link mid particle to scaled particle

            if let Some(next_particle) = old_map.get(&link) {
                scale_recursive(factor, next_particle, old_map, new_map, set);
            }
        }

        new_map.insert(scaled_particle.point.key(), scaled_particle);
    }

    // scales the map by some u32 factor, typically 2
    // adds new particles between existing links

    let mut new_map: HashMap<(u32, u32), Particle> = HashMap::new();
    let mut set: HashSet<(u32, u32)> = HashSet::new();

    for k in map.values() {
        if set.contains(&k.point.key()) {
            continue;
        }

        scale_recursive(factor, k, map, &mut new_map, &mut set);
    }

    return new_map;
}

fn random_particle(height: u32, width: u32, map: &HashMap<(u32, u32), Particle>) -> Point {
    let mut rng = rand::thread_rng();
    let spawn_mode = ParticleSpawnPattern::Random;

    match spawn_mode {
        ParticleSpawnPattern::Edge => {
            return Point::new(0, 0);
        }
        ParticleSpawnPattern::Random => {
            let mut i = 0;
            loop {
                let current = Point::new(
                    rng.gen_range(0..width) as u32,
                    rng.gen_range(0..height) as u32,
                );

                if !map.contains_key(&current.key()) {
                    return current;
                }

                if i > 10000 {
                    println!("WARNING: new particle failed to find new spot");
                    return Point::new(0, 0);
                }
                i = i + 1;
            }
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
        if moves.len() > 0 {
            let pos = rng.gen_range(0..moves.len());
            current = moves[pos].clone();
        } else {
        }
    }
}

fn gradient_growth_limited(x: f32) -> f32 {
    1.0 - (1.0 / (1.0 + x))
}

fn gradient_growth_linear(x: f32) -> f32 {
    x
}

fn absorbtion(t: f32, b: u32) -> f32 {
    return t.powi((3 - b) as i32);
}
