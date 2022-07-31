pub mod simulation;
pub mod state;
pub mod tree;

use crate::simulation::{Simulation, Star};
use crate::state::State;
use nalgebra::Vector2;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::{Duration, Instant};
use wgpu::SurfaceError;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

#[tokio::main]
async fn main() {
    const ALPHA: f32 = 35.0;

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).expect("failed to create window");

    let mut rng = StdRng::from_entropy();
    let simulation = Simulation::new((0..100_000).map(|_| {
        let a = rng.gen::<f32>() * std::f32::consts::TAU;
        let d = (rng.gen::<f32>() * 400.0 * 400.0).sqrt();
        let mass = 100.0 * (rng.gen::<f32>() * ALPHA).exp_m1() / ALPHA.exp_m1();

        Star::new(Vector2::new(a.sin(), a.cos()) * d, Vector2::zeros(), mass)
    }));

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
