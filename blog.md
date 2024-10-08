# Terrain Modelling

## Introduction

I have been looking for some simulation style projects where using a lower level language would be beneficial. Although it seems almost every second person has implimented their own, I want to try my hand at a terrain/environment simulation.

## First Implementation

I will be using Rust and [bevy](https://bevyengine.org/), a game engine built in rust, for this project. Initially I was considering using a library to use Vulkan or OpenGL directly in either C/C++ or Rust. At some stage I would like to make a super minimal engine that I can use for this and future projects but the simplicity of bevy for getting something working made it very apppealing. It is also a nice excuse to continue learning rust as I don't have the opportunity to explore the language at work.

### Noise

To generate terrain I've looked at a handlful of different algorithms but Perlin noise seems to pop up everwhere. There are so many blog posts and tutorials for creating perlin noise it feels slightly dizzying trying to sift through the useful material. I found some material from [Matt Zucker](https://mzucker.github.io/html/perlin-noise-math-faq.html) very useful in addition to [this](https://www.cs.umd.edu/class/spring2018/cmsc425/Lects/lect13-2d-perlin.pdf) lecture on perlin noise. Combined with [Ken Perlin](https://mrl.cs.nyu.edu/~perlin/noise/)'s original code it was fairly simple to get something to compile. I had also seen a [blog](http://riven8192.blogspot.com/2010/08/calculate-perlinnoise-twice-as-fast.html) post several times discussing a faster implementation of the grad function. It seems to behave the same so far so I'll continue to run with that.

I had to do a little debugging, resolving the typical incorrect sign issues and so forth but in the end things looked okay. At the end of the day it was far easier to follow Ken Perlin's implementation directly, converting to Rust in this case of course, rather than interpreting some other blog. To visualise the data I used a PointList in bevy so get an initial view of the noise I was creating.

![point_mesh](./resources/point_mesh.png "Point List Mesh")

### Meshing

Some of the newer Bevy [custom meshing](https://bevyengine.org/examples/3D%20Rendering/generate-custom-mesh/) documentation was quite nice for getting a mesh going. After a few attempts where the entire screen was covered by triangles spanning from one side of the screen to another I had a terrain that looked somewhat reasonable.

Initially I used only verticle normals, trying to get the lighting (both ambient and directional) working with this setup led to some interesting looking shadow behaviour that made the terrain look very strange. To get rid of these stange shadows I actually calculated the normal vectors using an answer on [stack overflow](https://stackoverflow.com/questions/6656358/calculating-normals-in-a-triangle-mesh). The calculation simplifies to calculating the height differences between the adjacent row and column points and the output looked much nicer.

![triangle_mesh](./resources/triangle_mesh.png "Triangle List Mesh")

There are still some issues with the lighting which become apparent on both very smooth and very hilly terrain. The smooth surfaces become very shiny and the sharper terrain becomes dull. This is due to the vertical scale factor in the normal vector calculation mentioned before.

```rust
let normal = Vector3::new(height_diff_x, 2.0, height_diff_z).normalize();
```

Typically a value of 2 is suitable but in these extremes would need to be altered. Larger values are suited to flatter terrain keeping the normal vectors more vertical while smaller values are more suitable to more hilly terrain. This can be dynamically adjusted based on the height differentials but it isn't super necessary for this kind of simulation so I'll leave the value at 2 for now. It is also worth noting this seems more noticeable in the dimly lit scene I currently have, when increasing the light I found it much harder to notice.

![triangle mesh light](./resources/triangle_mesh_light.png "Triangle List Mesh Lighter")

With some tweaking of the noise generation, some ambient and directional light I was able to get a nice wavy terrain using the following config:

size: 5000x5000, x and y point count
scale: 5000.0, maximum amplitude
octaves: 8, number of octaves of noise
persistence: 0.5, decay constant of the octave noise

```rust
let size = 5000;
let octaves = 8;
let persistence = 0.5;

let width = size;
let height = size;
let scale = size as f32;
```

and light config:

```rust
fn setup_ambient_light(mut ambient_light: ResMut<AmbientLight>) {
    ambient_light.brightness = 200.0;
}

fn setup_lights(mut commands: Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 30_000_000_000.0,
            range: 10_000.0,
            radius: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(500.0, 1000.0, 500.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });
}
```

## Improving Terrain Generation

I started to do some planning for the next stages of the development and what I actually want this project to turn into now that I have an extremely simple example running. The goal for me was to create an environment simulator. The main features being:

- terrain generation including material types such as sand, dirt and stone.
- cloud generation and movement
- terrain dynamics via water or wind erosion

To begin though I would like to start by improving some of the terrain generation. The octave perlin noise looks a little unrealistic and overly noisy. Some methods for improving this lie in hydraulic erosion, however there are some techniques to generate slightly more realistic terrain. As a start however, I needed to at least be able to alter the terrain while the application is running. I implimented an update function for the terrain, storing relevant world data on a component struct.

```rust
#[derive(Component, Debug)]
struct Terrain {
    size: usize,
    octaves: i32,
    persistence: f32,
    permutation: Vec<i32>,
    mesh_handle: Handle<Mesh>,
}
```

The initial perlin noise I was generating used a function to generate 3D perlin noise with the z value always set to 0. Changing the call to be implimented in the update function means I could scale the z value by the elapsed time in seconds to visualise changes in the mesh. I encountered a bug that, after some time, caused the terrain to vanish and strange flashes and lines to appear. Printing some values showed that the output of my perlin noise function exploded when one axis inputs to

```rust
pub fn octave_perlin3d(
    x: f32,
    y: f32,
    z: f32,
    octaves: i32,
    persistence: f32,
    permutation: &Vec<i32>,
) -> f32
```

the `octave_perlin3d` function increased above some threshold. The threshold itself, for my initial settings was 2.0. Diving deeper I discovered that this value changed as the octaves changed, due to the adjusted x, y and z values of the perlin3d function being above 255. It came down to using the _x, _y and _z values as shown below.

```rust
fn perlin3d(x: f32, y: f32, z: f32, p: &Vec<i32>) -> f32 {
    let _x = x.floor() as usize & 255;
    let _y = y.floor() as usize & 255;
    let _z = z.floor() as usize & 255;

    // let xf: f32 = _x - x.floor() as f32; // incorrect
    // let yf: f32 = _y - y.floor() as f32; // incorrect
    // let zf: f32 = _z - z.floor() as f32; // incorrect

    let xf: f32 = x - x.floor() as f32;
    let yf: f32 = y - y.floor() as f32;
    let zf: f32 = z - z.floor() as f32;

    // ...
}
```

Fixing this meant that the perlin noise could be generated correctly. To assist with some performance I added some code to run the noise generation in paralell that I used in another project.

```rust
heightmap
    .map
    .par_iter_mut()
    .enumerate()
    .for_each(|(i, row)| {
        row.iter_mut().enumerate().for_each(|(j, elem)| {
            *elem = noise::octave_perlin3d(
                i as f32 / height as f32,
                j as f32 / width as f32,
                z,
                terrain.octaves,
                terrain.persistence,
                &terrain.permutation,
            ) as f32
                * scale;
        });
    });
```

The result looks a little weird, I find the slower terrain to almost be slightly intoxicating.
![fast changing terrain](./resources/1000x1000_fast.gif "1000x1000 Fast Rate")

![slow changing terrain](./resources/1000x1000_slow.gif "1000x1000 Slow Rate")
