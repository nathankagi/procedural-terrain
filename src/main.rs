use plotters::prelude::*;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

fn fade(t: f64) -> f64 {
    return (6.0 * t.powi(5) - 15.0 * t.powi(4) + 1.0 * t.powi(3)).into();
}

fn lerp(t: f64, a: f64, b: f64) -> f64 {
    (a + t * (b - a)).into()
}

fn grad(hash: i32, x: f64, y: f64, z: f64) -> f64 {
    let input = hash & 0xF;
    match input {
        0x0 => return x + y,
        0x1 => return -x + y,
        0x2 => return x - y,
        0x3 => return -x - y,
        0x4 => return x + z,
        0x5 => return -x + z,
        0x6 => return x - z,
        0x7 => return -x - z,
        0x8 => return y + z,
        0x9 => return -y + z,
        0xA => return y - z,
        0xB => return -y - z,
        0xC => return y + x,
        0xD => return -y + z,
        0xE => return y - x,
        0xF => return -y - z,
        _ => return 0.0, // should never happen
    }
}

fn perlin(x: f64, y: f64, z: f64, p: &Vec<i32>) -> f64 {
    let _x = (x.floor() as usize % p.len()) & 255;
    let _y = (y.floor() as usize % p.len()) & 255;
    let _z = (z.floor() as usize % p.len()) & 255;

    let xf: f64 = x - _x as f64;
    let yf: f64 = x - _y as f64;
    let zf: f64 = x - _z as f64;

    let u = fade(xf);
    let v = fade(yf);
    let w = fade(zf);

    let aaa = p[p[p[_x] as usize + _y] as usize + _z];
    let aba = p[p[p[_x] as usize + _y + 1] as usize + _z];
    let aab = p[p[p[_x] as usize + _y] as usize + _z + 1];
    let abb = p[p[p[_x] as usize + _y + 1] as usize + _z + 1];
    let baa = p[p[p[_x + 1] as usize + _y] as usize + _z];
    let bba = p[p[p[_x + 1] as usize + _y + 1] as usize + _z];
    let bab = p[p[p[_x + 1] as usize + _y] as usize + _z + 1];
    let bbb = p[p[p[_x + 1] as usize + _y + 1] as usize + _z + 1];

    let x1 = lerp(
        grad(aaa, xf, yf, zf) as f64,
        grad(baa, xf - 1.0, yf, zf) as f64,
        u,
    );
    let x2 = lerp(
        grad(aba, xf, yf - 1.0, zf) as f64,
        grad(bba, xf - 1.0, yf - 1.0, zf) as f64,
        u,
    );
    let y1 = lerp(x1, x2, v);

    let x1 = lerp(
        grad(aab, xf, yf, zf - 1.0) as f64,
        grad(bab, xf - 1.0, yf, zf - 1.0) as f64,
        u,
    );
    let x2 = lerp(
        grad(abb, xf, yf - 1.0, zf - 1.0) as f64,
        grad(bbb, xf - 1.0, yf - 1.0, zf - 1.0) as f64,
        u,
    );
    let y2 = lerp(x1, x2, v);

    return (lerp(y1, y2, w) + 1.0) / 2.0;
}

fn generate_permutation(seed: u64) -> Vec<i32> {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);
    let mut permutation: Vec<i32> = (0..256).collect();
    permutation.shuffle(&mut rng);

    let mut p: Vec<i32> = vec![];
    for i in 0..512 {
        p.push(permutation[i % 256]);
    }
    return p;
}

fn main() {
    let width = 512;
    let height = 512;

    let scale = 0.1;
    let octaves = 6;
    let persistence: f64 = 0.5;

    let mut rng = rand::thread_rng();
    let seed = rng.gen::<u64>();

    let permutation = generate_permutation(seed);

    let mut noise_map = vec![vec![0.0; height]; width];
    for i in 0..width {
        for j in 0..height {
            let mut value = 0.0;
            for o in 0..octaves {
                let frequency = 2.0f64.powi(o);
                let amplitude = persistence.powi(o);
                value += perlin(
                    i as f64 * scale * frequency,
                    j as f64 * scale * frequency,
                    0.0,
                    &permutation,
                ) * amplitude;
            }
            noise_map[i][j] = value;
        }
    }

    let root = BitMapBackend::new("vector_plot.png", (600, 400)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    // Create a chart context
    let mut chart = ChartBuilder::on(&root)
        .caption("2D Vector Plot", ("Arial", 20))
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_ranged(
            data.iter().map(|point| point[0]).min()..data.iter().map(|point| point[0]).max(),
            data.iter().map(|point| point[1]).min()..data.iter().map(|point| point[1]).max(),
        )
        .unwrap();

    // Draw points for each data pair in the vector
    chart
        .draw_series(PointSeries::of_element(
            data.iter().map(|point| (point[0], point[1])),
            5,
            &RED,
            &|c, s, st| {
                return EmptyElement::at(c)
                    + Circle::new((0, 0), s, st.filled())
                    + Text::new(format!("{:?}", c), (10, 0), ("Arial", 10));
            },
        ))
        .unwrap();

    // Save the plot to a file
    root.present().unwrap();
}
