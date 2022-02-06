use std::time::Duration;

use bytemuck::{Pod, Zeroable};
use cgmath::Rotation3;
use wgpu::util::DeviceExt;

pub struct Initializer {
    pub uniform: LightUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    seed: u64,
}

impl Initializer {
    pub fn update(&mut self, queue: &wgpu::Queue, dt: Duration) {
        // let old_position: cgmath::Vector3<_> = self.uniform.position.into();

        // self.uniform.position = (cgmath::Quaternion::from_axis_angle(
        //     (0.0, 1.0, 0.0).into(),
        //     cgmath::Deg(60.0 * dt.as_secs_f32()),
        // ) * old_position)
        //     .into();

        // use oorandom;
        // self.seed += 1;
        // let mut rng = oorandom::Rand32::new(self.seed);
        // self.uniform.color = [rng.rand_float(), rng.rand_float(), rng.rand_float()];

        // queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}

pub fn init(device: &wgpu::Device) -> Initializer {
    let light_uniform = LightUniform {
        position: [5.0, 5.0, 5.0],
        _padding: 0,
        color: [1.0, 1.0, 1.0],
    };

    // *To be able to update lights position, we need to use COPY_DST
    let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Light VB"),
        contents: bytemuck::cast_slice(&[light_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let light_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Light Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Light Bind Group"),
        layout: &&light_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: light_buffer.as_entire_binding(),
        }],
    });

    Initializer {
        uniform: light_uniform,
        buffer: light_buffer,
        bind_group_layout: light_bind_group_layout,
        bind_group: light_bind_group,
        seed: 4,
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    // *Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    pub _padding: u32,
    pub color: [f32; 3],
}
