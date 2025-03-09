# quarto-wgpu-rust

A simple graphics project for Quarto board game - created through the integration of [WebGPU](https://gpuweb.github.io/gpuweb/) and Vulkan Graphics API in the [rust](https://www.rust-lang.org/) programming language. The project's goal is to produce a visually immersive computer adaptation of the board game Quarto, with online and offline gameplay modes versus human and AI opponents. The implementation makes use of the [wgpu](https://github.com/gfx-rs/wgpu) Rust graphics library, Blender for 3D modeling, and Firebase for data storage. The AI opponent is created using the minmax algorithm.

This project has been done following instructions from the wgpu tutorial: https://sotrh.github.io/learn-wgpu/

## Rendered View Samples

Front View | Side View 
-|-
![Screenshot_1.png](assets/images/rendered_view_samples/Screenshot_1.png) | ![Screenshot_1.png](assets/images/rendered_view_samples/Screenshot_2.png)

To find the full lists of view samples, go to the [rendered view samples folder](https://github.com/NSadikhov/quarto-wgpu-rust/tree/main/assets/images/rendered_view_samples).

## Getting Started

Clone and run from source:
```
git clone https://github.com/NSadikhov/quarto-wgpu-rust.git
```
```
cargo run
```


