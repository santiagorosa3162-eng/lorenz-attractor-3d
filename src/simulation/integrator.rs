use std::collections::VecDeque;
use std::time::Instant;

use bevy::prelude::*;

use crate::config::{IntegrationMethod, ResetEvent, SimulationConfig, SimulationStats};
use super::lorenz::{
    divergence, lorenz_derivatives, system_energy, velocity_magnitude, LorenzParams, LorenzState,
};

#[derive(Clone, Debug)]
pub struct TrailPoint {
    pub position: Vec3,
    pub color: Color,
}

#[derive(Resource)]
pub struct TrailBuffer {
    pub points: VecDeque<TrailPoint>,
    pub max_points: usize,
}

impl Default for TrailBuffer {
    fn default() -> Self {
        Self {
            points: VecDeque::with_capacity(25_000),
            max_points: 25_000,
        }
    }
}

#[inline]
pub fn euler_step(state: &LorenzState, params: &LorenzParams, dt: f64) -> LorenzState {
    let (dx, dy, dz) = lorenz_derivatives(state, params);
    LorenzState::new(
        state.x + dt * dx,
        state.y + dt * dy,
        state.z + dt * dz,
    )
}

#[inline]
pub fn rk4_step(state: &LorenzState, params: &LorenzParams, dt: f64) -> LorenzState {
    let (k1x, k1y, k1z) = lorenz_derivatives(state, params);

    let s2 = LorenzState::new(
        state.x + 0.5 * dt * k1x,
        state.y + 0.5 * dt * k1y,
        state.z + 0.5 * dt * k1z,
    );
    let (k2x, k2y, k2z) = lorenz_derivatives(&s2, params);

    let s3 = LorenzState::new(
        state.x + 0.5 * dt * k2x,
        state.y + 0.5 * dt * k2y,
        state.z + 0.5 * dt * k2z,
    );
    let (k3x, k3y, k3z) = lorenz_derivatives(&s3, params);

    let s4 = LorenzState::new(
        state.x + dt * k3x,
        state.y + dt * k3y,
        state.z + dt * k3z,
    );
    let (k4x, k4y, k4z) = lorenz_derivatives(&s4, params);

    let sixth_dt = dt / 6.0;
    LorenzState::new(
        state.x + sixth_dt * (k1x + 2.0 * k2x + 2.0 * k3x + k4x),
        state.y + sixth_dt * (k1y + 2.0 * k2y + 2.0 * k3y + k4y),
        state.z + sixth_dt * (k1z + 2.0 * k2z + 2.0 * k3z + k4z),
    )
}

fn velocity_to_color(velocity: f64) -> Color {
    const MAX_VELOCITY: f64 = 55.0;
    let t = (velocity / MAX_VELOCITY).clamp(0.0, 1.0);
    let hue: f32 = (240.0 * (1.0 - t)) as f32;
    let saturation: f32 = 0.85;
    let lightness: f32 = 0.55;
    Color::hsl(hue, saturation, lightness)
}

pub fn simulation_system(
    config: Res<SimulationConfig>,
    mut state_query: Query<&mut LorenzState>,
    mut trail: ResMut<TrailBuffer>,
    mut stats: ResMut<SimulationStats>,
    mut reset_events: EventReader<ResetEvent>,
) {
    for _ in reset_events.read() {
        trail.points.clear();
        for mut state in state_query.iter_mut() {
            state.x = config.initial_x;
            state.y = config.initial_y;
            state.z = config.initial_z;
        }
        return;
    }

    trail.max_points = config.max_trail_points;

    if config.paused {
        if let Ok(state) = state_query.get_single() {
            let params = LorenzParams {
                sigma: config.sigma,
                rho: config.rho,
                beta: config.beta,
            };
            stats.current_energy = system_energy(&state);
            stats.current_velocity = velocity_magnitude(&state, &params);
            stats.divergence = divergence(&params);
            stats.point_count = trail.points.len();
            stats.integration_time_us = 0.0;
        }
        return;
    }

    let params = LorenzParams {
        sigma: config.sigma,
        rho: config.rho,
        beta: config.beta,
    };

    let timer = Instant::now();

    for mut state in state_query.iter_mut() {
        for _ in 0..config.steps_per_frame {
            let new_state = match config.method {
                IntegrationMethod::Euler => euler_step(&state, &params, config.dt),
                IntegrationMethod::RungeKutta4 => rk4_step(&state, &params, config.dt),
            };

            if new_state.x.is_nan()
                || new_state.y.is_nan()
                || new_state.z.is_nan()
                || new_state.x.abs() > 1e6
            {
                continue;
            }

            let vel = velocity_magnitude(&new_state, &params);

            let point = TrailPoint {
                position: new_state.to_vec3(),
                color: velocity_to_color(vel),
            };
            trail.points.push_back(point);

            while trail.points.len() > trail.max_points {
                trail.points.pop_front();
            }

            state.x = new_state.x;
            state.y = new_state.y;
            state.z = new_state.z;
        }

        stats.current_energy = system_energy(&state);
        stats.current_velocity = velocity_magnitude(&state, &params);
        stats.divergence = divergence(&params);
        stats.point_count = trail.points.len();
    }

    stats.integration_time_us = timer.elapsed().as_secs_f64() * 1_000_000.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn std_params() -> LorenzParams {
        LorenzParams {
            sigma: 10.0,
            rho: 28.0,
            beta: 8.0 / 3.0,
        }
    }

    #[test]
    fn test_euler_advances_state() {
        let state = LorenzState::new(1.0, 1.0, 1.0);
        let next = euler_step(&state, &std_params(), 0.01);
        assert!((next.x - state.x).abs() > 1e-10);
    }

    #[test]
    fn test_rk4_advances_state() {
        let state = LorenzState::new(1.0, 1.0, 1.0);
        let next = rk4_step(&state, &std_params(), 0.01);
        assert!((next.x - state.x).abs() > 1e-10);
    }

    #[test]
    fn test_rk4_more_accurate_than_euler() {
        let state = LorenzState::new(1.0, 1.0, 1.0);
        let params = std_params();
        let dt_coarse = 0.1;
        let dt_fine = 0.0001;
        let steps_fine = (dt_coarse / dt_fine).round() as usize;

        let mut ref_state = state.clone();
        for _ in 0..steps_fine {
            ref_state = rk4_step(&ref_state, &params, dt_fine);
        }

        let euler_result = euler_step(&state, &params, dt_coarse);
        let euler_err = (euler_result.x - ref_state.x).powi(2)
            + (euler_result.y - ref_state.y).powi(2)
            + (euler_result.z - ref_state.z).powi(2);

        let rk4_result = rk4_step(&state, &params, dt_coarse);
        let rk4_err = (rk4_result.x - ref_state.x).powi(2)
            + (rk4_result.y - ref_state.y).powi(2)
            + (rk4_result.z - ref_state.z).powi(2);

        assert!(rk4_err < euler_err);
    }
}