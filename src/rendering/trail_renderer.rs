use bevy::prelude::*;

use crate::simulation::integrator::TrailBuffer;

pub fn draw_trail_system(mut gizmos: Gizmos, trail: Res<TrailBuffer>) {
    let len = trail.points.len();
    if len < 2 {
        return;
    }

    gizmos.linestrip_gradient(
        trail.points.iter().map(|p| (p.position, p.color)),
    );
}

pub fn draw_head_marker_system(
    mut gizmos: Gizmos,
    trail: Res<TrailBuffer>,
) {
    if let Some(head) = trail.points.back() {
        let radius = 0.35;
        gizmos.sphere(head.position, Quat::IDENTITY, radius, Color::WHITE);
    }
}

pub fn draw_axes_system(mut gizmos: Gizmos) {
    let half_len = 5.0;
    let alpha = 0.25;

    gizmos.line(
        Vec3::new(-half_len, 0.0, 0.0),
        Vec3::new(half_len, 0.0, 0.0),
        Color::srgba(1.0, 0.3, 0.3, alpha),
    );

    gizmos.line(
        Vec3::new(0.0, -half_len, 0.0),
        Vec3::new(0.0, half_len, 0.0),
        Color::srgba(0.3, 1.0, 0.3, alpha),
    );

    gizmos.line(
        Vec3::new(0.0, 0.0, -half_len),
        Vec3::new(0.0, 0.0, half_len),
        Color::srgba(0.3, 0.3, 1.0, alpha),
    );
}