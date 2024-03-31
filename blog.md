# Terrain Modelling

## Introduction

I have been looking for some more simulation style projects to attempt. Recently I have been really enjoying some of the virtual environment apps. One I use all the time while working or studying is [Virtual Cottage](https://store.steampowered.com/app/1369320/Virtual_Cottage/). For some time I have been wanting to attempt a terrain simulation project and I thought a terrain simulation would work quite well for something like this.

### Resources

I have to acknowledge [Sebastian Lague](https://www.youtube.com/@SebastianLague) and the videos made on procedural terrain generation.

## Implementation

I will be using rust and [bevy](https://bevyengine.org/) for this project.

### Noise

To generate terrain I've seen a handlful of different algorithms but Perlin noise seems to pop up everwhere so I'll try give that a go too. There are so many blog posts on generating this dat yourself. I found a lot of the content to muddy the waters and usually take massive steps that left me scratching my head a little from time to time.

I found some sources that discuss the algorithm in a much nicer approach that make it very easy to understand without reading the original source material. I found this material from [Matt Zucker](https://mzucker.github.io/html/perlin-noise-math-faq.html) very useful in addition to [this](https://www.cs.umd.edu/class/spring2018/cmsc425/Lects/lect13-2d-perlin.pdf) lecture on perlin noise. Combined with [Ken Perlin](https://mrl.cs.nyu.edu/~perlin/noise/)'s code it was fairly simple to get something to compile. I had also seen a [blog](http://riven8192.blogspot.com/2010/08/calculate-perlinnoise-twice-as-fast.html) post several times discussing a faster implementation of the grad function. It seems to behave the same so far so I'll continue to run with that.

There was too much shuffling of code around, incorrect signs used. To get the visual working I used a PointList form to generate a basic visualisation of the mesh.

```
let mut mesh = Mesh::new(PrimitiveTopology::PointList);
mesh.set_indices(Some(bevy::render::mesh::Indices::U32(triangles)));
mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
```

![point_mesh](./resources/point_mesh.png "Point List Mesh")

### Meshing

As a starting point the [3D viewport to world](https://bevyengine.org/examples/3D%20Rendering/3d-viewport-to-world/) example was used as a project template.

Something I found quite difficult to find solid information on was meshing.
