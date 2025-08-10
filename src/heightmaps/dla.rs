use image::{GrayImage, ImageFormat, Luma, Rgb, RgbImage};
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
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
    pub kernel: Kernel,
}

#[derive(Copy, Clone)]
pub struct Kernel {
    pub size: usize,
    pub value: f32,
    pub k_type: KernelType,
}

#[derive(Copy, Clone)]
pub enum KernelType {
    Gaussian,
    SingleValue,
    Directional,
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

    pub fn height(
        &self,
        map: &HashMap<(u32, u32), Particle>,
        chain: &mut HashMap<(u32, u32), bool>,
    ) -> f32 {
        chain.insert(self.point.key(), true);
        let mut m = 0.0;
        for a in self.linked.clone() {
            if let Some(t) = map.get(&a) {
                let h = t.height(map, chain);
                if h > m {
                    m = h;
                }
            } else {
            }
        }

        return m + 1.0;
    }
}

impl Kernel {
    fn to_vec(&self) -> Vec<Vec<f32>> {
        assert!(self.size % 2 == 1, "Kernel size must be odd");

        match self.k_type {
            KernelType::SingleValue => {
                vec![vec![self.value; self.size]; self.size]
            }

            KernelType::Gaussian => {
                let mut kernel = vec![vec![0.0; self.size]; self.size];
                let mut sum = 0.0;
                let center = (self.size / 2) as isize;

                for y in 0..self.size {
                    for x in 0..self.size {
                        let dx = x as isize - center;
                        let dy = y as isize - center;
                        let exponent =
                            -((dx * dx + dy * dy) as f32) / (2.0 * self.value * self.value);
                        let value =
                            (1.0 / (2.0 * PI * self.value * self.value)) * f32::exp(exponent);
                        kernel[y][x] = value;
                        sum += value;
                    }
                }

                for row in &mut kernel {
                    for v in row.iter_mut() {
                        *v /= sum;
                    }
                }

                kernel
            }

            KernelType::Directional => {
                let mut kernel = vec![vec![0.0; self.size]; self.size];
                let center = (self.size / 2) as isize;

                let angle_rad = self.value.to_radians();
                let dir_x = angle_rad.cos();
                let dir_y = angle_rad.sin();

                for y in 0..self.size {
                    for x in 0..self.size {
                        let dx = x as isize - center;
                        let dy = y as isize - center;

                        let proj = dx as f32 * dir_x + dy as f32 * dir_y;

                        if proj.abs() < 1e-6 {
                            kernel[y][x] = 0.0;
                        } else if proj < 0.0 {
                            kernel[y][x] = -1.0;
                        } else {
                            kernel[y][x] = 1.0;
                        }
                    }
                }

                kernel
            }
        }
    }
}

