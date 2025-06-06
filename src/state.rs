use bytemuck::bytes_of;
use camera::CameraController;
use camera::CameraUniform;
use std::{default, iter, sync::Arc};
use wgpu::AddressMode;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BufferUsages, CommandEncoderDescriptor, FrontFace, PipelineCompilationOptions,
    RenderPassDescriptor, ShaderStages, TextureViewDescriptor,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::event::WindowEvent;
use winit::event_loop::DeviceEvents;
use winit::window::CursorIcon::Default;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[path = "shared_funcs/camera.rs"]
mod camera;
#[path = "shared_funcs/matrix_helpers.rs"]
mod matrix_helpers;
#[path = "shared_funcs/shared_funcs.rs"]
mod shared_funcs;

use cgmath::{num_traits::int, *};

use crate::lighting;
use crate::texture::Texture;
use crate::{Driver, vertex::Vertex};
const ANIMATION_SPEED: f32 = 0.5;
pub struct State<'a> {
    pub driver: Driver<'a>,

    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub view_matrix: Matrix4<f32>,
    pub project_mat: Matrix4<f32>,
    pub num_vertices: u32,
    pub camera: camera::Camera,
    depth_texture: wgpu::TextureView,
    pub camera_controller: camera::CameraController,
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    mouse_pressed: bool,
    texture: Texture,
    texture_bg: wgpu::BindGroup,
    vertex_uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl<'a> State<'a> {
    pub async fn new(
        window: Arc<Window>,
        mesh_data: &Vec<Vertex>,
        light_data: lighting::Light,
        img_path: &str,
        u_mode: AddressMode,
        v_mode: AddressMode,
    ) -> State<'a> {
        let driver = Driver::new(window).await;

        let img_texture =
            Texture::create_texture(&driver.device, &driver.queue, img_path, u_mode, v_mode)
                .expect("err in creating texture");

        let text_bgl = driver
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
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
                            0: wgpu::SamplerBindingType::Filtering,
                        },
                        count: None,
                    },
                ],
                label: Some("Texture Bind Group Layout"),
            });

        let texture_bg = driver.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &text_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&img_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&img_texture.sampler),
                },
            ],
            label: Some("Texture Bind Group"),
        });

        let shader_mod = driver
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shader mod"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sphere_texture.wgsl").into()),
            });

        let user_camera = camera::Camera::new((2.0, 3.0, -0.5), Deg(-60.0), Deg(180.0));
        let camera_pos: Point3<f32> = (3.0, 1.5, 1.0).into();
        let look_dir  = (0.0, 0.0, 0.0).into();
        let (view_matrix, projection_matrix, view_projection_matrix) =
            matrix_helpers::create_view_projection_matrix(
                camera_pos,
                look_dir,
                cgmath::Vector3::unit_y(),
                driver.config.width as f32 / driver.config.height as f32,
                false,
            );

        
        let vertex_uniform_buffer = driver.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Uniform Buffer"),
            size: 192,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // create fragment uniform buffer. here we set eye_position = camera_position and
        // light_position = eye_position
        let fragment_uniform_buffer = driver.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment Uniform Buffer"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let light_position: &[f32; 3] = &[4.0 as f32, 2.0 as f32, 5.0 as f32];
        let eye_position: &[f32; 3] =  camera_pos.as_ref();

        driver.queue.write_buffer(
            &fragment_uniform_buffer,
            0,
            bytemuck::cast_slice(light_position),
        );

        driver.queue.write_buffer(
            &fragment_uniform_buffer,
            16,
            bytemuck::cast_slice(eye_position),
        );

        let light_uniform_buffer = driver.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Uniform Buffer"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // light params
        driver.queue.write_buffer(
            &light_uniform_buffer,
            0,
            bytemuck::cast_slice(&[light_data]),
        );

        let uniform_bind_group_layout =
            driver
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                    label: Some("Uniform Bind Group Layout"),
                });

        let uniform_bind_group = driver.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: vertex_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: fragment_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: light_uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("Uniform Bind Group"),
        });

        let pipeline_layout =
            driver
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&uniform_bind_group_layout, &text_bgl],
                    push_constant_ranges: &[],
                });

        let pipeline = driver
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_mod,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc()],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_mod,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: driver.config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    cull_mode: None,
                    ..wgpu::PrimitiveState::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let vertex_buffer = driver
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(mesh_data),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let num_vertices = mesh_data.len() as u32;

        let camera_controller = CameraController::new(0.01, 0.05);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_project_matrix(&user_camera, projection_matrix);
        let camera_buffer = driver.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("1st camera buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        //group 0, binding 0
        let camera_bind_group_layout =
            driver
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    label: Some("Uniform Bind Group Layout"),
                });

        let camera_bind_group = driver.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        let depth_texture =
            shared_funcs::create_depth_texture(&driver.device, &driver.config, "depth_texture");

        Self {
            driver,
            pipeline,
            vertex_uniform_buffer,
            uniform_bind_group,
            vertex_buffer,
            project_mat: projection_matrix,
            num_vertices: num_vertices,
            camera: user_camera,
            camera_controller: camera_controller,
            camera_uniform: camera_uniform,
            camera_buffer: camera_buffer,
            camera_bind_group: camera_bind_group,
            view_matrix: view_matrix,
            mouse_pressed: false,
            depth_texture,
            texture: img_texture,
            texture_bg,
        }
    }

    // Accepts &mut self and new_size as parameters
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.driver.size = new_size;
            self.driver.config.width = new_size.width;
            self.driver.config.height = new_size.height;
            self.driver
                .surface
                .configure(&self.driver.device, &self.driver.config);
            self.project_mat = matrix_helpers::create_projection_matrix(
                new_size.width as f32 / new_size.height as f32,
                false,
            );
        }
    }
    pub fn input(&mut self, event: Event<()>) -> bool {

        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                if self.mouse_pressed {
                    self.camera_controller.process_mouse(delta.0, delta.1);
                    true
                } else {
                    false
                }
            }

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::MouseWheel { delta, .. } => {
                    self.camera_controller.process_scroll(&delta);
                    true
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                            state,
                            ..
                        },
                    ..
                } => {
                    println!("ARROW OR MOVEMENT KEY PRESSED");
                    self.camera_controller.key_handler(key_code, state);
                    true
                }

                _ => false,
            },

            _ => false,
        }
    }

    pub fn update(&mut self, delta: std::time::Duration) {
        let anim_diff = ANIMATION_SPEED * delta.as_secs_f32();
        let model_mat = matrix_helpers::create_transforms_matrix(
            [0.0, 0.0, 0.0],
            [30f32.to_radians(), 45f32.to_radians() * anim_diff, 0.0],
            [1.0, 1.0, 1.0],
        );

        let view_project_mat = self.project_mat * self.view_matrix;
        let norm_mat = (model_mat.invert().unwrap()).transpose();
        let model_ref: &[f32; 16] = model_mat.as_ref();
        let view_projection_ref: &[f32; 16] = view_project_mat.as_ref();
        let normal_ref: &[f32; 16] = norm_mat.as_ref();
        self.driver.queue.write_buffer(
            &self.vertex_uniform_buffer,
            0,
            bytemuck::cast_slice(model_ref),
        );
        self.driver.queue.write_buffer(
            &self.vertex_uniform_buffer,
            64,
            bytemuck::cast_slice(view_projection_ref),
        );
        self.driver.queue.write_buffer(
            &self.vertex_uniform_buffer,
            128,
            bytemuck::cast_slice(normal_ref),
        );
    }
    /*


    let view_projection_ref:&[f32; 16] = view_project_mat.as_ref();



     */

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.driver.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let depth_texture = self.driver.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.driver.config.width,
                height: self.driver.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[wgpu::TextureFormat::Depth24Plus],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .driver
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.2,
                            g: 0.247,
                            b: 0.314,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_bg, &[]);
            render_pass.draw(0..self.num_vertices, 0..1);
        }

        self.driver.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

