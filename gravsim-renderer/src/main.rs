pub mod state;

use crate::state::State;
use gravsim_simulation::{Galaxy, MassDistribution, Simulation, Star};
use nalgebra::Vector2;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::{Duration, Instant};
use wgpu::SurfaceError;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

const N_STARS: usize = 100_000;

#[tokio::main]
async fn main() {

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).expect("failed to create window");

    let mass_distribution = MassDistribution::new(35.0, 200.0);
    let mut rng = StdRng::from_entropy();
    let simulation = Simulation::new((0..N_STARS).map(|_| {
        Star::new(
            Vector2::from_fn(|_, _| rng.gen::<f32>() * 500.0 - 250.0),
            Vector2::zeros(),
            [1.0; 3],
            mass_distribution.sample(rng.gen()),
        )
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
