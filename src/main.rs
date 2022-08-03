pub mod simulation;
pub mod state;
pub mod tree;

use crate::simulation::{MassDistribution, Simulation, Star};
use crate::state::State;
use nalgebra::{Vector2, Vector3};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::{Duration, Instant};
use wgpu::SurfaceError;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

#[tokio::main]
async fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).expect("failed to create window");

    let mut rng = StdRng::from_entropy();
    let mass_distribution = MassDistribution::new(75.0, 500.0);
    let center_star = Star::new(Vector2::zeros(), Vector2::zeros(), 1_000_000.0);
    let simulation = Simulation::new(
        (0..25_000)
            .map(|_| {
                let a = rng.gen::<f32>() * std::f32::consts::TAU;
                let d = 500.0 * (rng.gen::<f32>()).sqrt();
                let mass = 1.0 + mass_distribution.sample(rng.gen());

                let pos = Vector2::new(a.sin(), a.cos()) * d;
                let n = Vector3::cross(&*Vector3::z_axis(), &Vector3::new(pos.x, pos.y, 0.0));

                Star::new(
                    pos,
                    n.xy().normalize()
                        * (0.5 + rng.gen::<f32>())
                        * (Simulation::GRAVITY * (mass + center_star.mass()) / d).sqrt(),
                    mass,
                )
            })
            .chain(Some(center_star)),
        mass_distribution,
    );

    let mut state = State::new(&window, simulation).await;
    let mut last = Instant::now();
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == window.id() && !state.input(event) => match event {
            WindowEvent::Resized(new_size) => state.resize(*new_size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                state.resize(**new_inner_size)
            }
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
            _ => {}
        },
        Event::MainEventsCleared => window.request_redraw(),
        Event::RedrawRequested(window_id)
            if window_id == window.id() && last.elapsed() > Duration::from_millis(30) =>
        {
            state.update();
            last = Instant::now();

            match state.render() {
                Ok(_) => {}
                Err(e) => match e {
                    SurfaceError::OutOfMemory => *control_flow = ControlFlow::Exit,
                    SurfaceError::Lost => state.resize(state.size),
                    _ => eprintln!("Render Error: {:?}", e),
                },
            }
        }
        _ => {}
    });
}
