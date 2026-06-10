use library::core::TerminalCell;
use library::toolkit::sys_info::query_current_palette;
use library::core::logo_block::render_logo_block;
use library::platform::native::sys_info::get_system_info;
use super::Gnats;

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
    for star in &gnats.stars {
        let sx = (star.x * cols as f32) as usize;
        let sy = (star.y * rows as f32) as usize;
        if sx < cols && sy < rows {
            let sparkle = ((gnats.time_elapsed * 2.5 + star.phase).sin() + 1.0) * 0.5;
            let brightness = (sparkle * 0.35 + star.excitation * 0.65).min(1.0);
            
            let (star_r, star_g, star_b) = if star.excitation > 0.05 {
                // Blend excited star color with registry accent color
                let blend = star.excitation.min(1.0);
                (
                    (160.0 * (1.0 - blend) + accent.0 as f32 * blend).min(255.0) as u8,
                    (180.0 * (1.0 - blend) + accent.1 as f32 * blend).min(255.0) as u8,
                    (220.0 * (1.0 - blend) + accent.2 as f32 * blend).min(255.0) as u8,
                )
            } else {
                // Soft blue-ish white stars twinkling
                (
                    (110.0 + brightness * 70.0) as u8,
                    (120.0 + brightness * 75.0) as u8,
                    (140.0 + brightness * 80.0) as u8,
                )
            };

            let ch = if star.excitation > 0.8 {
                '✹'
            } else if star.excitation > 0.4 {
                '✦'
            } else {
                star.ch
            };

            grid[sy * cols + sx] = TerminalCell {
                ch,
                fg: (star_r, star_g, star_b),
                bg: (0, 0, 0),
                bold: star.excitation > 0.4 || brightness > 0.7,
            };

            // Draw anamorphic lens flares on excited stars
            if star.excitation > 0.6 {
                let flare_intensity = ((star.excitation - 0.6) / 0.4).min(1.0);

                // Horizontal lens flare streak
                let h_len = 6;
                for dx in 1..h_len {
                    let alpha = (90.0 * flare_intensity) as u8;
                    let fade = alpha.saturating_sub((dx * (80 / h_len)) as u8);
                    if fade > 5 {
                        let f_color = (
                            (star_r as f32 * 0.5 + fade as f32 * 0.5).min(255.0) as u8,
                            (star_g as f32 * 0.5 + fade as f32 * 0.5).min(255.0) as u8,
                            (star_b as f32 * 0.5 + fade as f32 * 0.5).min(255.0) as u8,
                        );
                        if sx + dx < cols {
                            let c = &mut grid[sy * cols + (sx + dx)];
                            if c.ch == ' ' {
                                c.ch = '─';
                                c.fg = f_color;
                                c.bold = false;
                            }
                        }
                        if sx >= dx {
                            let c = &mut grid[sy * cols + (sx - dx)];
                            if c.ch == ' ' {
                                c.ch = '─';
                                c.fg = f_color;
                                c.bold = false;
                            }
                        }
                    }
                }

                // Vertical lens flare streak
                let v_len = 3;
                for dy in 1..v_len {
                    let alpha = (70.0 * flare_intensity) as u8;
                    let fade = alpha.saturating_sub((dy * (60 / v_len)) as u8);
                    if fade > 5 {
                        let f_color = (
                            (star_r as f32 * 0.5 + fade as f32 * 0.5).min(255.0) as u8,
                            (star_g as f32 * 0.5 + fade as f32 * 0.5).min(255.0) as u8,
                            (star_b as f32 * 0.5 + fade as f32 * 0.5).min(255.0) as u8,
                        );
                        if sy + dy < rows {
                            let c = &mut grid[(sy + dy) * cols + sx];
                            if c.ch == ' ' {
                                c.ch = '│';
                                c.fg = f_color;
                                c.bold = false;
                            }
                        }
                        if sy >= dy {
                            let c = &mut grid[(sy - dy) * cols + sx];
                            if c.ch == ' ' {
                                c.ch = '│';
                                c.fg = f_color;
                                c.bold = false;
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Draw wireframe network connector lines
    let max_connect_dist = 11.5f32;
    let num_fireflies = gnats.fireflies.len();
    for i in 0..num_fireflies {
        for j in (i + 1)..num_fireflies {
            let dx = gnats.fireflies[j].x - gnats.fireflies[i].x;
            let dy = gnats.fireflies[j].y - gnats.fireflies[i].y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < max_connect_dist * max_connect_dist {
                let dist = dist_sq.sqrt();
                let x1 = gnats.fireflies[i].x;
                let y1 = gnats.fireflies[i].y;
                let x2 = gnats.fireflies[j].x;
                let y2 = gnats.fireflies[j].y;
                let ldx = x2 - x1;
                let ldy = y2 - y1;
                let steps = ldx.abs().max(ldy.abs()).round() as i32;

                if steps > 1 {
                    let x_step = ldx / steps as f32;
                    let y_step = ldy / steps as f32;

                    let adx = ldx.abs();
                    let ady = ldy.abs();
                    let ch = if ady < 0.4142 * adx {
                        '─'
                    } else if ady > 2.4142 * adx {
                        '│'
                    } else {
                        if ldx * ldy > 0.0 { '╲' } else { '╱' }
                    };

                    let color1 = gnats.fireflies[i].color;
                    let color2 = gnats.fireflies[j].color;
                    let intensity = (1.0 - dist / max_connect_dist).clamp(0.0, 1.0) * 0.45; // softer connections

                    let line_r = (((color1.0 as f32 + color2.0 as f32) * 0.5) * intensity) as u8;
                    let line_g = (((color1.1 as f32 + color2.1 as f32) * 0.5) * intensity) as u8;
                    let line_b = (((color1.2 as f32 + color2.2 as f32) * 0.5) * intensity) as u8;

                    for k in 1..steps {
                        let lx = (x1 + k as f32 * x_step).round() as i32;
                        let ly = (y1 + k as f32 * y_step).round() as i32;
                        if lx >= 0 && lx < cols as i32 && ly >= 0 && ly < rows as i32 {
                            let idx = ly as usize * cols + lx as usize;
                            if grid[idx].ch == ' ' {
                                grid[idx] = TerminalCell {
                                    ch,
                                    fg: (line_r, line_g, line_b),
                                    bg: (0, 0, 0),
                                    bold: intensity > 0.25,
                                };
                            }
                        }
                    }
                }
            }
        }
    }

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
    let logo_text = get_system_info().logo_text;
    let lines = render_logo_block(&logo_text, None);
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
