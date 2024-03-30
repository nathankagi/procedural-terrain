use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

fn fade(t: f64) -> f64 {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    (a + t * (b - a)).into()
}

// simplified algorithm for perlin noise gradient vector
fn grad3d(hash: i32, x: f64, y: f64, z: f64) -> f64 {
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

// original algorithm for perlin noise gradient vector
/*
pub fn grad3d(hash: i32, x: f64, y: f64, z: f64) -> f64 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };

    let v = if h < 4 {
        y
    } else if h == 12 || h == 14 {
        x
    } else {
        z
    };

    return if (h & 1) == 0 { u } else { -u } + if (h & 2) == 0 { v } else { -v };
}
*/

fn grad2d(hash: i32, x: f64, y: f64) -> f64 {
    let vec = vec![[0, 1], [0, -1], [1, 0], [-1, 0]];
    let index = hash % 4;
    let g = vec[index as usize];
    return (g[0] as f64 * x) + (g[1] as f64 * y);
}

fn perlin3d(x: f64, y: f64, z: f64, p: &Vec<i32>) -> f64 {
    let _x = x.floor() as usize & 255;
    let _y = y.floor() as usize & 255;
    let _z = z.floor() as usize & 255;

    let xf: f64 = x - _x as f64;
    let yf: f64 = y - _y as f64;
    let zf: f64 = z - _z as f64;

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

    let x01 = lerp(
        grad3d(aaa, xf, yf, zf) as f64,
        grad3d(baa, xf - 1.0, yf, zf) as f64,
        u,
    );
    let x02 = lerp(
        grad3d(aba, xf, yf - 1.0, zf) as f64,
        grad3d(bba, xf - 1.0, yf - 1.0, zf) as f64,
        u,
    );
    let y1 = lerp(x01, x02, v);

    let x11 = lerp(
        grad3d(aab, xf, yf, zf - 1.0) as f64,
        grad3d(bab, xf - 1.0, yf, zf - 1.0) as f64,
        u,
    );
    let x12 = lerp(
        grad3d(abb, xf, yf - 1.0, zf - 1.0) as f64,
        grad3d(bbb, xf - 1.0, yf - 1.0, zf - 1.0) as f64,
        u,
    );
    let y2 = lerp(x11, x12, v);

    return lerp(y1, y2, w);

    // let a = p[_x] as usize + _y;
    // let aa = p[a] as usize + _z;
    // let ab = p[a + 1] as usize + _z;

    // let b = p[_x + 1] as usize + _y;
    // let ba = p[b] as usize + _z;
    // let bb = p[b + 1] as usize + _z;

    // return lerp(
    //     lerp(
    //         lerp(
    //             grad3d(p[aa], xf, yf, zf),
    //             grad3d(p[ba], xf - 1.0, yf, zf),
    //             u,
    //         ),
    //         lerp(
    //             grad3d(p[ab], xf, yf - 1.0, zf),
    //             grad3d(p[bb], xf - 1.0, yf - 1.0, zf),
    //             u,
    //         ),
    //         v,
    //     ),
    //     lerp(
    //         lerp(
    //             grad3d(p[aa + 1], xf, yf, zf - 1.0),
    //             grad3d(p[ba + 1], xf - 1.0, yf, zf - 1.0),
    //             u,
    //         ),
    //         lerp(
    //             grad3d(p[ab + 1], xf, yf - 1.0, zf - 1.0),
    //             grad3d(p[bb + 1], xf - 1.0, yf - 1.0, zf - 1.0),
    //             u,
    //         ),
    //         v,
    //     ),
    //     w,
    // );
}

fn perlin2d(x: f64, y: f64, p: &Vec<i32>) -> f64 {
    let _x = x.floor() as usize & 255;
    let _y = y.floor() as usize & 255;

    let xf = x - _x as f64;
    let yf = y - _y as f64;

    let u = fade(xf);
    let v = fade(yf);

    let aa = grad2d(p[p[_x] as usize + _y], xf, yf);
    let ab = grad2d(p[p[_x] as usize + _y + 1], xf, yf - 1.0);
    let bb = grad2d(p[p[_x + 1] as usize + _y + 1], xf - 1.0, yf - 1.0);
    let ba = grad2d(p[p[_x + 1] as usize + _y], xf - 1.0, yf);

    let x1 = lerp(aa, ba, u);
    let x2 = lerp(ab, bb, u);
    return lerp(x1, x2, v);
}

pub fn octave_perlin3d(
    x: f64,
    y: f64,
    z: f64,
    octaves: i32,
    persistence: f64,
    permutation: &Vec<i32>,
) -> f64 {
    let mut value = 0.0;
    let mut max_value = 1.0;

    for o in 0..octaves {
        let f = 2.0f64.powi(o);
        let amplitude = persistence.powi(o);

        max_value += amplitude;
        value += perlin3d(x * f, y * f, z * f, permutation) * amplitude;
    }
    return value / max_value;
}

fn octave_perlin2d(x: f64, y: f64, octaves: i32, persistence: f64, permutation: &Vec<i32>) -> f64 {
    let mut value = 0.0;
    let mut max_value = 1.0;

    for o in 0..octaves {
        let f = 2.0f64.powi(o);
        let amplitude = persistence.powi(o);

        max_value += amplitude;
        value += perlin2d(x * f, y * f, permutation) * amplitude;
    }
    return value / max_value;
}

pub fn generate_permutation(seed: u32) -> Vec<i32> {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed.into());
    let mut permutation: Vec<i32> = (0..256).collect();
    permutation.shuffle(&mut rng);

    let mut p: Vec<i32> = vec![];
    for i in 0..512 {
        p.push(permutation[i % 256]);
    }
    return p;
}
