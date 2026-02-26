use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::config::{IntegrationMethod, ResetEvent, SimulationConfig, SimulationStats};
use crate::rendering::camera_controller::EguiWantsPointer;

pub fn ui_system(
    mut contexts: EguiContexts,
    mut config: ResMut<SimulationConfig>,
    stats: Res<SimulationStats>,
    diagnostics: Res<DiagnosticsStore>,
    mut reset_events: EventWriter<ResetEvent>,
    mut egui_wants: ResMut<EguiWantsPointer>,
) {
    let ctx = contexts.ctx_mut();

    egui_wants.0 = ctx.is_pointer_over_area();

    let mut style = (*ctx.style()).clone();
    style.spacing.slider_width = 180.0;
    ctx.set_style(style);

    egui::SidePanel::left("control_panel")
        .default_width(300.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("🦋 Lorenz Attractor");
            ui.separator();

            ui.collapsing("🔬 Lorenz Parameters", |ui| {
                ui.label("Canonical values: σ=10, ρ=28, β≈2.667");
                ui.add_space(4.0);

                ui.add(
                    egui::Slider::new(&mut config.sigma, 0.1..=30.0)
                        .text("σ (sigma)")
                        .clamp_to_range(true),
                );
                ui.add(
                    egui::Slider::new(&mut config.rho, 0.1..=50.0)
                        .text("ρ (rho)")
                        .clamp_to_range(true),
                );
                ui.add(
                    egui::Slider::new(&mut config.beta, 0.1..=10.0)
                        .text("β (beta)")
                        .clamp_to_range(true),
                );

                if ui.button("Reset to canonical").clicked() {
                    config.sigma = 10.0;
                    config.rho = 28.0;
                    config.beta = 8.0 / 3.0;
                }
            });

            ui.add_space(8.0);

            ui.collapsing("⚙️ Integration", |ui| {
                ui.add(
                    egui::Slider::new(&mut config.dt, 0.0001..=0.05)
                        .text("dt (time step)")
                        .logarithmic(true)
                        .clamp_to_range(true),
                );

                ui.add(
                    egui::Slider::new(&mut config.steps_per_frame, 1..=50)
                        .text("Steps / frame")
                        .clamp_to_range(true),
                );

                ui.add_space(4.0);
                ui.label("Integration method:");

                ui.radio_value(
                    &mut config.method,
                    IntegrationMethod::RungeKutta4,
                    IntegrationMethod::RungeKutta4.label(),
                );
                ui.radio_value(
                    &mut config.method,
                    IntegrationMethod::Euler,
                    IntegrationMethod::Euler.label(),
                );

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(match config.method {
                        IntegrationMethod::Euler => {
                            "⚠ Euler: O(dt) error. Expect drift at large dt."
                        }
                        IntegrationMethod::RungeKutta4 => {
                            "✓ RK4: O(dt⁴) error. Recommended for accuracy."
                        }
                    })
                    .small()
                    .color(match config.method {
                        IntegrationMethod::Euler => egui::Color32::YELLOW,
                        IntegrationMethod::RungeKutta4 => egui::Color32::LIGHT_GREEN,
                    }),
                );
            });

            ui.add_space(8.0);

            ui.collapsing("🎨 Trail", |ui| {
                let mut max_k = config.max_trail_points as f64 / 1000.0;
                ui.add(
                    egui::Slider::new(&mut max_k, 1.0..=100.0)
                        .text("Max points (×1000)")
                        .clamp_to_range(true),
                );
                config.max_trail_points = (max_k * 1000.0) as usize;

                ui.label(format!("Active points: {}", stats.point_count));
                ui.label(format!(
                    "Memory: ~{:.1} KB",
                    stats.point_count as f64 * 32.0 / 1024.0
                ));
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui
                    .button(if config.paused { "▶ Play" } else { "⏸ Pause" })
                    .clicked()
                {
                    config.paused = !config.paused;
                }
                if ui.button("🔄 Reset").clicked() {
                    reset_events.send(ResetEvent);
                }
            });

            ui.add_space(8.0);

            ui.collapsing("📍 Initial Conditions", |ui| {
                ui.add(
                    egui::Slider::new(&mut config.initial_x, -20.0..=20.0)
                        .text("x₀"),
                );
                ui.add(
                    egui::Slider::new(&mut config.initial_y, -20.0..=20.0)
                        .text("y₀"),
                );
                ui.add(
                    egui::Slider::new(&mut config.initial_z, -20.0..=20.0)
                        .text("z₀"),
                );
                ui.label(
                    egui::RichText::new("Changes apply on Reset")
                        .small()
                        .italics(),
                );
            });

            ui.add_space(8.0);

            ui.collapsing("📊 Diagnostics", |ui| {
                ui.checkbox(&mut config.show_energy, "Show energy");
                ui.checkbox(&mut config.show_velocity, "Show velocity");
                ui.checkbox(&mut config.show_divergence, "Show divergence");

                ui.add_space(4.0);
                ui.separator();

                if config.show_energy {
                    ui.label(format!("Energy (½|X|²): {:.2}", stats.current_energy));
                }
                if config.show_velocity {
                    ui.label(format!("Velocity |dX/dt|: {:.2}", stats.current_velocity));
                }
                if config.show_divergence {
                    ui.label(format!("Divergence ∇·F: {:.4}", stats.divergence));
                    ui.label(
                        egui::RichText::new("(Constant — system is uniformly dissipative)")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                }

                ui.add_space(4.0);
                ui.separator();

                if let Some(fps) = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(|d| d.smoothed())
                {
                    ui.label(format!("FPS: {:.0}", fps));
                }

                ui.label(format!(
                    "Integration time: {:.1} μs",
                    stats.integration_time_us
                ));
            });

            ui.add_space(16.0);
            ui.separator();

            ui.collapsing("❓ Camera Controls", |ui| {
                ui.label("🖱 Left drag: Orbit");
                ui.label("🖱 Right drag: Pan");
                ui.label("🖱 Scroll: Zoom");
            });
        });
}