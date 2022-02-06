use std::{collections::HashMap, env, f32::consts, path::Path};

use cgmath::prelude::*;
use game::Coordinate;
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    platform::windows::{IconExtWindows, WindowBuilderExtWindows},
    window::{Icon, Window, WindowBuilder},
};

use model::{Model, Vertex};

mod camera;
mod game;
mod instance;
mod light;
mod model;
mod texture;

mod ai;

use crate::game::{get_board_coords, GAME_PIECES_NAMES};

//* Refer to model module
// #[repr(C)]
// #[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// struct Vertex {
//     position: [f32; 3],
//     color: [f32; 3],
// }

// impl Vertex {
//     fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
//         wgpu::VertexBufferLayout {
//             array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Vertex,
//             // attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
//             attributes: &[
//                 wgpu::VertexAttribute {
//                     offset: 0,
//                     shader_location: 0,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
//                     shader_location: 1,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//             ],
//         }
//     }
// }

//* Refer to model module
// #[repr(C)]
// #[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// struct TextureVertex {
//     position: [f32; 3],
//     tex_coords: [f32; 2],
// }

// impl TextureVertex {
//     fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
//         wgpu::VertexBufferLayout {
//             array_stride: std::mem::size_of::<TextureVertex>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Vertex,
//             // attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
//             attributes: &[
//                 wgpu::VertexAttribute {
//                     offset: 0,
//                     shader_location: 0,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
//                     shader_location: 1,
//                     format: wgpu::VertexFormat::Float32x2,
//                 },
//             ],
//         }
//     }
// }

// const VERTICES: &[TextureVertex] = &[
//     TextureVertex {
//         position: [-0.0868241, 0.49240386, 0.0],
//         tex_coords: [0.4131759, 0.00759614],
//     }, // A
//     TextureVertex {
//         position: [-0.49513406, 0.06958647, 0.0],
//         tex_coords: [0.0048659444, 0.43041354],
//     }, // B
//     TextureVertex {
//         position: [-0.21918549, -0.44939706, 0.0],
//         tex_coords: [0.28081453, 0.949397],
//     }, // C
//     TextureVertex {
//         position: [0.35966998, -0.3473291, 0.0],
//         tex_coords: [0.85967, 0.84732914],
//     }, // D
//     TextureVertex {
//         position: [0.44147372, 0.2347359, 0.0],
//         tex_coords: [0.9414737, 0.2652641],
//     }, // E
// ];

// const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, /* padding */ 0];

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
    vertex_entry_point: &str,
    fragment_entry_point: &str,
    with_cull_mode: bool,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(&shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"), // can be put to parameters as well
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: vertex_entry_point,
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: fragment_entry_point,
            targets: &[wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: if with_cull_mode == true {
                Some(wgpu::Face::Back)
            } else {
                None
            },
            polygon_mode: wgpu::PolygonMode::Fill,
            clamp_depth: false,
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    })
}

struct State {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    arrow_circle_render_pipeline: wgpu::RenderPipeline,
    custom_render_pipeline: wgpu::RenderPipeline,
    light_render_pipeline: wgpu::RenderPipeline,
    // render_texture_pipeline: wgpu::RenderPipeline,
    // vertex_buffer: wgpu::Buffer,
    // index_buffer: wgpu::Buffer,
    // num_indices: u32,
    // diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
    camera_specs: camera::Initializer,
    light_specs: light::Initializer,
    instances: Vec<instance::Instance>,
    instance_buffer: wgpu::Buffer,
    depth_texture: texture::Texture,
    obj_model: Model,
    //
    // challenge_render_pipeline: wgpu::RenderPipeline,
    use_color: bool,
    //
    // challenge_vertex_buffer: wgpu::Buffer,
    // challenge_index_buffer: wgpu::Buffer,
    // num_challenge_indices: u32,
    use_complex: bool,
    board_model: Model,
    board_coords: Vec<(Coordinate, [f32; 3])>,
    game_pieces: HashMap<&'static str, (Model, wgpu::Buffer, [f32; 3])>,
    game_piece_initial_instance_data: instance::InstanceRaw,
    arrow_model: Model,
    arrow_instances_data: HashMap<&'static str, instance::InstanceRaw>,
    arrow_instance_buffer: wgpu::Buffer,
    circle_model: Model,
    circle_instances_data: HashMap<(i8, i8), instance::InstanceRaw>,
    circle_instance_buffer: wgpu::Buffer,
    game: game::Game,
    game_level: usize,
    custom_material: model::Material,
    mouse_pressed: bool,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        // let clear_color = wgpu::Color::BLACK;
        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        //* Challenge shader

