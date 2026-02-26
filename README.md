# Lorenz Attractor — 3D Numerical Simulation & Visualization

A high-fidelity, real-time 3D simulation of the **Lorenz strange attractor** built with **Rust** and the **Bevy** game engine. Implements both Euler and Runge-Kutta 4 numerical integration with full parameter control, orbital camera, velocity-mapped trail coloring, and live diagnostic overlays.

---

## Technical Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Language | Rust | 2021 edition |
| Engine | Bevy ECS | 0.14.x |
| UI | bevy_egui (egui) | 0.28.x |
| Rendering | Bevy Gizmos (linestrip gradient) | — |
| Numerics | Custom f64 integrators | — |

---

## Build & Run

### Prerequisites

- **Rust toolchain** ≥ 1.77 (install via [rustup.rs](https://rustup.rs))
- **System dependencies** for Bevy (Linux):
  ```bash
  # Ubuntu / Debian
  sudo apt install -y g++ pkg-config libx11-dev libxi-dev libxcursor-dev \
      libxrandr-dev libxinerama-dev libwayland-dev libxkbcommon-dev \
      libasound2-dev libudev-dev libvulkan-dev
  ```
  macOS and Windows require no additional system packages.

### Compilation

```bash
git clone https://github.com/your-org/lorenz-attractor.git
cd lorenz-attractor

# Development build (fast compile, dynamic linking enabled)
cargo run

# Release build (optimized, LTO, stripped)
cargo run --release
```

First build takes 3–8 minutes (Bevy compilation). Subsequent builds are incremental (~2–5 seconds).

---

## Mathematical Background

### The Lorenz System

In 1963, Edward Lorenz discovered that a drastically simplified model of atmospheric convection exhibits **deterministic chaos** — behavior that is fully determined by its equations yet practically unpredictable over long time horizons.

The system is defined by three coupled ordinary differential equations:

```
dx/dt = σ (y − x)
dy/dt = x (ρ − z) − y
dz/dt = x y − β z
```

where **x** represents convective intensity, **y** the horizontal temperature difference, and **z** the vertical temperature stratification.

#### Parameters

| Symbol | Name | Canonical Value | Role |
|--------|------|----------------|------|
| σ | Prandtl number | 10 | Ratio of momentum to thermal diffusivity |
| ρ | Rayleigh number | 28 | Normalized temperature gradient (bifurcation parameter) |
| β | Geometric factor | 8/3 ≈ 2.667 | Aspect ratio damping |

#### Equilibria

The system has three equilibrium points:

- **C₀ = (0, 0, 0)** — unstable saddle point
- **C₊ = (+√(β(ρ−1)), +√(β(ρ−1)), ρ−1)** — unstable spiral
- **C₋ = (−√(β(ρ−1)), −√(β(ρ−1)), ρ−1)** — unstable spiral

With standard parameters, all three equilibria are unstable, forcing the trajectory to wander perpetually between the two spiral lobes — forming the characteristic "butterfly" shape.

### Chaos & Sensitive Dependence on Initial Conditions

The **largest Lyapunov exponent** of the Lorenz system (with standard parameters) is λ₁ ≈ 0.906. This means two trajectories starting at distance δ₀ apart diverge as:

```
δ(t) ~ δ₀ · exp(λ₁ · t)
```

For an initial perturbation of 10⁻¹⁰, the trajectories become uncorrelated after approximately 28 time units. This is the mathematical foundation of the **butterfly effect**: no finite-precision computation can track the true trajectory indefinitely.

### Dissipative Structure

The divergence of the vector field is:

```
∇·F = −(σ + 1 + β) ≈ −13.667
```

This is **constant** — a remarkable property meaning phase-space volumes contract uniformly at an exponential rate. The attractor therefore has zero Lebesgue measure and a fractal (Hausdorff) dimension of approximately 2.06.

### Numerical Integration

#### Euler Method

```
X_{n+1} = X_n + dt · F(X_n)
```

- **Order**: 1 (global error O(dt))
- **Stability**: Requires dt < ~0.02 for the Lorenz system
- **Pros**: Simplest possible integrator; useful as baseline
- **Cons**: Rapid phase drift on chaotic trajectories; poor energy conservation

#### Runge-Kutta 4 (RK4)

```
k₁ = F(X_n)
k₂ = F(X_n + ½dt·k₁)
k₃ = F(X_n + ½dt·k₂)
k₄ = F(X_n + dt·k₃)
X_{n+1} = X_n + (dt/6)(k₁ + 2k₂ + 2k₃ + k₄)
```

- **Order**: 4 (global error O(dt⁴))
- **Stability**: Larger stability region; reliable up to dt ≈ 0.05
- **Pros**: Excellent curvature tracking; 50× more efficient than Euler at equivalent accuracy
- **Cons**: 4× more derivative evaluations per step (negligible for our system size)

#### Impact of Step Size (dt)

| dt | RK4 Error (per unit time) | Euler Error (per unit time) | Visual Quality |
|----|--------------------------|----------------------------|----------------|
| 0.001 | ~10⁻¹² | ~10⁻³ | Excellent |
| 0.005 | ~10⁻⁸ | ~10⁻² | Excellent (default) |
| 0.01 | ~10⁻⁶ | ~10⁻¹ | Good |
| 0.05 | ~10⁻² | >1 (unstable) | Euler diverges |

---

## Architecture

### Module Structure

```
src/
├── main.rs                        # App entry point, plugin & system registration
├── config.rs                      # SimulationConfig, SimulationStats, ResetEvent
├── simulation/
│   ├── mod.rs                     # Module declarations
│   ├── lorenz.rs                  # ODE definition, state, energy, divergence
│   └── integrator.rs              # Euler, RK4, TrailBuffer, simulation_system
├── rendering/
│   ├── mod.rs                     # Module declarations
│   ├── trail_renderer.rs          # Gizmo-based trail, head marker, axes
│   └── camera_controller.rs       # Orbital camera, EguiWantsPointer
└── ui/
    ├── mod.rs                     # Module declarations
    └── controls.rs                # egui side panel, diagnostics overlay
```

### Responsibility Matrix

| Module | Reads | Writes | Purpose |
|--------|-------|--------|---------|
| `ui/controls` | SimulationStats, DiagnosticsStore | SimulationConfig, ResetEvent, EguiWantsPointer | User interaction |
| `simulation/integrator` | SimulationConfig | LorenzState, TrailBuffer, SimulationStats | Numerical integration |
| `rendering/trail_renderer` | TrailBuffer | Gizmos (GPU) | Visual output |
| `rendering/camera_controller` | EguiWantsPointer, MouseInput | Camera Transform | Viewport navigation |

### ECS Entity Layout

| Entity | Components | Role |
|--------|-----------|------|
| Camera | Camera3d, Transform, OrbitCamera | Viewport |
| Particle | LorenzState | Integrated state |

### System Execution Order

```
ui_system → simulation_system → draw_trail → draw_head → draw_axes → camera_control
```

Enforced via Bevy's `.chain()` combinator to guarantee data consistency within a single frame.

### Decoupling Simulation from Rendering

The simulation and rendering layers share exactly **one data structure**: `TrailBuffer`. The simulation appends `TrailPoint` records (position + color); the renderer reads them. Neither layer has any knowledge of the other's implementation details. This means:

1. The simulation can be unit-tested without a GPU context.
2. The renderer could be swapped (e.g., to a custom mesh pipeline) without touching the integrator.
3. The `TrailBuffer` acts as a bounded FIFO queue, preventing unbounded memory growth.

---

## Controls

### Real-Time Parameter Panel

| Control | Range | Default | Effect |
|---------|-------|---------|--------|
| σ (sigma) | 0.1 – 30 | 10 | Prandtl number |
| ρ (rho) | 0.1 – 50 | 28 | Rayleigh number (bifurcation) |
| β (beta) | 0.1 – 10 | 8/3 | Geometric damping |
| dt | 0.0001 – 0.05 | 0.005 | Integration step size |
| Steps/frame | 1 – 50 | 8 | Simulation speed multiplier |
| Method | Euler / RK4 | RK4 | Integration algorithm |
| Max points | 1K – 100K | 25K | Trail memory budget |

### Camera

| Input | Action |
|-------|--------|
| Left mouse drag | Orbit around attractor |
| Right mouse drag | Pan focus point |
| Scroll wheel | Zoom in/out |

### Playback

| Button | Action |
|--------|--------|
| ▶ Play / ⏸ Pause | Toggle simulation |
| 🔄 Reset | Clear trail, reset to initial conditions |

---

## Diagnostics

The panel displays real-time computed values:

- **Energy** E = ½(x² + y² + z²) — oscillates as the trajectory spirals
- **Velocity** |dX/dt| — high during lobe transitions, low near equilibria
- **Divergence** ∇·F = −(σ + 1 + β) — constant, confirming dissipative dynamics
- **FPS** — rendering frame rate (Bevy diagnostic)
- **Integration time** — wall-clock microseconds spent in the ODE solver per frame

---

## Expected Visuals

Upon launch, the application renders a 3D butterfly-shaped attractor with:

- A **gradient-colored trail** flowing from blue (low velocity) through green to red (high velocity)
- A **white sphere** marking the current trajectory position
- **Subtle reference axes** at the origin
- A **dark background** (near-black with slight blue tint)
- An **egui side panel** with all controls visible

The attractor fills approximately 40×60×50 units of 3D space. The default camera angle provides an oblique view showing both lobes clearly.

---

## Design Decisions & Justifications

### Why f64 for simulation, f32 for rendering?

The Lorenz system's chaotic dynamics amplify floating-point errors exponentially. Using `f64` for the integrator provides ~15 decimal digits of precision, extending useful trajectory accuracy from ~100 time units (f32) to ~1000+ time units (f64). Rendering uses `f32` (Bevy's `Vec3`) since pixel-level precision doesn't benefit from the extra bits.

### Why Gizmos instead of custom meshes?

Bevy's Gizmos API provides per-vertex color interpolation, automatic depth testing, and zero shader boilerplate. For 25K points at 60 FPS, the per-frame GPU upload (~800 KB) is well within bandwidth limits. A custom mesh approach would avoid the per-frame rebuild but requires a WGSL vertex-color shader and manual buffer management — added complexity for marginal gain at this scale.

### Why fixed dt instead of adaptive?

Fixed step size makes the simulation deterministic and allows the user to directly observe how `dt` affects stability and accuracy — a pedagogical feature. Adaptive methods (e.g., Dormand-Prince RK45) would be preferred for production scientific computing.

### Why bevy_egui instead of Bevy UI?

`egui` provides rich widgets (sliders, collapsibles, radio buttons) with minimal code. Bevy's built-in UI requires significantly more boilerplate for equivalent functionality. The immediate-mode paradigm also eliminates widget state management, reducing bug surface.

---

## Future Enhancements

- **Multiple simultaneous trajectories** with different initial conditions to visualize Lyapunov divergence
- **Adaptive RK45 (Dormand-Prince)** with embedded error estimation and automatic step control
- **Custom mesh rendering** with GPU compute for 10⁶+ point trails
- **Poincaré section** visualization (intersection of trajectory with a plane)
- **Bifurcation diagram** as ρ varies, showing transition from fixed points to chaos
- **Export** trail data to CSV or PLY for external analysis
- **Color mode selector**: velocity, acceleration, time, curvature
- **VR/AR support** via Bevy's XR plugins for immersive exploration

---

## License

MIT

---

## References

1. Lorenz, E.N. (1963). "Deterministic Nonperiodic Flow." *Journal of the Atmospheric Sciences*, 20(2), 130–141.
2. Strogatz, S.H. (2015). *Nonlinear Dynamics and Chaos*. Westview Press, 2nd edition.
3. Hairer, E., Nørsett, S.P., & Wanner, G. (1993). *Solving Ordinary Differential Equations I*. Springer.
4. Sparrow, C. (1982). *The Lorenz Equations: Bifurcations, Chaos, and Strange Attractors*. Springer.