pub fn generate(params: DiffusionLimitedAggregationParams) -> Vec<Vec<f32>> {
    let scale_factor: u32 = 2;
    let height_scale = 80.0;

    let mut point_map: HashMap<(u32, u32), Particle> =
        HashMap::with_capacity(params.particles as usize);
    let mut height_map: Vec<Vec<f32>> = vec![vec![1.0; params.width]; params.height];

    println!("Adding starting points to DLA image");
    for p in params.spawns.clone() {
        if point_map.contains_key(&p.key()) {
        } else {
            point_map.insert(p.key(), Particle::new(p));
        }
    }

    let layer = 0;
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
        kernel: params.kernel,
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

    println!("saving layer image");
    let mut img = RgbImage::new(layer_params.width as u32, layer_params.height as u32);
    for each in point_map.keys() {
        img.put_pixel(each.0 as u32, each.1 as u32, Rgb([255, 255, 255]));
    }
    let name = format!("./outputs/layer_{}_particle.jpg", 0);
    let _ = img.save_with_format(name, image::ImageFormat::Jpeg);

    // ========== heightmap from particle map  ==========
    let mut chain: HashMap<(u32, u32), bool> = HashMap::with_capacity(params.particles as usize);
    for each in point_map.values() {
        height_map[each.point.y as usize][each.point.x as usize] = height_map
            [each.point.y as usize][each.point.x as usize]
            + (height_scale * gradient_growth_limited(each.height(&point_map, &mut chain)));
    }

    let name = format!("./outputs/layer_{}_heightmap.jpg", layer);
    save_heightmpa_as_jpg(&height_map, &name);

    for layer in 1..params.layers {
        // ========== scale heightmap ==========
        println!("scaling heightmap layer {}", layer);
        height_map = scale_heightmap(&height_map);
        let mut layer_kernel = params.kernel.clone();
        layer_kernel.size = (params.kernel.size as f32 * 2_u32.pow(layer) as f32 / 12.0) as usize;
        if layer_kernel.size % 2 == 0 {
            layer_kernel.size = layer_kernel.size + 1;
        }
        println!("{} kernel size {}", layer, layer_kernel.size);
        println!("{} kernel value {}", layer, layer_kernel.value);
        height_map = filter_heightmap(height_map, layer_kernel.to_vec());
        let name = format!("./outputs/layer_{}_heightmap_base.jpg", layer);
        save_heightmpa_as_jpg(&height_map, &name);

        // ========== scale particle map ==========
        println!("scaling particle map");
        point_map = scale_particle_map(scale_factor, &point_map);

        // ========== add to particle map ==========
        println!("adding to particle map");
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
            kernel: params.kernel,
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

        // ========== add particle map to heightmap ==========
        println!("adding to heightmap");
        let mut chain: HashMap<(u32, u32), bool> =
            HashMap::with_capacity(params.particles as usize);
        for each in point_map.values() {
            let h = height_scale * gradient_growth_limited(each.height(&point_map, &mut chain));
            height_map[each.point.y as usize][each.point.x as usize] =
                height_map[each.point.y as usize][each.point.x as usize] + h;
        }

        // ========== save images ==========
        println!("saving layer image");
        let mut img = RgbImage::new(layer_params.width as u32, layer_params.height as u32);
        for each in point_map.keys() {
            img.put_pixel(each.0 as u32, each.1 as u32, Rgb([255, 255, 255]));
        }
        let name = format!("./outputs/layer_{}_particle.jpg", layer);
        let _ = img.save_with_format(name, image::ImageFormat::Jpeg);

        let name = format!("./outputs/layer_{}_heightmap_detailed.jpg", layer);
        save_heightmpa_as_jpg(&height_map, &name);

        if layer == (params.layers - 1) {
            break;
        }
    }

    println!("Saving final heightmap");
    height_map = scale_heightmap(&height_map);
    let mut layer_kernel = params.kernel.clone();
    layer_kernel.size = (params.height as f32 * 2_u32.pow(params.layers) as f32 / 20.0) as usize;
    if layer_kernel.size % 2 == 0 {
        layer_kernel.size = layer_kernel.size + 1;
    }
    println!("final kernel size {}", layer_kernel.size);
    println!("final kernel value {}", layer_kernel.value);
    height_map = filter_heightmap(height_map, layer_kernel.to_vec());
    save_heightmpa_as_jpg(&height_map, "./outputs/final.jpg");

    // let height_map_scale = 50.0;
    // for row in height_map.iter_mut() {
    //     for val in row.iter_mut() {
    //         *val *= height_map_scale;
    //     }
    // }

    let mut layer_kernel = params.kernel.clone();
    layer_kernel.size = 5;
    layer_kernel.value = 2.0;
    println!("final kernel size {}", layer_kernel.size);
    println!("final kernel value {}", layer_kernel.value);
    height_map = filter_heightmap(height_map, layer_kernel.to_vec());

    height_map
}