        // let challenge_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        //     label: Some("Challenge Shader"),
        //     source: wgpu::ShaderSource::Wgsl(include_str!("challenge.wgsl").into()),
        // });

        // let challenge_shader =
        //     device.create_shader_module(&wgpu::include_wgsl!("shader.wgsl" /*"challenge.wgsl"*/));

        // let challenge_render_pipeline =
        //     device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //         label: Some("Render Pipeline"),
        //         layout: Some(&render_pipeline_layout),
        //         vertex: wgpu::VertexState {
        //             module: &challenge_shader,
        //             entry_point: "vs_main",
        //             buffers: &[Vertex::desc()],
        //         },
        //         fragment: Some(wgpu::FragmentState {
        //             module: &challenge_shader,
        //             entry_point: "fs_main",
        //             targets: &[wgpu::ColorTargetState {
        //                 format: config.format,
        //                 blend: Some(wgpu::BlendState::REPLACE),
        //                 write_mask: wgpu::ColorWrites::ALL,
        //             }],
        //         }),
        //         primitive: wgpu::PrimitiveState {
        //             topology: wgpu::PrimitiveTopology::TriangleList,
        //             strip_index_format: None,
        //             front_face: wgpu::FrontFace::Cw,
        //             cull_mode: Some(wgpu::Face::Back),
        //             polygon_mode: wgpu::PolygonMode::Fill,
        //             ..Default::default()
        //         },
        //         depth_stencil: None,
        //         multisample: wgpu::MultisampleState {
        //             count: 1,
        //             mask: !0,
        //             alpha_to_coverage_enabled: false,
        //         },
        //     });

        let use_color = true;

        // let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Vertex Buffer"),
        //     contents: bytemuck::cast_slice(VERTICES),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });

        // let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Index Buffer"),
        //     contents: bytemuck::cast_slice(INDICES),
        //     usage: wgpu::BufferUsages::INDEX,
        // });

        // let num_indices = INDICES.len() as u32;

        //* Challenge: Draw Circle
        // let mut challenge_verts = Vec::<TextureVertex>::new();
        // let num_vertices = 50;
        // let incr = consts::TAU / num_vertices as f32;
        // let radius: f32 = 0.5;
        // let mut rad: f32 = 0.0;
        // let mut tex_coords = (1.0, 0.5);
        // let tex_incr = 0.05; // incr;
        //                      // TAU => 2 * PI
        // while rad < consts::TAU {
        //     challenge_verts.push(TextureVertex {
        //         position: [radius * rad.cos(), radius * rad.sin(), 0.0],
        //         tex_coords: [
        //             if tex_coords.0 < 0.0 {
        //                 -tex_coords.0
        //             } else {
        //                 tex_coords.0
        //             },
        //             if tex_coords.1 < 0.0 {
        //                 -tex_coords.1
        //             } else {
        //                 tex_coords.1
        //             },
        //         ],
        //     });
        //     // println!("x: {}, y: {}", tex_coords.0, tex_coords.1);
        //     tex_coords.0 -= tex_incr;
        //     tex_coords.1 -= tex_incr;
        //     rad += incr;
        // }

