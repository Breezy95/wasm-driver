use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use vertex::Vertex;
use web_sys::Element;
use wgpu::DeviceDescriptor;

use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};
pub mod state;
pub mod vertex;
#[path = "shared_funcs/lighting.rs"]
pub mod lighting;

#[path = "shared_funcs/texture.rs"]
pub mod texture;

pub struct Driver<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,
}
impl<'a> Driver<'a> {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let window_clone = window.clone();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window_clone).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                    memory_hints: Default::default(),
                },
                None, // Option<&std::path::Path> for trace_path, use None for no tracing
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window: window,
        }
    }
}

// https://rustwasm.github.io/wasm-bindgen/examples/performance.html
fn perf_to_system(amt: f64) -> Duration {
    let secs = (amt as u64) / 1_000;
    let nanos = (((amt as u64) % 1_000) as u32) * 1_000_000;
    Duration::new(secs, nanos)
}

pub fn run(mesh_data: &Vec<Vertex>, light_data: lighting::Light, img_name: &str, u_mode:wgpu::AddressMode, v_mode:wgpu::AddressMode) {
    let event_loop = EventLoop::new().expect("Failure to create event loop");

    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .build(&event_loop)
            .unwrap(),
    );
    window.set_title(&"sphere");
    let mut state = pollster::block_on(state::State::new(window.clone(), &mesh_data, light_data, img_name, u_mode, v_mode));
    #[cfg(target_arch = "wasm32")]
    let performance = web_sys::window()
        .unwrap()
        .performance()
        .expect("issue with gettign performance");

    #[cfg(target_arch = "wasm32")]
    let mut last_frame_time = perf_to_system(performance.time_origin());

    #[cfg(not(target_arch = "wasm32"))]
    let mut last_frame_time = std::time::Instant::now();

    let _ = event_loop.run(move |ref og_event, control_flow| {
        match og_event {
            Event::DeviceEvent { event, .. } => {
                println!("EVENT TYPE : {:?}", event);
                state.input(og_event.to_owned());
            }
            Event::WindowEvent {
                window_id,
                event,
            } if window_id == &window.id() => {
                state.input(og_event.to_owned());
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),

                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }

                    WindowEvent::ScaleFactorChanged {
                        inner_size_writer, ..
                    } => {
                        // You can handle scale changes here if needed
                    }

                    WindowEvent::RedrawRequested => {
                        #[cfg(target_arch = "wasm32")]
                        let current_time = perf_to_system(performance.now());
                        #[cfg(not(target_arch = "wasm32"))]
                        let current_time = std::time::Instant::now();
                        let delta: std::time::Duration = (current_time - last_frame_time);
                        println!("\nDELTA: {:?}\n", delta);
                        state.update(delta);

                        match state.render() {
                            Ok(()) => println!(
                                "\ncamera mat: {:?}\n",
                                state.camera.calc_view_mat(),
                            ),
                            Err(e) => println!("{:?}", e),
                        }
                    }
                    _ => {
                        println!("Something else happened");
                    }
                }
            }

            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    });
    ()
}
