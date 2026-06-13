use crate::runner::core::TerminalCell;
use crate::runner::toolkit::sys_info::query_current_palette;
use crate::runner::core::logo_block::render_logo_block;
use super::Gnats;
use super::render_helpers::{draw_stars, draw_connectors};

pub fn draw_gnats(gnats: &Gnats, grid: &mut [TerminalCell], cols: usize, rows: usize) {
    if cols == 0 || rows == 0 {
        return;
    }

    // library 4.0: pull the canonical ScreenPalette so helm, pulse,
    // and the screensaver effects all share the same color story.
    // The accent is the primary color; hot/cool give us the triadic
    // accents that hand-rolled HSL math used to compute locally.
    let palette = query_current_palette();
    let accent = palette.accent;

    // 1. Clear grid (screen starts black)
    for cell in grid.iter_mut() {
        *cell = TerminalCell {
            ch: ' ',
            fg: (0, 0, 0),
            bg: (0, 0, 0),
            bold: false,
        };
    }

    // 1b. Draw distant backdrop stars with lens flares when excited by fireflies
    draw_stars(&gnats.stars, gnats.time_elapsed, cols, rows, accent, grid);

    // 2. Draw wireframe network connector lines
    draw_connectors(&gnats.fireflies, cols, rows, grid);

    // 3. Draw firefly history trails
    for p in &gnats.fireflies {
        let h_len = p.history.len();
        for (k, &(hx, hy)) in p.history.iter().enumerate() {
            if hx >= 0 && hx < cols as i32 && hy >= 0 && hy < rows as i32 {
                let idx = hy as usize * cols + hx as usize;
                if grid[idx].ch == ' ' {
                    // Trail fades as it goes further back in time
                    let t = (k + 1) as f32 / (h_len + 1) as f32;
                    let intensity = t * 0.35;
                    let tr = (p.color.0 as f32 * intensity) as u8;
                    let tg = (p.color.1 as f32 * intensity) as u8;
                    let tb = (p.color.2 as f32 * intensity) as u8;

                    grid[idx] = TerminalCell {
                        ch: '·',
                        fg: (tr, tg, tb),
                        bg: (0, 0, 0),
                        bold: false,
                    };
                }
            }
        }
    }

    // 3.5. Draw kill sparks
    for spark in &gnats.kill_sparks {
        let sx = spark.x.round() as i32;
        let sy = spark.y.round() as i32;
        if sx >= 0 && sx < cols as i32 && sy >= 0 && sy < rows as i32 {
            let idx = sy as usize * cols + sx as usize;
            if grid[idx].ch == ' ' || grid[idx].ch == '·' || grid[idx].ch == '─' || grid[idx].ch == '│' || grid[idx].ch == '╱' || grid[idx].ch == '╲' {
                grid[idx] = TerminalCell {
                    ch: '*',
                    fg: spark.color,
                    bg: (0, 0, 0),
                    bold: spark.life > 0.4,
                };
            }
        }
    }

    // 4. Draw fireflies themselves
    for p in &gnats.fireflies {
        let px = p.x.round() as i32;
        let py = p.y.round() as i32;
        if px >= 0 && px < cols as i32 && py >= 0 && py < rows as i32 {
            let idx = py as usize * cols + px as usize;
            let ch = match p.size {
                3 => '✦',
                2 => 'o',
                1 => '+',
                _ => '·',
            };
            grid[idx] = TerminalCell {
                ch,
                fg: p.color,
                bg: (0, 0, 0),
                bold: true,
            };
        }
    }

    // 5. Draw Attractors as faint pulsing halo flares
    for (i, attr) in gnats.attractors.iter().enumerate() {
        let ax = attr.x.round() as i32;
        let ay = attr.y.round() as i32;
        if ax >= 0 && ax < cols as i32 && ay >= 0 && ay < rows as i32 {
            let idx = ay as usize * cols + ax as usize;
            
            // Pulsing indicator char
            let pulse = (gnats.time_elapsed * 3.0 + i as f32 * 1.5).sin();
            let ch = if pulse > 0.5 { '¤' } else if pulse > -0.5 { '☼' } else { 'o' };
            
            // Soft color intensity
            let att_r = (attr.color.0 as f32 * 0.4) as u8;
            let att_g = (attr.color.1 as f32 * 0.4) as u8;
            let att_b = (attr.color.2 as f32 * 0.4) as u8;

            grid[idx] = TerminalCell {
                ch,
                fg: (att_r, att_g, att_b),
                bg: (0, 0, 0),
                bold: false,
            };
        }
    }

    // 6. Draw centered logo with glow excitation
    // library 4.1: render the system logo from the live OS info
    // (replaces pre-4.1 `trance_core::logo_lines()` + `logo_dimensions()`).
    let logo_text = &gnats.logo_text;
    let lines = render_logo_block(logo_text, None);
    let logo_h = lines.len();
    let logo_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    if logo_w > 0 && logo_h > 0 {
        let logo_x = cols.saturating_sub(logo_w) / 2;
        let logo_y = rows.saturating_sub(logo_h) / 2;

        for (r_offset, line) in lines.iter().enumerate().take(logo_h) {
            let gy = logo_y + r_offset;
            if gy >= rows {
                continue;
            }
            for (c_offset, ch) in line.chars().enumerate() {
                let gx = logo_x + c_offset;
                if gx >= cols {
                    continue;
                }
                if ch != ' ' {
                    let l_idx = r_offset * logo_w + c_offset;
                    let exc = gnats.logo_excitation.get(l_idx).copied().unwrap_or(0.0);
                    
                    let (fg, bold) = if exc > 0.05 {
                        let t = exc;
                        let r = (accent.0 as f32 * t + 255.0 * (1.0 - t)).min(255.0) as u8;
                        let g = (accent.1 as f32 * t + 255.0 * (1.0 - t)).min(255.0) as u8;
                        let b = (accent.2 as f32 * t + 255.0 * (1.0 - t)).min(255.0) as u8;
                        ((r, g, b), true)
                    } else {
                        (
                            (
                                (accent.0 as f32 * 0.25) as u8,
                                (accent.1 as f32 * 0.25) as u8,
                                (accent.2 as f32 * 0.25) as u8,
                            ),
                            false,
                        )
                    };
                    
                    grid[gy * cols + gx] = TerminalCell {
                        ch,
                        fg,
                        bg: (0, 0, 0),
                        bold,
                    };
                }
            }
        }
    }
}