        // let num_triangles = num_vertices - 2;
        // let challenge_indices = (1u16..num_triangles + 1)
        //     .into_iter()
        //     .flat_map(|i| vec![0, i, i + 1])
        //     .collect::<Vec<_>>();

        // let num_challenge_indices = challenge_indices.len() as u32;

        // let challenge_vertex_buffer =
        //     device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //         label: Some("Challenge Vertex Buffer"),
        //         contents: bytemuck::cast_slice(&challenge_verts),
        //         usage: wgpu::BufferUsages::VERTEX,
        //     });
        // let challenge_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Challenge Index Buffer"),
        //     contents: bytemuck::cast_slice(&challenge_indices),
        //     usage: wgpu::BufferUsages::INDEX,
        // });

        let use_complex = true;

        //* Texture

        //* At Compile Time - with macro */
        let diffuse_bytes = include_bytes!("../assets/happy-tree.png");
        let diffuse_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            diffuse_bytes,
            Some("Happy Tree Texture"),
            true,
        )
        .unwrap();

        //* At Runtime - with load*/
        let assets_dir = env::current_dir().unwrap().join("assets");
        // let happy_tree_image_path = assets_dir.join("happy-tree.png");

        // let diffuse_texture =
        //     texture::Texture::load(&device, &queue, happy_tree_image_path).unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    // *Diffuse
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                    // *Normal
                    // wgpu::BindGroupLayoutEntry {
                    //     binding: 2,
                    //     visibility: wgpu::ShaderStages::FRAGMENT,
                    //     ty: wgpu::BindingType::Texture {
                    //         multisampled: false,
                    //         view_dimension: wgpu::TextureViewDimension::D2,
                    //         sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    //     },
                    //     count: None,
                    // },
                    // wgpu::BindGroupLayoutEntry {
                    //     binding: 3,
                    //     visibility: wgpu::ShaderStages::FRAGMENT,
                    //     ty: wgpu::BindingType::Sampler {
                    //         comparison: false,
                    //         filtering: true,
                    //     },
                    //     count: None,
                    // },
                ],
            });

        // let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("Diffuse Bind Group"),
        //     layout: &texture_bind_group_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
        //         },
        //     ],
        // });

        //* Model
        let models_dir = assets_dir.join("models");
        let obj_model = model::Model {
            materials: vec![],
            meshes: vec![],
        };
        // let obj_model = model::Model::load(
        //     &device,
        //     &queue,
        //     &texture_bind_group_layout,
        //     models_dir.join("cube").join("cube.obj"),
        //     true,
        // )
        // .unwrap();

        //* Camera
        let camera_specs = camera::init(&device, &config);

        //* Light
        let light_specs = light::init(&device);

        //* Instances
        const NUM_INSTANCES_PER_ROW: u32 = 1;
        const NUM_INSTANCES: u32 = NUM_INSTANCES_PER_ROW * NUM_INSTANCES_PER_ROW;
        const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
            NUM_INSTANCES_PER_ROW as f32 * 0.5,
            0.0,
            NUM_INSTANCES_PER_ROW as f32 * 0.5,
        );

        const SPACE_BETWEEN: f32 = 3.0;

        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = cgmath::vec3(0.0, 0.0, 0.0);
                    //  {
                    //     x: SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0),
                    //     y: 0.0,
                    //     z: SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0),
                    // };

                    let rotation = if position.is_zero() {
                        //* this is needed so an object at (0, 0, 0) won't get scaled to zero
                        //* as Quaternions can effect scale if they're not created correctly
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(
                            position.normalize(),
                            cgmath::Deg(/*45.0*/ 0.0),
                        )
                    };

                    instance::Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances
            .iter()
            .map(instance::Instance::to_raw)
            .collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        //* Depth Buffer | Depth Texture
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, Some("Depth Texture"));

        //* Default Pipeline Layout
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_specs.bind_group_layout,
                    &light_specs.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        // let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //     label: Some("Render Pipeline"),
        //     layout: Some(&render_pipeline_layout),
        //     vertex: wgpu::VertexState {
        //         module: &shader,
        //         entry_point: "vs_main",
        //         buffers: &[model::ModelTextureVertex::desc(), InstanceRaw::desc()],
        //     },
        //     fragment: Some(wgpu::FragmentState {
        //         module: &shader,
        //         entry_point: "fs_main",
        //         targets: &[wgpu::ColorTargetState {
        //             format: config.format,
        //             blend: Some(wgpu::BlendState::REPLACE),
        //             write_mask: wgpu::ColorWrites::ALL,
        //         }],
        //     }),
        //     primitive: wgpu::PrimitiveState {
        //         topology: wgpu::PrimitiveTopology::TriangleList,
        //         strip_index_format: None,
        //         front_face: wgpu::FrontFace::Ccw,
        //         cull_mode: Some(wgpu::Face::Back),
        //         polygon_mode: wgpu::PolygonMode::Fill,
        //         clamp_depth: false,
        //         conservative: false,
        //     },
        //     depth_stencil: Some(wgpu::DepthStencilState {
        //         format: texture::Texture::DEPTH_FORMAT,
        //         depth_write_enabled: true,
        //         depth_compare: wgpu::CompareFunction::Less,
        //         stencil: wgpu::StencilState::default(),
        //         bias: wgpu::DepthBiasState::default(),
        //     }),
        //     multisample: wgpu::MultisampleState {
        //         count: 1,
        //         mask: !0,
        //         alpha_to_coverage_enabled: false,
        //     },
        // });

        // *Default Pipeline
        let render_pipeline = {
            //* Shader
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Texture Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("texture.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[
                    model::ModelTextureVertex::desc(),
                    instance::InstanceRaw::desc(),
                ],
                shader,
                "vs_main",
                "fs_main",
                true,
            )
        };

        // *Arrow Pipeline
        let arrow_circle_render_pipeline = {
            //* Shader
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Arrow Texture Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("texture.wgsl").into()),
            };

            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[
                    model::ModelTextureVertex::desc(),
                    instance::InstanceRaw::desc(),
                ],
                shader,
                "vs_main",
                "fs_arrow_main",
                false,
            )
        };

        // *Pipeline for Object not interacting with camera and light
        let custom_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Custom Pipeline Layout"),
                bind_group_layouts: &[&camera_specs.bind_group_layout],
                push_constant_ranges: &[],
            });

            //* Shader
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Custom Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("custom.wgsl").into()),
            };

            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), instance::InstanceRaw::desc()],
                shader,
                "vs_main",
                "fs_main",
                true,
            )
        };

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[
                    // &texture_bind_group_layout,
                    &camera_specs.bind_group_layout,
                    &light_specs.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            };

            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelTextureVertex::desc()],
                shader,
                "vs_main",
                "fs_main",
                true,
            )
        };

        // * Custom Practice

        let mut game_pieces = HashMap::new();
        let mut arrow_instances_data = HashMap::new();

        // let mut game_piece_instances_data = HashMap::new();
        // let mut game_piece_instance_buffers = HashMap::new();

        let board_model = model::Model::load(
            &device,
            &queue,
            &texture_bind_group_layout,
            models_dir.join("quarto").join("board.obj"),
            true,
        )
        .unwrap();

        let arrow_model = model::Model::load(
            &device,
            &queue,
            &texture_bind_group_layout,
            models_dir.join("quarto").join("Arrow.obj"),
            true,
        )
        .unwrap();

        let game_piece_pos;
        let game_piece_rot;

        game_piece_pos = cgmath::Vector3::from([0.0, 0.0, 0.0]);

        game_piece_rot =
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0));

        let game_piece_initial_instance_data = instance::Instance {
            position: game_piece_pos,
            rotation: game_piece_rot,
        }
        .to_raw();

        for (model_name, arrow_point, board_point) in GAME_PIECES_NAMES {
            let game_piece = model::Model::load(
                &device,
                &queue,
                &texture_bind_group_layout,
                models_dir
                    .join("quarto")
                    .join("game_pieces")
                    .join(model_name.to_string() + ".obj"),
                true,
            )
            .unwrap();

            let game_piece_instance_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Game Piece Instance Buffer"),
                    contents: bytemuck::cast_slice(&[game_piece_initial_instance_data]),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

            game_pieces.insert(
                model_name,
                (game_piece, game_piece_instance_buffer, board_point),
            );

            let arrow_position;
            let arrow_rotation;

            // if model_name.contains("Tall") {
            arrow_position = cgmath::Vector3::from(arrow_point);

            arrow_rotation = if arrow_position.is_zero() {
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
            } else {
                cgmath::Quaternion::from_axis_angle(arrow_position.normalize(), cgmath::Deg(0.0))
            };

            let arrow_instance_data = instance::Instance {
                position: arrow_position,
                rotation: arrow_rotation,
            }
            .to_raw();

            arrow_instances_data.insert(model_name, arrow_instance_data);
        }

        let arrow_instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&[*arrow_instances_data
                .get(&GAME_PIECES_NAMES[0].0)
                .unwrap()]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let circle_model = model::Model::load(
            &device,
            &queue,
            &texture_bind_group_layout,
            models_dir.join("quarto").join("Circle.obj"),
            true,
        )
        .unwrap();

        let mut circle_instances_data = HashMap::new();

        let board_coords = get_board_coords().to_vec();
        for (coor, circle_point) in board_coords.iter() {
            let position;
            let rotation;

            position = cgmath::Vector3::from(*circle_point);

            rotation = if position.is_zero() {
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
            } else {
                cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(0.0))
            };

            let circle_instance_data = instance::Instance { position, rotation }.to_raw();

            circle_instances_data.insert((coor.row, coor.col), circle_instance_data);
        }

        // println!("{:?}", circle_instances_data.get(&(0, 0)).unwrap());

        let circle_instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&[*circle_instances_data.get(&(0, 0)).unwrap()]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let game_level = 1;
        let game = game::Game::init(game_level);

        let custom_material = {
            let diffuse_bytes =
                include_bytes!("../assets/models/quarto/textures/Wood_Bamboo_Medium.jpg");
            // include_bytes!("C:\\Users\\Nicat\\Downloads\\wood_texture.jpg");

            let diffuse_texture = texture::Texture::from_bytes(
                &device,
                &queue,
                diffuse_bytes,
                Some("assets/cobble-diffuse.png"),
                false,
            )
            .unwrap();
            // let normal_texture = texture::Texture::from_bytes(
            //     &device,
            //     &queue,
            //     normal_bytes,
            //     Some("assets/cobble-normal.png"),
            //     true,
            // )
            // .unwrap();

            model::Material::new(
                &device,
                "alt-material",
                diffuse_texture,
                &texture_bind_group_layout,
            )
        };

        Self {
            instance,
            adapter,
            surface,
            device,
            queue,
            config,
            size,
            clear_color,
            render_pipeline,
            arrow_circle_render_pipeline,
            custom_render_pipeline,
            light_render_pipeline,
            // render_texture_pipeline,
            // vertex_buffer,
            // index_buffer,
            // num_indices,
            // diffuse_bind_group,
            diffuse_texture,
            camera_specs,
            light_specs,
            instances,
            instance_buffer,
            depth_texture,
            obj_model,
            //
            // challenge_render_pipeline,
            use_color,
            //
            // challenge_vertex_buffer,
            // challenge_index_buffer,
            // num_challenge_indices,
            use_complex,
            //
            board_model,
            board_coords,
            game_pieces,
            game_piece_initial_instance_data,
            arrow_model,
            arrow_instances_data,
            arrow_instance_buffer,
            circle_model,
            circle_instances_data,
            circle_instance_buffer,
            game,
            game_level,
            custom_material,
            mouse_pressed: false,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.camera_specs
                .projection
                .resize(new_size.width, new_size.height);

            self.depth_texture = texture::Texture::create_depth_texture(
                &self.device,
                &self.config,
                Some("Depth Texture"),
            );
        }
    }

    fn input(&mut self, event: &DeviceEvent) -> bool {
        match event {
            // WindowEvent::CursorMoved { position, .. } => {
            //     self.clear_color = wgpu::Color {
            //         r: position.x as f64 / self.size.width as f64,
            //         g: position.y as f64 / self.size.height as f64,
            //         b: 1.0,
            //         a: 1.0,
            //     };
            //     true
            // },
            DeviceEvent::Key(KeyboardInput {
                state,
                virtual_keycode: Some(key),
                ..
            }) => {
                let game_keyboard_processed = if !self.game.ended {
                    self.game.process_keyboard(*key, *state)
                } else {
                    false
                };

                if game_keyboard_processed {
                    self.game.update(
                        &self.queue,
                        &self.arrow_instances_data,
                        &self.arrow_instance_buffer,
                        &self.circle_instances_data,
                        &self.circle_instance_buffer,
                        &self.game_pieces,
                        &self.board_coords,
                    );
                }
                self.camera_specs.controller.process_keyboard(*key, *state)
                    || game_keyboard_processed
            }
            DeviceEvent::MouseWheel { delta, .. } => {
                self.camera_specs.controller.process_scroll(delta);
                true
            }
            DeviceEvent::Button {
                button: 1, // left mouse button
                state,
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            DeviceEvent::MouseMotion { delta } => {
                if self.mouse_pressed {
                    self.camera_specs.controller.process_mouse(delta.0, delta.1);
                }
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        self.camera_specs.update(&self.queue, dt);
        self.light_specs.update(&self.queue, dt);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                // This is what [[location(0)]] in the fragment shader targets
                wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                },
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        // render_pass.set_pipeline(if self.use_complex {
        //     &self.render_pipeline
        // } else {
        //     &self.challenge_render_pipeline
        // });

        // let data = if self.use_complex {
        //     (
        //         &self.challenge_vertex_buffer,
        //         &self.challenge_index_buffer,
        //         self.num_challenge_indices,
        //     )
        // } else {
        //     (&self.vertex_buffer, &self.index_buffer, self.num_indices)
        // };

        // render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        // render_pass.set_vertex_buffer(0, data.0.slice(..));

        //* Set Instance Buffer */
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        // // *Camera Bind Group
        // render_pass.set_bind_group(1, &self.camera_specs.bind_group, &[]);
        // // *Light Bind Group
        // render_pass.set_bind_group(2, &self.light_specs.bind_group, &[]);

        // render_pass.set_index_buffer(data.1.slice(..), wgpu::IndexFormat::Uint16);

        // render_pass.draw(0..self.num_vertices, 0..1);
        // render_pass.draw_indexed(0..data.2, 0, 0..1);
        // render_pass.draw_indexed(0..data.2, 0, 0..self.instances.len() as _);

        use model::DrawLight;
        render_pass.set_pipeline(&self.light_render_pipeline);
        render_pass.draw_light_model(
            &self.obj_model,
            &self.camera_specs.bind_group,
            &self.light_specs.bind_group,
        );

        use model::DrawModel;
        render_pass.set_pipeline(&self.render_pipeline);
        // * Usefull for manual mesh & material assigning
        // let mesh = &self.obj_model.meshes[0];
        // let material = &self.obj_model.materials[mesh.material];
        // render_pass.draw_mesh_instanced(
        //     mesh,
        //     material,
        //     0..self.instances.len() as u32,
        //     &self.camera_specs.bind_group,
        //     &self.light_specs.bind_group,
        // );

        // render_pass.draw_model_instanced(
        //     &self.obj_model,
        //     0..self.instances.len() as u32,
        //     &self.camera_specs.bind_group,
        //     &self.light_specs.bind_group,
        // );

        //* Board model
        render_pass.draw_model_instanced(
            &self.board_model,
            // &self.custom_material,
            0..1, //self.instances.len() as u32,
            &self.camera_specs.bind_group,
            &self.light_specs.bind_group,
        );

        //* Game Piece models
        for (_, (game_piece, game_piece_instance_buffer, _)) in self.game_pieces.iter() {
            render_pass.set_vertex_buffer(1, game_piece_instance_buffer.slice(..));
            render_pass.draw_model_instanced(
                &game_piece,
                // &self.custom_material,
                0..1, //self.instances.len() as u32,
                &self.camera_specs.bind_group,
                &self.light_specs.bind_group,
            );
        }

        //* Arrow model
        render_pass.set_pipeline(&self.arrow_circle_render_pipeline);

        if !self.game.ended && self.game.available_pieces.len() > 0 {
            render_pass.set_vertex_buffer(1, self.arrow_instance_buffer.slice(..));
            render_pass.draw_model_instanced(
                &self.arrow_model,
                0..1, //self.instances.len() as u32,
                &self.camera_specs.bind_group,
                &self.light_specs.bind_group,
            );
        }
        //* Circle model
        if !self.game.ended && self.game.selected_coor.is_some() {
            render_pass.set_vertex_buffer(1, self.circle_instance_buffer.slice(..));
            render_pass.draw_model_instanced(
                &self.circle_model,
                0..1, //self.instances.len() as u32,
                &self.camera_specs.bind_group,
                &self.light_specs.bind_group,
            );
        }

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    //* Enabling logging
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Quarto")
        // .with_position(position)
        // .with_taskbar_icon(Some(
        //     IconExtWindows::from_path(std::path::PathBuf::from("assets/icon.png"), None).unwrap(),
        // ))
        // .with_window_icon(Some(
        //     Icon::from_path(
        //         std::path::Path::new("assets/images/icon.png"),
        //         Some(PhysicalSize {
        //             width: 96,
        //             height: 96,
        //         }),
        //     )
        //     .unwrap(),
        // ))
        .build(&event_loop)
        .unwrap();

    let mut state = pollster::block_on(State::new(&window));

    let mut last_render_time = std::time::Instant::now();

    let mut is_cursor_on_window = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::DeviceEvent { ref event, .. } => {
                if is_cursor_on_window {
                    state.input(event);
                }
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size);
                }
                WindowEvent::CursorEntered { .. } => {
                    is_cursor_on_window = true;
                }
                WindowEvent::CursorLeft { .. } => {
                    is_cursor_on_window = false;
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let delta_time = now - last_render_time;
                last_render_time = now;
                state.update(delta_time);

                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }

                if state.game.ended {
                    *control_flow = ControlFlow::Wait;
                    let mut input = String::new();
                    println!("Do you want to reset the game? (y/n): ");
                    std::io::stdin().read_line(&mut input).unwrap();

                    if input.chars().nth(0).is_some() {
                        let c = input.chars().nth(0).unwrap();
                        if c == 'y' || c == 'Y' {
                            *control_flow = ControlFlow::Poll;
                            println!("<<< Reseting The Game >>>");

                            for (_, (_, game_piece_instance_buffer, _)) in state.game_pieces.iter()
                            {
                                state.queue.write_buffer(
                                    &game_piece_instance_buffer,
                                    0,
                                    bytemuck::cast_slice(&[state.game_piece_initial_instance_data]),
                                );
                            }

                            state.queue.write_buffer(
                                &state.arrow_instance_buffer,
                                0,
                                bytemuck::cast_slice(&[*state
                                    .arrow_instances_data
                                    .get(&GAME_PIECES_NAMES[0].0)
                                    .unwrap()]),
                            );
                            state.game_level += 1;
                            state.game.reset(state.game_level);
                        } else if c == 'n' || c == 'N' {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
