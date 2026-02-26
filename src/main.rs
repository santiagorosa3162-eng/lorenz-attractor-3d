mod config;
mod rendering;
mod simulation;
mod ui;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use config::{ResetEvent, SimulationConfig, SimulationStats};
use rendering::camera_controller::{camera_control_system, EguiWantsPointer, OrbitCamera};
use rendering::trail_renderer::{draw_axes_system, draw_head_marker_system, draw_trail_system};
use simulation::integrator::{simulation_system, TrailBuffer};
use simulation::lorenz::LorenzState;
use ui::controls::ui_system;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Lorenz Attractor — RK4 / Euler Simulation".into(),
                resolution: (1400.0, 900.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .init_resource::<SimulationConfig>()
        .init_resource::<SimulationStats>()
        .init_resource::<TrailBuffer>()
        .init_resource::<EguiWantsPointer>()
        .add_event::<ResetEvent>()
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.04)))
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                ui_system,
                simulation_system,
                draw_trail_system,
                draw_head_marker_system,
                draw_axes_system,
                camera_control_system,
            )
                .chain(),
        )
        .run();
}

fn setup_scene(mut commands: Commands, config: Res<SimulationConfig>) {
    let orbit = OrbitCamera::default();

    let x = orbit.radius * orbit.theta.sin() * orbit.phi.cos();
    let y = orbit.radius * orbit.theta.cos();
    let z = orbit.radius * orbit.theta.sin() * orbit.phi.sin();
    let camera_pos = orbit.focus + Vec3::new(x, y, z);

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(camera_pos)
                .looking_at(orbit.focus, Vec3::Y),
            ..default()
        },
        orbit,
    ));

    commands.spawn(LorenzState::new(
        config.initial_x,
        config.initial_y,
        config.initial_z,
    ));

    info!("Lorenz attractor simulation initialized.");
    info!("  σ = {}, ρ = {}, β = {:.4}", config.sigma, config.rho, config.beta);
    info!("  dt = {}, method = {:?}", config.dt, config.method);
    info!(
        "  Initial state: ({}, {}, {})",
        config.initial_x, config.initial_y, config.initial_z
    );
}