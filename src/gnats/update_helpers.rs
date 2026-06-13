use super::{Gnats, Firefly, Star};
use crate::runner::core::{hsl_to_rgb, rgb_to_hsl};

impl Gnats {
    pub(crate) fn spawn_new_firefly(&mut self, cols: usize, rows: usize) {
        let size = self.rng.next_range(0.0, 4.0) as u8;
        let speed_mult = self.rng.next_range(0.7, 1.3);

        // library 4.0: pull from the canonical ScreenPalette.
        let accent = self.accent;
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

    pub(crate) fn adjust_populations(&mut self, cols: usize, rows: usize) {
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
    }
}
