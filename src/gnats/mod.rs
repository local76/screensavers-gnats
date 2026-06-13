//! Consolidated gnats screensaver effect module.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).

mod types;
mod physics;
mod physics_helpers;
mod render;
mod render_helpers;
mod update_helpers;

pub use types::{Firefly, Attractor, Star, KillSpark};

use crate::runner::core::{LcgRng, TerminalCell, hsl_to_rgb, rgb_to_hsl};
use std::time::Duration;
use crate::runner::core::screensaver::Screensaver;
use crate::runner::toolkit::sys_info::{query_current_palette, get_system_info};

pub struct Gnats {
    pub(crate) rng: LcgRng,
    pub(crate) fireflies: Vec<Firefly>,
    pub(crate) attractors: Vec<Attractor>,
    pub(crate) stars: Vec<Star>,
    pub(crate) kill_sparks: Vec<KillSpark>,
    pub(crate) time_elapsed: f32,
    pub(crate) last_cols: usize,
    pub(crate) last_rows: usize,
    pub(crate) logo_excitation: Vec<f32>,
    pub(super) on_battery: bool,
    pub(super) frame_time_ema: f32,
    pub(super) quality_scale: f32,
    pub(super) target_frame_time: f32,
    pub(crate) sys_refresh_timer: f32,
    pub(crate) mem_pressure: f32,
    pub(crate) cpu_load: f32,
    pub(crate) logo_text: String,
    pub(crate) accent: (u8, u8, u8),
}

impl Default for Gnats {
    fn default() -> Self {
        Self::new()
    }
}

impl Gnats {
    pub fn new() -> Self {
        let rng = LcgRng::new(1357);
        let sys = get_system_info();
        let on_battery = sys.power_status.contains("Battery");
        let accent = query_current_palette().accent;
        Self {
            rng,
            fireflies: Vec::new(),
            attractors: Vec::new(),
            stars: Vec::new(),
            kill_sparks: Vec::new(),
            time_elapsed: 0.0,
            last_cols: 0,
            last_rows: 0,
            logo_excitation: Vec::new(),
            on_battery,
            frame_time_ema: 0.01666667,
            quality_scale: 1.0,
            target_frame_time: 0.01666667,
            sys_refresh_timer: 0.0,
            mem_pressure: sys.mem_used_pct / 100.0,
            cpu_load: (sys.cpu_usage_pct / 100.0).clamp(0.0, 1.0),
            logo_text: sys.logo_text,
            accent,
        }
    }


}

impl Screensaver for Gnats {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let dt_secs = dt.as_secs_f32().min(0.1);

        // Auto-detect high refresh rates during the startup phase
        if self.time_elapsed < 2.0 && dt_secs > 0.001 {
            if dt_secs < self.target_frame_time - 0.001 {
                self.target_frame_time = dt_secs;
            }
        }

        // Exponential moving average for frame time (alpha = 0.1)
        self.frame_time_ema = self.frame_time_ema * 0.9 + dt_secs.min(0.2) * 0.1;

        let speed_mult = if self.on_battery { 0.65 } else { 1.0 };
        let delta = dt_secs * speed_mult;
        self.time_elapsed += delta;

        // Adjust quality_scale based on frame time performance vs target
        if self.time_elapsed > 1.5 {
            if self.frame_time_ema > self.target_frame_time * 1.15 {
                self.quality_scale = (self.quality_scale - 0.15 * delta).max(0.20);
            } else if self.frame_time_ema < self.target_frame_time * 1.05 {
                self.quality_scale = (self.quality_scale + 0.04 * delta).min(1.0);
            }
        }

        self.sys_refresh_timer += delta;
        if self.sys_refresh_timer >= 1.0 {
            let sys = get_system_info();
            self.mem_pressure = sys.mem_used_pct / 100.0;
            self.cpu_load = (sys.cpu_usage_pct / 100.0).clamp(0.0, 1.0);
            self.on_battery = sys.power_status.contains("Battery");
            self.logo_text = sys.logo_text;
            self.accent = query_current_palette().accent;
            self.sys_refresh_timer = 0.0;
        }

        // Initialize particles and attractors if grid size changes
        if cols != self.last_cols || rows != self.last_rows {
            self.last_cols = cols;
            self.last_rows = rows;

            // library 4.1: fixed-size logo excitation buffer (pre-4.1
            // `trance_core::logo_dimensions()` was a Windows file read).
            self.logo_excitation = vec![0.0; 80 * 12];

            // Recreate attractors
            self.attractors.clear();
            let accent = self.accent;
            let (acc_h, _acc_s, _acc_l) = rgb_to_hsl(accent.0, accent.1, accent.2);
            self.attractors.push(Attractor {
                x: cols as f32 / 2.0,
                y: rows as f32 / 2.0,
                color: accent,
                phase: 0.0,
                speed: 0.6,
            });
            self.attractors.push(Attractor {
                x: cols as f32 / 2.0,
                y: rows as f32 / 2.0,
                color: hsl_to_rgb((acc_h + 120.0).rem_euclid(360.0), 0.95, 0.60),
                phase: 2.0,
                speed: 0.45,
            });
            self.attractors.push(Attractor {
                x: cols as f32 / 2.0,
                y: rows as f32 / 2.0,
                color: hsl_to_rgb((acc_h - 120.0).rem_euclid(360.0), 0.95, 0.60),
                phase: 4.0,
                speed: 0.75,
            });

            self.fireflies.clear();
            self.stars.clear();
            self.kill_sparks.clear();
        }

        self.adjust_populations(cols, rows);

        let cols_f = cols as f32;
        let rows_f = rows as f32;

        physics::update_attractors(&mut self.attractors, self.time_elapsed, cols_f, rows_f);
        physics::decay_logo_excitations(&mut self.logo_excitation, delta);

        let dead_indices = physics::compute_firefly_forces_and_update(
            &mut self.fireflies,
            &self.attractors,
            self.time_elapsed,
            cols_f,
            rows_f,
            &mut self.rng,
            delta,
        );

        // Process dead fireflies (remove, trigger explosions, and respawn)
        if !dead_indices.is_empty() {
            let mut unique_dead = dead_indices;
            unique_dead.sort_unstable();
            unique_dead.dedup();

            for &idx in unique_dead.iter().rev() {
                if idx < self.fireflies.len() {
                    let dead = self.fireflies.remove(idx);

                    // Spawn a colorful neon spark explosion burst
                    for _ in 0..12 {
                        let angle = self.rng.next_range(0.0, std::f32::consts::TAU);
                        let speed = self.rng.next_range(8.0, 22.0);
                        self.kill_sparks.push(KillSpark {
                            x: dead.x,
                            y: dead.y,
                            vx: angle.cos() * speed,
                            vy: angle.sin() * speed * 0.5,
                            color: dead.color,
                            life: self.rng.next_range(0.5, 1.2),
                        });
                    }

                    // Respawn a new firefly on the border to replace the population
                    self.spawn_new_firefly(cols, rows);
                }
            }
        }

        physics::update_kill_sparks(&mut self.kill_sparks, delta);
        physics::update_stars(&mut self.stars, &self.fireflies, delta, cols_f, rows_f);
        physics::update_logo_excitations(&mut self.logo_excitation, &self.fireflies, cols, rows);
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        render::draw_gnats(self, grid, cols, rows);
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;

