use std::num::NonZero;

use anyhow::*;
use wgpu::{util::DeviceExt, Texture};
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointsVertex {
    position: [f32; 2],
    value: f32,
}

// lib.rs
impl PointsVertex {
    const VERTEX_ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32,
    ];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<PointsVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::VERTEX_ATTRIBUTES,
        }
    }
}

pub struct Heatmap {
    pub field: ScalarField,
    pub mesh: Mesh,
}

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
}

impl Vertex {
    const VERTEX_ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
    ];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::VERTEX_ATTRIBUTES,
        }
    }
}

fn compute_weights(sigma: f32) -> [f32; 8] {
    let mut weights = [0.0; 8];
    let mut sum = 0.0;

    for i in 0..8 {
        weights[i] = 1.0 / f32::sqrt(2.0 * std::f32::consts::PI * sigma * sigma)
            * f32::exp(-(i as f32) * i as f32 / (2.0 * sigma * sigma));
        sum += weights[i];
    }
    for i in 0..8 {
        weights[i] /= sum;
    }
    weights
}

impl Heatmap {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[f32],
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let (max, min) = data.iter().fold((f32::MIN, f32::MAX), |(max, min), v| {
            (f32::max(max, *v), f32::min(min, *v))
        });
        let points: Vec<[f32; 4]> = data
            .iter()
            .map(|v| {
                let ratio = (v - min) / (max - min);
                [ratio, 0.0, 1.0 - ratio, 1.0]
            })
            .collect();
        for p in points.iter().take(10) {
            println!("{:?}", p);
        }
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let colormap_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &colormap_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(points.as_slice()),
            wgpu::TexelCopyBufferLayout {
                offset: 1,
                bytes_per_row: Some(16 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        );

        let view = colormap_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // wgpu::BindGroupLayoutEntry {
                //     binding: 2,
                //     visibility: wgpu::ShaderStages::FRAGMENT,
                //     ty: wgpu::BindingType::Buffer {
                //         ty: wgpu::BufferBindingType::Storage { read_only: true },
                //         has_dynamic_offset: false,
                //         min_binding_size: None,
                //     },
                //     count: None,
                // },
                // wgpu::BindGroupLayoutEntry {
                //     binding: 3,
                //     visibility: wgpu::ShaderStages::FRAGMENT,
                //     ty: wgpu::BindingType::Buffer {
                //         ty: wgpu::BufferBindingType::Uniform,
                //         has_dynamic_offset: false,
                //         min_binding_size: None,
                //     },
                //     count: None,
                // },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                // wgpu::BindGroupEntry {
                //     binding: 3,
                //     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                //         buffer: &device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                //             label: Some("Scalar Field size"),
                //             contents: bytemuck::cast_slice(&[width as f32, height as f32]),
                //             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::STORAGE,
                //         }),
                //         offset: 0,
                //         size: Some(
                //             NonZero::new(2 * std::mem::size_of::<f32>() as wgpu::BufferAddress)
                //                 .unwrap(),
                //         ),
                //     }),
                // },
                // wgpu::BindGroupEntry {
                //     binding: 2,
                //     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                //         buffer: &device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                //             label: Some("Gaussian Kernel"),
                //             contents: bytemuck::cast_slice(&compute_weights(8.0)),
                //             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::STORAGE,
                //         }),
                //         offset: 0,
                //         size: Some(
                //             NonZero::new(8 * std::mem::size_of::<f32>() as wgpu::BufferAddress)
                //                 .unwrap(),
                //         ),
                //     }),
                // },
            ],
            label: Some("diffuse_bind_group"),
        });
        let field = ScalarField {
            texture: colormap_texture,
            bind_group_layout,
            bind_group,
        };
        let vertices = vec![
            Vertex {
                position: [-0.5, -0.5],
                tex_coord: [0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5],
                tex_coord: [1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5],
                tex_coord: [1.0, 1.0],
            },
            Vertex {
                position: [-0.5, 0.5],
                tex_coord: [0.0, 1.0],
            },
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Heatmap Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Heatmap Index Buffer"),
            contents: bytemuck::cast_slice(&[0u16, 1, 2, 0, 2, 3]),
            usage: wgpu::BufferUsages::INDEX,
        });
        Ok(Self {
            field,
            mesh: Mesh {
                vertex_buffer,
                index_buffer,
                num_indices: 6,
            },
        })
    }
}
pub struct ScalarField {
    pub texture: Texture,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

fn color_scale() -> Vec<[f32; 4]> {
    let mut start: [f32; 4] = [0.0, 0.0, 1., 1.];
    let step = 0.01176470588;
    (0..100)
        .map(|v| {
            if v >= 15 {
                start[0] += step;
            }

            start[2] = f32::max(0.0, start[2] - step);

            start.clone()
        })
        .collect()
}
