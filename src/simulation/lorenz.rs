use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct LorenzState {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl LorenzState {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.z as f32, self.y as f32)
    }
}

impl Default for LorenzState {
    fn default() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }
}

pub struct LorenzParams {
    pub sigma: f64,
    pub rho: f64,
    pub beta: f64,
}

#[inline]
pub fn lorenz_derivatives(state: &LorenzState, params: &LorenzParams) -> (f64, f64, f64) {
    let dx = params.sigma * (state.y - state.x);
    let dy = state.x * (params.rho - state.z) - state.y;
    let dz = state.x * state.y - params.beta * state.z;
    (dx, dy, dz)
}

pub fn velocity_magnitude(state: &LorenzState, params: &LorenzParams) -> f64 {
    let (dx, dy, dz) = lorenz_derivatives(state, params);
    (dx * dx + dy * dy + dz * dz).sqrt()
}

pub fn system_energy(state: &LorenzState) -> f64 {
    0.5 * (state.x * state.x + state.y * state.y + state.z * state.z)
}

pub fn divergence(params: &LorenzParams) -> f64 {
    -(params.sigma + 1.0 + params.beta)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standard_params() -> LorenzParams {
        LorenzParams {
            sigma: 10.0,
            rho: 28.0,
            beta: 8.0 / 3.0,
        }
    }

    #[test]
    fn test_equilibrium_origin() {
        let state = LorenzState::new(0.0, 0.0, 0.0);
        let (dx, dy, dz) = lorenz_derivatives(&state, &standard_params());
        assert!((dx.abs() + dy.abs() + dz.abs()) < 1e-15);
    }

    #[test]
    fn test_equilibrium_c_plus() {
        let p = standard_params();
        let val = (p.beta * (p.rho - 1.0)).sqrt();
        let state = LorenzState::new(val, val, p.rho - 1.0);
        let (dx, dy, dz) = lorenz_derivatives(&state, &p);
        assert!(dx.abs() < 1e-12);
        assert!(dy.abs() < 1e-12);
        assert!(dz.abs() < 1e-12);
    }

    #[test]
    fn test_divergence_value() {
        let p = standard_params();
        let div = divergence(&p);
        let expected = -(10.0 + 1.0 + 8.0 / 3.0);
        assert!((div - expected).abs() < 1e-12);
    }

    #[test]
    fn test_energy_positive() {
        let state = LorenzState::new(5.0, -3.0, 12.0);
        assert!(system_energy(&state) > 0.0);
    }
}