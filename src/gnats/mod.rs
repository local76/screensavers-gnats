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
use library::toolkit::sys_info::query_current_palette;
use library::toolkit::rgb_controller::{RgbController, is_openrgb_enabled};
use library::toolkit::rgb_protocol::RgbColor;

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
    rgb: Option<RgbController>,
    rgb_timer: f32,
}

impl Default for Gnats {
    fn default() -> Self {
        Self::new()
    }
}

impl Gnats {
    pub fn new() -> Self {
        let rng = LcgRng::new(1357);
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
            rgb: if is_openrgb_enabled() { Some(RgbController::new()) } else { None },
            rgb_timer: 0.0,
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
        let delta = dt.as_secs_f32().min(0.1);
        self.time_elapsed += delta;

        // OpenRGB drift updates
        self.rgb_timer += delta;
        if self.rgb_timer >= 0.15 {
            self.rgb_timer = 0.0;
            if let Some(ref r) = self.rgb {
                if self.fireflies.len() >= 4 {
                    // 5: Keyboard
                    let c0 = self.fireflies[0].color;
                    r.set_device_color(5, RgbColor::new(c0.0, c0.1, c0.2));

                    // 6: Mouse
                    let c1 = self.fireflies[1].color;
                    r.set_device_color(6, RgbColor::new(c1.0, c1.1, c1.2));

                    // 12: Speaker
                    let c2 = self.fireflies[2].color;
                    r.set_device_color(12, RgbColor::new(c2.0, c2.1, c2.2));

                    // 0, 1, 2: Motherboard, RAM, GPU
                    let c3 = self.fireflies[3].color;
                    let m_color = RgbColor::new(c3.0, c3.1, c3.2);
                    r.set_device_color(0, m_color);
                    r.set_device_color(1, m_color);
                    r.set_device_color(2, m_color);
                } else if !self.fireflies.is_empty() {
                    let c0 = self.fireflies[0].color;
                    r.set_color(RgbColor::new(c0.0, c0.1, c0.2));
                }
            }
        }

        // Initialize particles and attractors if grid size changes
        if cols != self.last_cols || rows != self.last_rows {
            self.last_cols = cols;
            self.last_rows = rows;

            // library 4.1: fixed-size logo excitation buffer (pre-4.1
            // `trance_core::logo_dimensions()` was a Windows file read).
            self.logo_excitation = vec![0.0; 80 * 12];

            // library 4.0: pull from the canonical ScreenPalette.
            let accent = query_current_palette().accent;
            let (acc_h, _acc_s, _acc_l) = rgb_to_hsl(accent.0, accent.1, accent.2);

            // Recreate fireflies
            self.fireflies.clear();
            self.kill_sparks.clear();
            let num_fireflies = ((cols * rows) / 45).clamp(30, 60);
            for _ in 0..num_fireflies {
                let size = self.rng.next_range(0.0, 4.0) as u8;
                let speed_mult = self.rng.next_range(0.7, 1.3);

                // Select a harmonious neon color using triadic accent color schemes
                let p = self.rng.next_f32();
                let h = if p < 0.4 {
                    (acc_h + self.rng.next_range(-15.0, 15.0)).rem_euclid(360.0)
                } else if p < 0.7 {
                    (acc_h + 120.0 + self.rng.next_range(-15.0, 15.0)).rem_euclid(360.0)
                } else {
                    (acc_h - 120.0 + self.rng.next_range(-15.0, 15.0)).rem_euclid(360.0)
                };
                let color = hsl_to_rgb(h, 0.95, 0.60);

                self.fireflies.push(Firefly {
                    x: self.rng.next_range(0.0, cols as f32),
                    y: self.rng.next_range(0.0, rows as f32),
                    vx: self.rng.next_range(-5.0, 5.0),
                    vy: self.rng.next_range(-5.0, 5.0),
                    color,
                    size,
                    speed_mult,
                    history: Vec::new(),
                });
            }

            // Recreate stars
            self.stars.clear();
            let target_stars = ((cols * rows) / 25).clamp(30, 120);
            for i in 0..target_stars {
                let ch = if i % 8 == 0 { '✦' } else if i % 3 == 0 { '+' } else { '.' };
                self.stars.push(Star {
                    x: self.rng.next_f32(),
                    y: self.rng.next_f32(),
                    phase: self.rng.next_f32() * std::f32::consts::TAU,
                    ch,
                    excitation: 0.0,
                });
            }

            // Recreate attractors
            self.attractors.clear();
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
}
