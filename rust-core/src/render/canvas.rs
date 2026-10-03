#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn transparent() -> Self {
        Self { r: 0, g: 0, b: 0, a: 0 }
    }

    pub fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    /// Converts to Android ARGB_8888 little-endian u32 in memory (0xAABBGGRR)
    #[inline(always)]
    pub fn to_u32(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Linear interpolation between two colors
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t_clamped = t.clamp(0.0, 1.0);
        let inv_t = 1.0 - t_clamped;
        Self {
            r: (self.r as f32 * inv_t + other.r as f32 * t_clamped) as u8,
            g: (self.g as f32 * inv_t + other.g as f32 * t_clamped) as u8,
            b: (self.b as f32 * inv_t + other.b as f32 * t_clamped) as u8,
            a: (self.a as f32 * inv_t + other.a as f32 * t_clamped) as u8,
        }
    }

    #[inline(always)]
    pub fn blend(src: Color, dst: Color) -> Color {
        if src.a == 255 {
            return src;
        }
        if src.a == 0 {
            return dst;
        }

        let sa = src.a as u32;
        let inv_sa = 255 - sa;

        let r = ((src.r as u32 * sa + dst.r as u32 * inv_sa) / 255) as u8;
        let g = ((src.g as u32 * sa + dst.g as u32 * inv_sa) / 255) as u8;
        let b = ((src.b as u32 * sa + dst.b as u32 * inv_sa) / 255) as u8;
        let a = (sa + (dst.a as u32 * inv_sa) / 255).min(255) as u8;

        Color { r, g, b, a }
    }
}

pub struct Canvas<'a> {
    pub pixels: &'a mut [u32],
    pub width: usize,
    pub height: usize,
    pub stride: usize,
}

impl<'a> Canvas<'a> {
    pub fn new(pixels: &'a mut [u32], width: usize, height: usize, stride: usize) -> Self {
        Self {
            pixels,
            width,
            height,
            stride,
        }
    }

    #[inline(always)]
    pub fn clear(&mut self, color: Color) {
        let val = color.to_u32();
        for row in 0..self.height {
            let start = row * self.stride;
            let end = start + self.width;
            self.pixels[start..end].fill(val);
        }
    }