// static viewpoint, removed for camera that can theoretically be static
/*
                // create viewing angle
                let camera = (1.5, 1.0, 3.0).into();
                let look_dir = (0.0, 0.0, 0.0).into();
                // orient the view so that the y-axis is "up"
                let up_dir: Vector3<f32> = cgmath::Vector3::unit_y();
                // create transformation matrix for rendered model
                let model_matrix = matrix_helpers::create_transforms_matrix(
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                    [1.0, 1.0, 1.0],
                );
                let aspect_ratio: f32 = driver.size.width as f32 / driver.size.height as f32;
                let (view_matrix, projection_matrix, view_projection_matrix) =
                    matrix_helpers::create_view_projection_matrix(
                        camera,
                        look_dir,
                        up_dir,
                        aspect_ratio,
                        false,
                    );

                //  model view projection matrix, done in the order proj * view * model
                let vpm_matrix = view_projection_matrix * model_matrix;

        let uniform_buf = driver.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("unif buffer"),
            contents: bytemuck::cast_slice(vpm_matrix.as_ref() as &[f32; 16]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let uniform_bg_layout =
            driver
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let uniform_bg = driver.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &uniform_bg_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });
        let pipeline_layout =
            driver
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&uniform_bg_layout],
                    push_constant_ranges: &[],
                });

        let pipeline = driver
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_mod,
                    entry_point: Some("vs_main"),
                    buffers: &[crate::vertex::Vertex::desc()],
                    compilation_options: PipelineCompilationOptions::default(),
                },

                fragment: Some(wgpu::FragmentState {
                    module: &shader_mod,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: driver.config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    strip_index_format: None,
                    ..wgpu::PrimitiveState::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
*/
