//! Consolidated gnats screensaver effect module.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).

mod types;
mod physics;
mod render;

pub use types::{Firefly, Attractor, Star, KillSpark};

use library::core::{LcgRng, TerminalCell, hsl_to_rgb, rgb_to_hsl};
use std::time::Duration;
use library::core::screensaver::Screensaver;
use library::toolkit::sys_info::{query_current_palette, get_system_info};

pub struct Gnats {
    rng: LcgRng,
    pub(crate) fireflies: Vec<Firefly>,
    pub(crate) attractors: Vec<Attractor>,
    pub(crate) stars: Vec<Star>,
    pub(crate) kill_sparks: Vec<KillSpark>,
    pub(crate) time_elapsed: f32,
    last_cols: usize,
    last_rows: usize,
    pub(crate) logo_excitation: Vec<f32>,
    pub(super) on_battery: bool,
    pub(super) frame_time_ema: f32,
    pub(super) quality_scale: f32,
    pub(super) target_frame_time: f32,
    sys_refresh_timer: f32,
    mem_pressure: f32,
    cpu_load: f32,
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
        }
    }

    fn spawn_new_firefly(&mut self, cols: usize, rows: usize) {
        let size = self.rng.next_range(0.0, 4.0) as u8;
        let speed_mult = self.rng.next_range(0.7, 1.3);

        // library 4.0: pull from the canonical ScreenPalette.
        let accent = query_current_palette().accent;
        let (acc_h, _acc_s, _acc_l) = rgb_to_hsl(accent.0, accent.1, accent.2);
        let p = self.rng.next_f32();
        let h = if p < 0.4 {
            (acc_h + self.rng.next_range(-15.0, 15.0)).rem_euclid(360.0)
        } else if p < 0.7 {
            (acc_h + 120.0 + self.rng.next_range(-15.0, 15.0)).rem_euclid(360.0)
        } else {
            (acc_h - 120.0 + self.rng.next_range(-15.0, 15.0)).rem_euclid(360.0)
        };
        let color = hsl_to_rgb(h, 0.95, 0.60);

        // Spawn on the border of the screen to make it feel like they fly in
        let side = self.rng.next_usize(4);
        let (x, y) = match side {
            0 => (0.0, self.rng.next_range(0.0, rows as f32)), // Left
            1 => (cols as f32 - 1.0, self.rng.next_range(0.0, rows as f32)), // Right
            2 => (self.rng.next_range(0.0, cols as f32), 0.0), // Top
            _ => (self.rng.next_range(0.0, cols as f32), rows as f32 - 1.0), // Bottom
        };

        self.fireflies.push(Firefly {
            x,
            y,
            vx: self.rng.next_range(-3.0, 3.0),
            vy: self.rng.next_range(-3.0, 3.0),
            color,
            size,
            speed_mult,
            history: Vec::new(),
        });
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
            let accent = query_current_palette().accent;
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

        // Dynamically adjust fireflies to match target capacity
        let num_fireflies = (((cols * rows) / 45).clamp(30, 60) as f32 * self.quality_scale * (if self.on_battery { 0.55 } else { 1.0 })) as usize;
        if self.fireflies.len() > num_fireflies {
            self.fireflies.truncate(num_fireflies);
        } else if self.fireflies.len() < num_fireflies && num_fireflies > 0 {
            while self.fireflies.len() < num_fireflies {
                self.spawn_new_firefly(cols, rows);
            }
        }

        // Dynamically adjust star population to match target capacity
        let target_stars = (((cols * rows) / 25).clamp(30, 120) as f32 * self.quality_scale * (if self.on_battery { 0.55 } else { 1.0 })) as usize;
        if self.stars.len() > target_stars {
            self.stars.truncate(target_stars);
        } else if self.stars.len() < target_stars && target_stars > 0 {
            while self.stars.len() < target_stars {
                let ch = if self.stars.len() % 8 == 0 { '✦' } else if self.stars.len() % 3 == 0 { '+' } else { '.' };
                self.stars.push(Star {
                    x: self.rng.next_f32(),
                    y: self.rng.next_f32(),
                    phase: self.rng.next_f32() * std::f32::consts::TAU,
                    ch,
                    excitation: 0.0,
                });
            }
        }

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
mod tests {
    use super::*;

    #[test]
    fn test_gnats_new() {
        let gnats = Gnats::new();
        assert_eq!(gnats.fireflies.len(), 0);
        assert_eq!(gnats.attractors.len(), 0);
        assert_eq!(gnats.stars.len(), 0);
        assert_eq!(gnats.kill_sparks.len(), 0);
        assert_eq!(gnats.time_elapsed, 0.0);
    }

    #[test]
    fn test_gnats_update_and_draw() {
        let mut gnats = Gnats::new();
        gnats.update(Duration::from_millis(16), 80, 24);
        let mut grid = vec![TerminalCell::default(); 80 * 24];
        gnats.draw(&mut grid, 80, 24);
        // Ensure state variables get initialized
        assert_eq!(gnats.last_cols, 80);
        assert_eq!(gnats.last_rows, 24);
        assert!(!gnats.fireflies.is_empty());
        assert!(!gnats.stars.is_empty());
        assert!(!gnats.attractors.is_empty());
    }
}

