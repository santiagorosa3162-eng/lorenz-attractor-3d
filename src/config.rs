use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationMethod {
    Euler,
    RungeKutta4,
}

impl IntegrationMethod {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Euler => "Euler (1st order)",
            Self::RungeKutta4 => "Runge-Kutta 4 (4th order)",
        }
    }
}

#[derive(Resource)]
pub struct SimulationConfig {
    pub sigma: f64,
    pub rho: f64,
    pub beta: f64,

    pub dt: f64,
    pub method: IntegrationMethod,
    pub steps_per_frame: u32,
    pub paused: bool,

    pub max_trail_points: usize,

    pub initial_x: f64,
    pub initial_y: f64,
    pub initial_z: f64,

    pub show_energy: bool,
    pub show_divergence: bool,
    pub show_velocity: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            sigma: 10.0,
            rho: 28.0,
            beta: 8.0 / 3.0,

            dt: 0.005,
            method: IntegrationMethod::RungeKutta4,
            steps_per_frame: 8,
            paused: false,

            max_trail_points: 25_000,

            initial_x: 1.0,
            initial_y: 1.0,
            initial_z: 1.0,

            show_energy: true,
            show_divergence: true,
            show_velocity: true,
        }
    }
}

#[derive(Resource, Default)]
pub struct SimulationStats {
    pub integration_time_us: f64,
    pub current_energy: f64,
    pub current_velocity: f64,
    pub divergence: f64,
    pub point_count: usize,
}

#[derive(Event)]
pub struct ResetEvent;