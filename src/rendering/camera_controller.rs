use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

#[derive(Component)]
pub struct OrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub theta: f32,
    pub phi: f32,
    pub rotate_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::new(0.0, 23.0, 0.0),
            radius: 65.0,
            theta: 1.2,
            phi: -0.5,
            rotate_sensitivity: 0.005,
            pan_sensitivity: 0.08,
            zoom_sensitivity: 2.5,
        }
    }
}

pub fn camera_control_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut OrbitCamera, &mut Transform)>,
    egui_wants: Res<EguiWantsPointer>,
) {
    if egui_wants.0 {
        mouse_motion.read().for_each(|_| {});
        scroll_events.read().for_each(|_| {});
        return;
    }

    let Ok((mut orbit, mut transform)) = camera_query.get_single_mut() else {
        return;
    };

    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    if mouse_button.pressed(MouseButton::Left) {
        orbit.phi -= delta.x * orbit.rotate_sensitivity;
        orbit.theta -= delta.y * orbit.rotate_sensitivity;
        orbit.theta = orbit.theta.clamp(0.05, std::f32::consts::PI - 0.05);
    }

    if mouse_button.pressed(MouseButton::Right) {
        let forward = (orbit.focus - transform.translation).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        let up = right.cross(forward).normalize();

        let pan = right * (-delta.x * orbit.pan_sensitivity)
            + up * (delta.y * orbit.pan_sensitivity);
        orbit.focus += pan;
    }

    for event in scroll_events.read() {
        orbit.radius -= event.y * orbit.zoom_sensitivity;
        orbit.radius = orbit.radius.clamp(5.0, 200.0);
    }

    let x = orbit.radius * orbit.theta.sin() * orbit.phi.cos();
    let y = orbit.radius * orbit.theta.cos();
    let z = orbit.radius * orbit.theta.sin() * orbit.phi.sin();

    transform.translation = orbit.focus + Vec3::new(x, y, z);
    transform.look_at(orbit.focus, Vec3::Y);
}

#[derive(Resource, Default)]
pub struct EguiWantsPointer(pub bool);