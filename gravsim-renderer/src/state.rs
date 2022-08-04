use bytemuck::{Pod, Zeroable};
use gravsim_simulation::Simulation;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::mem::size_of;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    include_spirv, vertex_attr_array, Backends, BlendState, Buffer, BufferUsages, Color,
    ColorTargetState, ColorWrites, CommandEncoderDescriptor, Device, DeviceDescriptor, Face,
    Features, FragmentState, IndexFormat, Instance, Limits, LoadOp, Operations,
    PipelineLayoutDescriptor, PowerPreference, PresentMode, PrimitiveState, PushConstantRange,
    Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderStages, Surface, SurfaceConfiguration,
    SurfaceError, TextureUsages, TextureViewDescriptor, VertexAttribute, VertexBufferLayout,
    VertexState, VertexStepMode,
};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use winit::window::Window;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    pub const ATTRIBS: &'static [VertexAttribute] = &vertex_attr_array![0 => Float32x2];
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RenderInstance {
    position: [f32; 2],
    color: [f32; 3],
    radius: f32,
}

impl RenderInstance {
    pub const ATTRIBS: &'static [VertexAttribute] =
        &vertex_attr_array![1 => Float32x2, 2 => Float32x3, 3 => Float32];
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct PushConstants {
    inv_aspect: f32,
    render_scale: f32,
    pos: [f32; 2],
}

pub struct State {
    pub simulation: Simulation,

    pub size: PhysicalSize<u32>,
    pub surface: Surface,
    pub config: SurfaceConfiguration,
    pub device: Device,
    pub queue: Queue,

    pub render_pipeline: RenderPipeline,

    pub vertex_buffer: Buffer,
    pub instance_buffer: Buffer,
    pub index_buffer: Buffer,

    pub index_count: u32,

    pub push_constants: PushConstants,
    pub instances: Vec<RenderInstance>,
}

impl State {
    const VERTEX_COUNT: usize = 25;

    pub async fn new(window: &Window, simulation: Simulation) -> Self {
        let size = window.inner_size();

        let instance = Instance::new(Backends::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    features: Features::CONSERVATIVE_RASTERIZATION | Features::PUSH_CONSTANTS,
                    limits: Limits {
                        max_push_constant_size: size_of::<PushConstants>() as u32,
                        ..Default::default()
                    },
                },
                None,
            )
            .await
            .unwrap();

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let vert_shader = device.create_shader_module(include_spirv!("../shaders/vert.spv"));
        let frag_shader = device.create_shader_module(include_spirv!("../shaders/frag.spv"));

        let rp_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::VERTEX,
                range: 0..size_of::<PushConstants>() as u32,
            }],
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&rp_layout),
            vertex: VertexState {
                module: &vert_shader,
                entry_point: "main",
                buffers: &[
                    VertexBufferLayout {
                        array_stride: size_of::<Vertex>() as u64,
                        step_mode: VertexStepMode::Vertex,
                        attributes: Vertex::ATTRIBS,
                    },
                    VertexBufferLayout {
                        array_stride: size_of::<RenderInstance>() as u64,
                        step_mode: VertexStepMode::Instance,
                        attributes: RenderInstance::ATTRIBS,
                    },
                ],
            },
            primitive: PrimitiveState {
                cull_mode: Some(Face::Back),
                conservative: true,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &frag_shader,
                entry_point: "main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let vertices: Vec<_> = (0..Self::VERTEX_COUNT)
            .map(|i| i as f32 / Self::VERTEX_COUNT as f32 * std::f32::consts::TAU)
            .map(|a| Vertex {
                position: [-a.sin() * 0.5, a.cos() * 0.5],
            })
            .collect();
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let indices: Vec<_> = (2..Self::VERTEX_COUNT as u16 - 1)
            .flat_map(|i| [0, i, i + 1])
            .chain([0, 1, 2])
            .collect();
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        let instances: Vec<_> = simulation
            .stars
            .iter()
            .map(|star| RenderInstance {
                position: [star.pos().x, star.pos().y],
                color: star.color(&simulation.mass_dist),
                radius: star.radius(),
            })
            .collect();
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&instances),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let push_constants = PushConstants {
            inv_aspect: size.height as f32 / size.width as f32,
            render_scale: 1.0,
            pos: [0.0; 2],
        };

        Self {
            simulation,

            size,
            surface,
            config,
            device,
            queue,

            render_pipeline,

            vertex_buffer,
            index_buffer,
            instance_buffer,

            index_count: indices.len() as u32,

            push_constants,
            instances,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = self.size.width;
            self.config.height = self.size.height;

            self.push_constants.inv_aspect = self.config.height as f32 / self.config.width as f32;

            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        const STEP: f32 = 0.25;

        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(key),
                        ..
                    },
                ..
            } => match key {
                VirtualKeyCode::W | VirtualKeyCode::Up => {
                    self.push_constants.pos[1] -= STEP / self.push_constants.render_scale
                }
                VirtualKeyCode::A | VirtualKeyCode::Left => {
                    self.push_constants.pos[0] += STEP / self.push_constants.render_scale
                }
                VirtualKeyCode::S | VirtualKeyCode::Down => {
                    self.push_constants.pos[1] += STEP / self.push_constants.render_scale
                }
                VirtualKeyCode::D | VirtualKeyCode::Right => {
                    self.push_constants.pos[0] -= STEP / self.push_constants.render_scale
                }
                VirtualKeyCode::Return => {
                    self.push_constants.render_scale = 1.0;
                    self.push_constants.pos = [0.0; 2];
                }
                _ => return false,
            },
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                ..
            } => match y.total_cmp(&0.0) {
                Ordering::Greater => self.push_constants.render_scale *= 0.8,
                Ordering::Less => self.push_constants.render_scale *= 1.25,
                _ => return false,
            },
            _ => return false,
        }
        true
    }

    pub fn update(&mut self) {
        // update simulation state
        self.simulation.update();

        // update instance buffer
        self.instances
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, instance)| {
                let position = self.simulation.stars[i].pos();
                instance.position = [position.x, position.y];
            });
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let current_texture = self.surface.get_current_texture()?;
        let view = current_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut command_encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_push_constants(
                ShaderStages::VERTEX,
                0,
                bytemuck::bytes_of(&self.push_constants),
            );
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

            render_pass.draw_indexed(0..self.index_count, 0, 0..self.instances.len() as u32);
        }

        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances),
        );
        self.queue.submit(Some(command_encoder.finish()));

        current_texture.present();
        Ok(())
    }
}