    #[inline(always)]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x < self.width && y < self.height {
            let idx = y * self.stride + x;
            if color.a == 255 {
                self.pixels[idx] = color.to_u32();
            } else if color.a > 0 {
                let existing = self.pixels[idx];
                let dst = Color {
                    a: (existing >> 24) as u8,
                    r: (existing >> 16) as u8,
                    g: (existing >> 8) as u8,
                    b: existing as u8,
                };
                self.pixels[idx] = Color::blend(color, dst).to_u32();
            }
        }
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        let x0 = (x.max(0.0) as usize).min(self.width);
        let y0 = (y.max(0.0) as usize).min(self.height);
        let x1 = ((x + w).max(0.0) as usize).min(self.width);
        let y1 = ((y + h).max(0.0) as usize).min(self.height);

        if color.a == 255 {
            let val = color.to_u32();
            for py in y0..y1 {
                let row_start = py * self.stride;
                self.pixels[row_start + x0..row_start + x1].fill(val);
            }
        } else if color.a > 0 {
            for py in y0..y1 {
                for px in x0..x1 {
                    self.set_pixel(px, py, color);
                }
            }
        }
    }

    #[inline(always)]
    pub fn fill_circle(&mut self, cx: f32, cy: f32, radius: f32, color: Color) {
        if radius <= 0.0 || color.a == 0 {
            return;
        }
        let r_ceil = radius.ceil() as isize;
        let cy_i = cy.round() as isize;
        let r_sq = radius * radius;

        let y0 = (cy_i - r_ceil).max(0) as usize;
        let y1 = (cy_i + r_ceil + 1).min(self.height as isize).max(0) as usize;

        for py in y0..y1 {
            let dy = py as f32 + 0.5 - cy;
            let dy_sq = dy * dy;
            if dy_sq > r_sq {
                continue;
            }
            let dx_max = (r_sq - dy_sq).sqrt();
            let x0 = ((cx - dx_max).round() as isize).max(0) as usize;
            let x1 = ((cx + dx_max + 1.0).round() as usize).min(self.width);
            for px in x0..x1 {
                self.set_pixel(px, py, color);
            }
        }
    }

    /// Fast scanline anti-aliased rounded rectangle
    pub fn fill_rounded_rect(&mut self, x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
        if w <= 0.0 || h <= 0.0 || color.a == 0 {
            return;
        }

        let r = radius.min(w * 0.5).min(h * 0.5).max(0.0);
        if r <= 0.5 {
            self.fill_rect(x, y, w, h, color);
            return;
        }

        let y0 = (y.floor() as isize).max(0) as usize;
        let y1 = ((y + h).ceil() as usize).min(self.height);
        if y0 >= y1 {
            return;
        }

        let half_w = w * 0.5;
        let half_h = h * 0.5;
        let center_x = x + half_w;
        let center_y = y + half_h;
        let inner_half_w = half_w - r;
        let inner_half_h = half_h - r;
        let r_sq = r * r;
        let val = color.to_u32();

        for py in y0..y1 {
            let dy = (py as f32 + 0.5) - center_y;
            let abs_dy = dy.abs();

            let span_half_w = if abs_dy <= inner_half_h {
                half_w
            } else {
                let q_y = abs_dy - inner_half_h;
                if q_y >= r {
                    continue;
                }
                inner_half_w + (r_sq - q_y * q_y).max(0.0).sqrt()
            };

            let left_f = center_x - span_half_w;
            let right_f = center_x + span_half_w;

            let row_start = py * self.stride;

            // Fully interior pixel interval
            let inner_x0 = (left_f + 0.5).ceil() as usize;
            let inner_x1 = (right_f - 0.5).floor() as usize;

            let rx0 = inner_x0.min(self.width);
            let rx1 = inner_x1.min(self.width);

            if rx0 < rx1 {
                if color.a == 255 {
                    self.pixels[row_start + rx0..row_start + rx1].fill(val);
                } else {
                    for px in rx0..rx1 {
                        self.set_pixel(px, py, color);
                    }
                }
            }

            // Anti-aliased boundary edges
            let edge_left = inner_x0.saturating_sub(1);
            if edge_left < self.width {
                let cov = (inner_x0 as f32 - left_f).clamp(0.0, 1.0);
                let edge_a = (color.a as f32 * cov) as u8;
                if edge_a > 0 {
                    self.set_pixel(edge_left, py, color.with_alpha(edge_a));
                }
            }

            let edge_right = inner_x1;
            if edge_right < self.width {
                let cov = (right_f - inner_x1 as f32).clamp(0.0, 1.0);
                let edge_a = (color.a as f32 * cov) as u8;
                if edge_a > 0 {
                    self.set_pixel(edge_right, py, color.with_alpha(edge_a));
                }
            }
        }
    }

    /// Renders a fast tactile elevation drop shadow under a key
    #[allow(clippy::too_many_arguments)]
    pub fn draw_drop_shadow(&mut self, x: f32, y: f32, w: f32, h: f32, radius: f32, blur: f32, shadow_color: Color) {
        if shadow_color.a == 0 {
            return;
        }
        let offset_y = blur * 0.6;
        let shadow_y = y + h;
        let shadow_h = offset_y + 1.2;
        let shadow_r = radius * 0.8;
        // Key bottom edge shadow: subtle tactile lip beneath the key bottom
        self.fill_rounded_rect(
            x + 1.0,
            shadow_y - 2.0,
            (w - 2.0).max(0.0),
            shadow_h + 2.0,
            shadow_r,
            shadow_color,
        );
    }

    /// Draws an expanding ripple animation circle
    pub fn draw_ripple(&mut self, cx: f32, cy: f32, radius: f32, color: Color) {
        if radius <= 0.5 || color.a == 0 {
            return;
        }

        let y0 = ((cy - radius).floor() as isize).max(0) as usize;
        let y1 = ((cy + radius).ceil() as usize).min(self.height);
        let r_sq = radius * radius;

        for py in y0..y1 {
            let dy = py as f32 + 0.5 - cy;
            let dy_sq = dy * dy;
            if dy_sq >= r_sq {
                continue;
            }
            let dx_max = (r_sq - dy_sq).sqrt();
            let x0 = ((cx - dx_max).floor() as isize).max(0) as usize;
            let x1 = ((cx + dx_max).ceil() as usize).min(self.width);
            for px in x0..x1 {
                self.set_pixel(px, py, color);
            }
        }
    }
}
