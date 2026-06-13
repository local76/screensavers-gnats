use crate::runner::core::LcgRng;
use super::types::{Firefly, Attractor};
pub use super::physics_helpers::{
    update_attractors, decay_logo_excitations, update_kill_sparks,
    update_stars, update_logo_excitations
};

pub fn compute_firefly_forces_and_update(
    fireflies: &mut [Firefly],
    attractors: &[Attractor],
    time_elapsed: f32,
    cols_f: f32,
    rows_f: f32,
    rng: &mut LcgRng,
    delta: f32,
) -> Vec<usize> {
    let num_fireflies = fireflies.len();
    let mut dead_indices = Vec::new();
    let mut forces = vec![(0.0f32, 0.0f32); num_fireflies];

    for (i, force) in forces.iter_mut().enumerate() {
        let mut fx = 0.0f32;
        let mut fy = 0.0f32;

        // Pull towards orbit attractors
        for attr in attractors {
            let dx = attr.x - fireflies[i].x;
            let dy = attr.y - fireflies[i].y;
            let dist_sq = dx * dx + dy * dy;
            let inv_dist = 1.0 / dist_sq.sqrt().max(0.1);
            
            let pull = 45.0 / (dist_sq + 20.0);
            let pull_over_d = pull * inv_dist;
            fx += dx * pull_over_d;
            fy += dy * pull_over_d;
        }

        // Faint pull towards screen center
        let cx = cols_f / 2.0;
        let cy = rows_f / 2.0;
        let dx = cx - fireflies[i].x;
        let dy = cy - fireflies[i].y;
        let dist_sq = dx * dx + dy * dy;
        let inv_dist = 1.0 / dist_sq.sqrt().max(0.1);
        let center_pull = 15.0 / (dist_sq + 60.0);
        let f_over_d = center_pull * inv_dist;
        fx += dx * f_over_d;
        fy += dy * f_over_d;

        // Flow wind fields
        let wind_x = (time_elapsed * 0.35 + fireflies[i].y * 0.08).cos() * 0.35;
        let wind_y = (time_elapsed * 0.45 + fireflies[i].x * 0.06).sin() * 0.25;
        fx += wind_x;
        fy += wind_y;

        // Faint random jitter
        let rx = rng.next_range(-0.5, 0.5);
        let ry = rng.next_range(-0.5, 0.5);
        fx += rx;
        fy += ry;

        // Predator-prey logic
        let mut closest_prey_dist = f32::MAX;
        let mut closest_predator_dist = f32::MAX;
        let mut prey_dx = 0.0;
        let mut prey_dy = 0.0;
        let mut pred_dx = 0.0;
        let mut pred_dy = 0.0;
        let mut prey_idx = None;

        for j in 0..num_fireflies {
            if i == j { continue; }
            let dx_j = fireflies[j].x - fireflies[i].x;
            let dy_j = fireflies[j].y - fireflies[i].y;
            let dist_sq_j = dx_j * dx_j + dy_j * dy_j;
            let dist_j = dist_sq_j.sqrt().max(0.1);

            if fireflies[j].size < fireflies[i].size {
                // Larger fireflies chase smaller fireflies
                if dist_j < closest_prey_dist {
                    closest_prey_dist = dist_j;
                    prey_dx = dx_j;
                    prey_dy = dy_j;
                    prey_idx = Some(j);
                }
            } else if fireflies[j].size > fireflies[i].size {
                // Smaller fireflies run away from larger fireflies
                if dist_j < closest_predator_dist {
                    closest_predator_dist = dist_j;
                    pred_dx = dx_j;
                    pred_dy = dy_j;
                }
            }
        }

        // Apply chase force
        if closest_prey_dist < f32::MAX {
            let force_chase = 55.0 / (closest_prey_dist + 4.5);
            let force_chase_over_d = force_chase / closest_prey_dist;
            fx += prey_dx * force_chase_over_d;
            fy += prey_dy * force_chase_over_d;

            // Mark prey for death if close enough
            if closest_prey_dist < 1.1 {
                if let Some(idx) = prey_idx {
                    dead_indices.push(idx);
                }
            }
        }

        // Apply flee force
        if closest_predator_dist < f32::MAX {
            let force_flee = 75.0 / (closest_predator_dist + 2.5);
            let force_flee_over_d = force_flee / closest_predator_dist;
            fx -= pred_dx * force_flee_over_d;
            fy -= pred_dy * force_flee_over_d;
        }

        *force = (fx, fy);
    }

    // Apply forces to velocity and position
    for (p, &(fx, fy)) in fireflies.iter_mut().zip(forces.iter()) {
        p.vx += fx * delta * 24.0 * p.speed_mult;
        p.vy += fy * delta * 24.0 * p.speed_mult;
        p.vx *= 1.0 - (delta * 1.8);
        p.vy *= 1.0 - (delta * 1.8);

        let speed_sq = p.vx * p.vx + p.vy * p.vy;
        let max_speed = 36.0;
        let max_speed_sq = max_speed * max_speed;
        if speed_sq > max_speed_sq {
            let inv_speed = 1.0 / speed_sq.sqrt().max(1e-6);
            let factor = inv_speed * max_speed;
            p.vx *= factor;
            p.vy *= factor;
        }

        p.x += p.vx * delta;
        p.y += p.vy * delta;

        // Wall bounces
        if p.x < 0.0 {
            p.x = 0.0;
            p.vx = -p.vx * 0.7;
        } else if p.x >= cols_f {
            p.x = cols_f - 1.0;
            p.vx = -p.vx * 0.7;
        }
        if p.y < 0.0 {
            p.y = 0.0;
            p.vy = -p.vy * 0.7;
        } else if p.y >= rows_f {
            p.y = rows_f - 1.0;
            p.vy = -p.vy * 0.7;
        }

        // Save coordinate history trail
        let cell_x = p.x.round() as i32;
        let cell_y = p.y.round() as i32;
        if p.history.is_empty() || p.history.last() != Some(&(cell_x, cell_y)) {
            p.history.push((cell_x, cell_y));
            if p.history.len() > 5 {
                p.history.remove(0);
            }
        }
    }

    dead_indices
}



#[cfg(test)]
#[path = "physics_tests.rs"]
mod tests;

