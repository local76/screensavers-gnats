use super::types::{Attractor, KillSpark, Star, Firefly};

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
