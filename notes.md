# Development Notes

## Notes

### Terrain Data Model

needs to be simple to re-mesh

- calculating normals and vetices quickly, to do this you need height of cell and nearby cells
    - need to update an array that contains vertices and normals to match bevy rendering
- code should be easily computed in parallel, can split cells into chunks? 16x16 or something similar
- need to call update on the mesh only when the mesh changes
    - only need to update the vertex and normal data on cells that change
    - keep track of updated cells, having to update a chunk seems pointless if the number of updated cells in a chunk is sparse, it is convenient for splitting into parallel ops though

### ?

## Reference Material

- [Nick's Blog](https://nickmcd.me/) - basically this project already implemented using custom engine in c++
- [Job Talle](https://jobtalle.com/index.html) - rendering and simulations, specifically [this](https://jobtalle.com/layered_voxel_rendering.html) page on voxel layering
- [Noise Wiki](https://www.campi3d.com/External/MariExtensionPack/userGuide5R4v1/Understandingsomebasicnoiseterms.html) - information on noises
- [Perlin Noise](https://mrl.cs.nyu.edu/~perlin/noise/) - Ken Perlin's noise implementation
- [RTIN Blog](http://clynamen.github.io/blog/2021/01/04/terrain_generation_bevy/) - more efficient triangle meshes
