# fastlem

![terrain](sample.webp)

[![Crates.io](https://img.shields.io/crates/v/fastlem)](https://crates.io/crates/fastlem)
[![Documentation](https://docs.rs/fastlem/badge.svg)](https://docs.rs/fastlem)

fastlem is a Rust library that provides methods for creating virtual terrains based on a simplified landscape evolution model (LEM). It creates terrain data with plausible reliefs by simulating the erosion process from a given initial topographical parameters. It helps you to create or generate realistic terrains for your creative projects. 

> [!WARNING]
> This project is now in development. During `0.1.*`, the interface may change a lot.

## Examples

|**Simple Landscape Evolution**|**Simple Terrain Generation**|
|:---:|:---:|
|![Simple Landscape Evolution](images/out/landscape_evolution.png)|![Simple Terrain Generation](images/out/terrain_generation.png)|
|`$ cargo run --example landscape_evolution --release`|`$ cargo run --example terrain_generation --release`|

|**Advanced Terrain Generation**|**Terrain Generation from Given Parameters**|
|:---:|:---:|
|![Advanced Terrain Generation](images/out/terrain_generation_advanced.png)|![Terrain Generation from Given Parameters](images/out/sample_terrain.png)|
|`$ cargo run --example terrain_generation_advanced --release`|`$ cargo run --example sample_terrain --release`|