fn scale_heightmap(input: &Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    let height = input.len();
    let width = input[0].len();
    let mut output = vec![vec![0.0; width * 2]; height * 2];

    for y in 0..height {
        for x in 0..width {
            let val = input[y][x];
            output[y * 2][x * 2] = val;
            output[y * 2][x * 2 + 1] = val;
            output[y * 2 + 1][x * 2] = val;
            output[y * 2 + 1][x * 2 + 1] = val;
        }
    }

    output
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
            // let mut rng = rand::thread_rng();
            // let num = rng.gen_range(0..3);

            // if x_diff == 0 {
            //     x_move = num - 1;
            // }
            // if y_diff == 0 {
            //     y_move = num - 1;
            // }

            let mid_point = Point::new(
                (middle.0 as i32 + x_move) as u32,
                (middle.1 as i32 + y_move) as u32,
            );

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

    if map.contains_key(&pos.key()) {
        panic!("Trying to walk particle when already exists.")
    }

    // *   3   *
    // 0   x   2
    // *   1  *
    let cords: [(i32, i32); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];

    // 0   7   6
    // 1   x   5
    // 2   3   4
    // let cords: [(i32, i32); 8] = [
    //     (-1, 1),
    //     (-1, 0),
    //     (-1, -1),
    //     (0, 1),
    //     (0, -1),
    //     (1, 1),
    //     (1, 0),
    //     (1, -1),
    // ];

    let mut rng = rand::thread_rng();

    loop {
        // check for connections
        let mut moves: Vec<Point> = Vec::with_capacity(cords.len());
        let mut links: Vec<Point> = Vec::with_capacity(cords.len());
        let mut p_cnt: u32 = 0;

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
        if p_cnt >= (cords.len() as u32) {
            let new_particle: Particle = Particle::new(current.clone());
            if let Some(l) = map.get_mut(&links[0].key()) {
                l.link(new_particle.point.key());
            }

            map.insert(new_particle.point.key(), new_particle);
            break;
        }

        if p_cnt > 0 {
            let absorbtion_prob = absorbtion(params.t, cords.len() as u32, p_cnt);
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

fn filter_heightmap(input: Vec<Vec<f32>>, kernel: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    let height = input.len();
    let width = input[0].len();
    let k = kernel.len();
    let offset = k / 2;

    let mut output = vec![vec![0.0; width]; height];

    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0;
            for ky in 0..k {
                for kx in 0..k {
                    let iy = y as isize + ky as isize - offset as isize;
                    let ix = x as isize + kx as isize - offset as isize;
                    if iy >= 0 && iy < height as isize && ix >= 0 && ix < width as isize {
                        sum += input[iy as usize][ix as usize] * kernel[ky][kx];
                    }
                }
            }
            output[y][x] = sum;
        }
    }

    output
}

fn gradient_growth_limited(x: f32) -> f32 {
    1.0 - (1.0 / (1.0 + x))
}

fn gradient_growth_linear(x: f32) -> f32 {
    x
}

fn gradient_growth_ln(x: f32) -> f32 {
    x.ln()
}

fn absorbtion(t: f32, a: u32, b: u32) -> f32 {
    return t.powi((a - b) as i32).min(1.0);
}

fn save_heightmpa_as_jpg(height_map: &Vec<Vec<f32>>, filename: &str) {
    let height = height_map.len();
    let width = height_map[0].len();

    let mut min_val = f32::MAX;
    let mut max_val = f32::MIN;

    for row in height_map {
        for &val in row {
            if val < min_val {
                min_val = val;
            }
            if val > max_val {
                max_val = val;
            }
        }
    }

    let range = if (max_val - min_val).abs() < std::f32::EPSILON {
        1.0
    } else {
        max_val - min_val
    };

    let mut img = GrayImage::new(width as u32, height as u32);

    for (y, row) in height_map.iter().enumerate() {
        for (x, &val) in row.iter().enumerate() {
            let norm = (val - min_val) / range;
            let pixel_value = (norm * 255.0).round() as u8;
            img.put_pixel(x as u32, y as u32, Luma([pixel_value]));
        }
    }

    let _ = img.save_with_format(filename, image::ImageFormat::Jpeg);
}
