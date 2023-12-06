# fastlem

![terrain](https://github.com/TadaTeruki/fastlem/assets/69315285/03be5898-677d-411b-8f23-755b69cb1e07)

fastlem is a Rust library that provides methods for creating virtual terrains based on a simplified landscape evolution model (LEM). It generates terrain data with plausible reliefs by simulating the erosion process from a given initial topographical parameters. It helps you to create or generate realistic terrains for your creative projects. 

> [!WARNING]
> This project is now in development. During `0.1.*`, the interface may change a lot.

## Previews

**Simple Landscape Evolution**

```
$ cargo run --example landscape_evolution --release
```

![Simple Landscape Evolution](images/out/landscape_evolution.png)

**Simple Terrain Generation**

```
$ cargo run --example terrain_generation --release
```

![Simple Terrain Generation](images/out/terrain_generation.png)

**Advanced Terrain Generation**

```
$ cargo run --example terrain_generation_advanced --release
```

![Advanced Terrain Generation](images/out/terrain_generation_advanced.png)

**Terrain Generation from Given Parameters**

```
$ cargo run --example sample_terrain --release
```

![Terrain Generation from Given Parameters](images/out/sample_terrain.png)
