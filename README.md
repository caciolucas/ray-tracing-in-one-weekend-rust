# Ray Tracer in Rust

This is a simple ray tracer project implemented in Rust, capable of generating images by rendering spheres in a scene described using XML files.

## Features

- **Ray Tracing**: Utilizes ray tracing techniques to simulate the interaction of light with objects in a scene.
- **XML Scene Description**: Scenes are described using XML files, allowing for straightforward sphere placements and configurations.
- **Materials and Textures**: Supports various materials for realistic object appearances.
- **Output**: Generates rendered images depicting spheres with shading and lighting effects.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) installed on your system.

### Installation

Clone the repository:

```bash
git clone https://github.com/caciolucas/ray-tracing-in-one-weekend-rust.git
cd ray-tracing-in-one-weekend-rust
```

Build the project:

```bash
cargo build
```

### Usage

To generate a scene based on an XML file, execute:

```bash
cargo run
```

Then type the path for your XML scene file.


### Example XML Scene File

```xml
<RT>
    <film filename="path_to_new_img.ppm">
    <camera look_from="x y z" look_at="x y z" up="x y z" aperture="a"/>
    <world>
        <material type="material_type" center="x y z" />
        <object type="sphere" center="x y z" radius="r" />
    </world>
</RT>

```
