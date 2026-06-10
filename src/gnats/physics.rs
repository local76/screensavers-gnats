use library::core::LcgRng;
use super::types::{Firefly, Attractor, Star, KillSpark};

pub fn update_attractors(attractors: &mut [Attractor], time_elapsed: f32, cols_f: f32, rows_f: f32) {
    if attractors.len() >= 3 {
        let cx = cols_f / 2.0;
        let cy = rows_f / 2.0;

        // Attractor 0
        let t0 = time_elapsed * attractors[0].speed + attractors[0].phase;
        attractors[0].x = cx + t0.cos() * (cols_f * 0.35);
        attractors[0].y = cy + (t0 * 2.0).sin() * (rows_f * 0.30);

        // Attractor 1
        let t1 = time_elapsed * attractors[1].speed + attractors[1].phase;
        attractors[1].x = cx + (t1 * 1.5).sin() * (cols_f * 0.40);
        attractors[1].y = cy + t1.cos() * (rows_f * 0.35);

        // Attractor 2
        let t2 = time_elapsed * attractors[2].speed + attractors[2].phase;
        attractors[2].x = cx + t2.cos() * (cols_f * 0.28);
        attractors[2].y = cy + (t2 * 1.8).cos() * (rows_f * 0.28);
    }
}

pub fn decay_logo_excitations(logo_excitation: &mut [f32], delta: f32) {
    for exc in logo_excitation {
        *exc = (*exc - 1.8 * delta).max(0.0);
    }
}

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
            let dist = dist_sq.sqrt().max(0.1);
            
            let pull = 45.0 / (dist_sq + 20.0);
            fx += (dx / dist) * pull;
            fy += (dy / dist) * pull;
        }

        // Faint pull towards screen center
        let cx = cols_f / 2.0;
        let cy = rows_f / 2.0;
        let dx = cx - fireflies[i].x;
        let dy = cy - fireflies[i].y;
        let dist_sq = dx * dx + dy * dy;
        let dist = dist_sq.sqrt().max(0.1);
        let center_pull = 15.0 / (dist_sq + 60.0);
        fx += (dx / dist) * center_pull;
        fy += (dy / dist) * center_pull;

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
            fx += (prey_dx / closest_prey_dist) * force_chase;
            fy += (prey_dy / closest_prey_dist) * force_chase;

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
            fx -= (pred_dx / closest_predator_dist) * force_flee;
            fy -= (pred_dy / closest_predator_dist) * force_flee;
        }

        *force = (fx, fy);
    }

    // Apply forces to velocity and position
    for (p, &(fx, fy)) in fireflies.iter_mut().zip(forces.iter()) {
        p.vx += fx * delta * 24.0 * p.speed_mult;
        p.vy += fy * delta * 24.0 * p.speed_mult;
        p.vx *= 1.0 - (delta * 1.8);
        p.vy *= 1.0 - (delta * 1.8);

        let speed = (p.vx * p.vx + p.vy * p.vy).sqrt();
        let max_speed = 36.0;
        if speed > max_speed {
            p.vx = (p.vx / speed) * max_speed;
            p.vy = (p.vy / speed) * max_speed;
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

pub fn update_kill_sparks(kill_sparks: &mut Vec<KillSpark>, delta: f32) {
    for spark in kill_sparks.iter_mut() {
        spark.x += spark.vx * delta;
        spark.y += spark.vy * delta;
        spark.life -= delta * 2.0;
    }
    kill_sparks.retain(|s| s.life > 0.0);
}

pub fn update_stars(stars: &mut [Star], fireflies: &[Firefly], delta: f32, cols_f: f32, rows_f: f32) {
    for star in stars.iter_mut() {
        star.excitation = (star.excitation - 1.2 * delta).max(0.0);
    }
    for p in fireflies {
        for star in stars.iter_mut() {
            let dx = p.x - star.x * cols_f;
            let dy = (p.y - star.y * rows_f) * 2.0;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < 9.0 {
                let dist = dist_sq.sqrt();
                let force = (1.0 - dist / 3.0).max(0.0) * 1.5;
                star.excitation = star.excitation.max(force);
            }
        }
    }
}

pub fn update_logo_excitations(logo_excitation: &mut [f32], fireflies: &[Firefly], cols: usize, rows: usize) {
    let logo_w: usize = 80;
    let logo_h: usize = 12;
    if logo_w > 0 && logo_h > 0 && logo_excitation.len() == logo_w * logo_h {
        let logo_x = cols.saturating_sub(logo_w) / 2;
        let logo_y = rows.saturating_sub(logo_h) / 2;

        for p in fireflies {
            let px = p.x.round() as i32;
            let py = p.y.round() as i32;
            if px >= logo_x as i32 && px < (logo_x + logo_w) as i32 &&
               py >= logo_y as i32 && py < (logo_y + logo_h) as i32 {
                let lx = px as usize - logo_x;
                let ly = py as usize - logo_y;
                let l_idx = ly * logo_w + lx;
                if l_idx < logo_excitation.len() {
                    logo_excitation[l_idx] = 1.0;
                }
            }
        }
    }
}
