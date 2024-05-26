# Terrain Modelling

## Introduction

I have been looking for some more simulation style projects to attempt. Recently I have been really enjoying some of the virtual environment apps. One I use all the time while working or studying is [Virtual Cottage](https://store.steampowered.com/app/1369320/Virtual_Cottage/). For some time I have been wanting to attempt a terrain simulation project and I thought a terrain simulation would work quite well for something like this.

### Resources

I have to acknowledge [Sebastian Lague](https://www.youtube.com/@SebastianLague) and the videos made on procedural terrain generation.

## Implementation

I will be using Rust and [bevy](https://bevyengine.org/) for this project. There were several candidates when choosing something render these models. I ranged from using something like Unity or Godot. I was also considering using a library to use Vulkan or OpenGL directly in either C/C++ or Rust although I thought this would be a bit too much work for what I was looking for at this stage.

### Noise

To generate terrain I've seen a handlful of different algorithms but Perlin noise seems to pop up everwhere so I'll try give that a go too. There are so many blog posts on generating Perlin noise. I found a lot of the content muddied the waters and usually took massive steps that left me scratching my head a little from time to time.

I found some sources that discuss the algorithm in a much nicer approach that make it very easy to understand without reading the original source material. I found this material from [Matt Zucker](https://mzucker.github.io/html/perlin-noise-math-faq.html) very useful in addition to [this](https://www.cs.umd.edu/class/spring2018/cmsc425/Lects/lect13-2d-perlin.pdf) lecture on perlin noise. Combined with [Ken Perlin](https://mrl.cs.nyu.edu/~perlin/noise/)'s code it was fairly simple to get something to compile. I had also seen a [blog](http://riven8192.blogspot.com/2010/08/calculate-perlinnoise-twice-as-fast.html) post several times discussing a faster implementation of the grad function. It seems to behave the same so far so I'll continue to run with that.

I had to do a bit of shuffling around, resolving the typical incorrect sign issues and so forth but in the end things looked okay. At the end of the day it was far easier to follow Ken Perlin's implementation directly, converting to Rust in this case of course, rather than interpreting some other blog. To visualise the data I used a PointList.

![point_mesh](./resources/point_mesh.png "Point List Mesh")

### Meshing

I found it somewhat hard to find solid information on meshing. Some of the new Bevy documentation does a reasonable job, seen in [custom meshing](https://bevyengine.org/examples/3D%20Rendering/generate-custom-mesh/). Initially I used verticle only normals, trying to get the lighting (both ambient and directional) working with this setup led to some interesting looking shadow behaviour.

To get rid of these stange shadows I changed the normal vector calculation to an answer from [stack overflow](https://stackoverflow.com/questions/6656358/calculating-normals-in-a-triangle-mesh). The calculation simplifies to calculating the height differences between the adjacent row and column points. Tuning the weighing value of 2.0 can allow for more, or less, contribution from the height variation. With some tweaking of the noise generation, some ambient and directional light I was able to get a nice wavy terrain using the following config:

```rust
HeightMap::new(1000, 1000, 1000.0, 8, 0.5);
```

and light config:

```rust
fn setup_ambient_light(mut ambient_light: ResMut<AmbientLight>) {
    ambient_light.brightness = 100.0;
}

fn setup_lights(mut commands: Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 20_00_000_000.0,
            range: 10_000.0,
            radius: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(500.0, 500.0, 0.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });
}
```

![triangle_mesh](./resources/triangle_mesh.png "Triangle List Mesh")

There are still some issues with the lighting which become apparent on both very smooth and very hilly terrain. The smooth surfaces become very shiny and the sharper terrain becomes dull. This is due to the y scale factor in the normal vector calculation.

```rust
let normal = Vector3::new(height_diff_x, 2.0, height_diff_z).normalize();
```

Typically a value of 2 is suitable but in these extremes would need to be altered. Larger values are suited to flatter terrain keeping the normal vectors more vertical while smaller values are more suitable to more hilly terrain. This can be dynamically adjusted but isn't super necessary for this kind of simulation so I'll leave the value at 2 for now. It is also worth noting this seems more noticeable in the dimly lit scene I currently have.

![triangle mesh light](./resources/triangle_mesh_light.png "Triangle List Mesh Lighter")
