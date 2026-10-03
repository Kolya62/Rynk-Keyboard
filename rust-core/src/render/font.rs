use super::canvas::{Canvas, Color};

pub struct FontRenderer;

impl FontRenderer {
    /// Draws a string centered at (cx, cy)
    pub fn draw_text_centered(
        canvas: &mut Canvas,
        text: &str,
        cx: f32,
        cy: f32,
        font_size: f32,
        color: Color,
    ) {
        let total_w = Self::measure_text(text, font_size);
        let mut curr_x = cx - total_w * 0.5;

        for ch in text.chars() {
            let cw = Self::char_width(ch, font_size);
            Self::draw_glyph(canvas, ch, curr_x + cw * 0.5, cy, font_size, color);
            curr_x += cw;
        }
    }

    /// Draws a string starting at (x, cy)
    pub fn draw_text_left(
        canvas: &mut Canvas,
        text: &str,
        x: f32,
        cy: f32,
        font_size: f32,
        color: Color,
    ) {
        let mut curr_x = x;
        for ch in text.chars() {
            let cw = Self::char_width(ch, font_size);
            Self::draw_glyph(canvas, ch, curr_x + cw * 0.5, cy, font_size, color);
            curr_x += cw;
        }
    }

    pub fn measure_text(text: &str, font_size: f32) -> f32 {
        text.chars().map(|c| Self::char_width(c, font_size)).sum()
    }

    pub fn char_width(ch: char, font_size: f32) -> f32 {
        match ch {
            ' ' => font_size * 0.45,
            '.' | ',' | '\'' | '!' | ':' | ';' | '|' | 'i' | 'l' => font_size * 0.35,
            'm' | 'w' | 'M' | 'W' | 'ж' | 'ш' | 'щ' | 'ю' | 'Ж' | 'Ш' | 'Щ' | 'Ю' => {
                font_size * 0.85
            }
            '⇧' | '⬆' | '⇪' | '⌫' | '↵' => font_size * 0.85,
            '🌐' | '😊' | '⚙' => font_size * 0.95,
            _ => font_size * 0.60,
        }
    }

    /// Renders a single glyph with anti-aliased vector strokes
    pub fn draw_glyph(
        canvas: &mut Canvas,
        ch: char,
        cx: f32,
        cy: f32,
        font_size: f32,
        color: Color,
    ) {
        let s = font_size;
        let stroke_w = (s * 0.12).max(1.5);

        match ch {
            // UI Icons
            '⇧' => {
                // Shift arrow (outline)
                Self::draw_icon_shift(canvas, cx, cy, s * 0.85, stroke_w, color);
            }
            '⬆' => {
                // Shift arrow (solid filled)
                Self::draw_icon_shift_solid(canvas, cx, cy, s * 0.85, stroke_w, color);
            }
            '⇪' => {
                // Caps Lock arrow with solid horizontal base bar
                Self::draw_icon_caps_lock(canvas, cx, cy, s * 0.85, stroke_w, color);
            }
            '⌫' => {
                // Backspace badge
                Self::draw_icon_backspace(canvas, cx, cy, s * 0.85, stroke_w, color);
            }
            '↵' => {
                // Enter arrow
                Self::draw_icon_enter(canvas, cx, cy, s * 0.85, stroke_w, color);
            }
            '🌐' => {
                // Globe icon
                Self::draw_icon_globe(canvas, cx, cy, s * 0.85, stroke_w, color);
            }
            '😊' => {
                // Emoji smiley
                Self::draw_icon_smiley(canvas, cx, cy, s * 0.85, stroke_w, color);
            }
            // Standard Letters & Numerals via vector segments
            _ => {
                Self::render_standard_char(canvas, ch, cx, cy, s, stroke_w, color);
            }
        }
    }

    fn draw_line(
        canvas: &mut Canvas,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        width: f32,
        color: Color,
    ) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt();
        let radius = width * 0.5;
        if len <= 0.5 {
            canvas.fill_circle(x0, y0, radius, color);
            return;
        }

        let steps = len.ceil() as usize;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let px = x0 + dx * t;
            let py = y0 + dy * t;
            canvas.fill_circle(px, py, radius, color);
        }
    }

    fn draw_icon_shift(
        canvas: &mut Canvas,
        cx: f32,
        cy: f32,
        size: f32,
        stroke: f32,
        color: Color,
    ) {
        let h = size * 0.5;
        let w = size * 0.45;
        // Peak of arrow
        let top_x = cx;
        let top_y = cy - h * 0.8;
        let left_x = cx - w;
        let left_y = cy;
        let right_x = cx + w;
        let right_y = cy;

        Self::draw_line(canvas, top_x, top_y, left_x, left_y, stroke, color);
        Self::draw_line(canvas, top_x, top_y, right_x, right_y, stroke, color);
        Self::draw_line(
            canvas,
            left_x,
            left_y,
            left_x + w * 0.4,
            left_y,
            stroke,
            color,
        );
        Self::draw_line(
            canvas,
            right_x,
            right_y,
            right_x - w * 0.4,
            right_y,
            stroke,
            color,
        );
        Self::draw_line(
            canvas,
            left_x + w * 0.4,
            left_y,
            left_x + w * 0.4,
            cy + h * 0.7,
            stroke,
            color,
        );
        Self::draw_line(
            canvas,
            right_x - w * 0.4,
            right_y,
            right_x - w * 0.4,
            cy + h * 0.7,
            stroke,
            color,
        );
        Self::draw_line(
            canvas,
            left_x + w * 0.4,
            cy + h * 0.7,
            right_x - w * 0.4,
            cy + h * 0.7,
            stroke,
            color,
        );
    }

    fn draw_icon_shift_solid(
        canvas: &mut Canvas,
        cx: f32,
        cy: f32,
        size: f32,
        stroke: f32,
        color: Color,
    ) {
        let h = size * 0.50;
        let w = size * 0.44;

        let top_y = cy - h * 0.85;
        let wing_y = cy - h * 0.05;
        let shaft_bot_y = cy + h * 0.75;
        let shaft_w = w * 0.72;

        let tri_steps = ((wing_y - top_y).abs() * 2.0).ceil() as usize;
        for i in 0..=tri_steps {
            let t = i as f32 / tri_steps as f32;
            let y = top_y + (wing_y - top_y) * t;
            let span = w * t;
            canvas.fill_rect(cx - span, y, span * 2.0, 1.5, color);
        }

        canvas.fill_rect(
            cx - shaft_w * 0.5,
            wing_y,
            shaft_w,
            shaft_bot_y - wing_y,
            color,
        );

        Self::draw_line(canvas, cx, top_y, cx - w, wing_y, stroke, color);
        Self::draw_line(canvas, cx, top_y, cx + w, wing_y, stroke, color);
        Self::draw_line(
            canvas,
            cx - w,
            wing_y,
            cx - shaft_w * 0.5,
            wing_y,
            stroke,
            color,
        );
        Self::draw_line(
            canvas,
            cx + w,
            wing_y,
            cx + shaft_w * 0.5,
            wing_y,
            stroke,
            color,
        );
        Self::draw_line(
            canvas,
            cx - shaft_w * 0.5,
            wing_y,
            cx - shaft_w * 0.5,
            shaft_bot_y,
            stroke,
            color,
        );
        Self::draw_line(
            canvas,
            cx + shaft_w * 0.5,
            wing_y,
            cx + shaft_w * 0.5,
            shaft_bot_y,
            stroke,
            color,
        );
        Self::draw_line(
            canvas,
            cx - shaft_w * 0.5,
            shaft_bot_y,
            cx + shaft_w * 0.5,
            shaft_bot_y,
            stroke,
            color,
        );
    }

    fn draw_icon_caps_lock(
        canvas: &mut Canvas,
        cx: f32,
        cy: f32,
        size: f32,
        stroke: f32,
        color: Color,
    ) {
        let h = size * 0.52;
        let w = size * 0.44;

        // Top arrow head
        let top_y = cy - h * 0.88;
        let wing_y = cy - h * 0.18;
        let tri_steps = ((wing_y - top_y).abs() * 2.0).ceil() as usize;
        for i in 0..=tri_steps {
            let t = i as f32 / tri_steps as f32;
            let y = top_y + (wing_y - top_y) * t;
            let span = w * t;
            canvas.fill_rect(cx - span, y, span * 2.0, 1.5, color);
        }

        // Shaft
        let shaft_bot_y = cy + h * 0.35;
        let shaft_w = w * 0.72;
        canvas.fill_rect(
            cx - shaft_w * 0.5,
            wing_y,
            shaft_w,
            shaft_bot_y - wing_y,
            color,
        );

        Self::draw_line(canvas, cx, top_y, cx - w, wing_y, stroke, color);
        Self::draw_line(canvas, cx, top_y, cx + w, wing_y, stroke, color);
        Self::draw_line(
            canvas,
            cx - w,
            wing_y,
            cx - shaft_w * 0.5,
            wing_y,
            stroke,
            color,
        );
        Self::draw_line(
            canvas,
            cx + w,
            wing_y,
            cx + shaft_w * 0.5,
            wing_y,
            stroke,
            color,
        );
        Self::draw_line(
            canvas,
            cx - shaft_w * 0.5,
            wing_y,
            cx - shaft_w * 0.5,
            shaft_bot_y,
            stroke,
            color,
        );
        Self::draw_line(
            canvas,
            cx + shaft_w * 0.5,
            wing_y,
            cx + shaft_w * 0.5,
            shaft_bot_y,
            stroke,
            color,
        );
        Self::draw_line(
            canvas,
            cx - shaft_w * 0.5,
            shaft_bot_y,
            cx + shaft_w * 0.5,
            shaft_bot_y,
            stroke,
            color,
        );

        // Distinct Bottom Horizontal Bar (Caps Lock base bar)
        let bar_y = cy + h * 0.72;
        let bar_h = (stroke * 1.35).max(3.0);
        let bar_w = w * 1.75;
        let bar_r = bar_h * 0.5;
        canvas.fill_rounded_rect(
            cx - bar_w * 0.5,
            bar_y - bar_h * 0.5,
            bar_w,
            bar_h,
            bar_r,
            color,
        );
    }

    fn draw_icon_backspace(
        canvas: &mut Canvas,
        cx: f32,
        cy: f32,
        size: f32,
        stroke: f32,
        color: Color,
    ) {
        let w = size * 0.65;
        let h = size * 0.45;
        let tip_x = cx - w;
        let body_l = cx - w * 0.4;
        let body_r = cx + w;
        let top_y = cy - h;
        let bot_y = cy + h;

        // Outline
        Self::draw_line(canvas, tip_x, cy, body_l, top_y, stroke, color);
        Self::draw_line(canvas, tip_x, cy, body_l, bot_y, stroke, color);
        Self::draw_line(canvas, body_l, top_y, body_r, top_y, stroke, color);
        Self::draw_line(canvas, body_l, bot_y, body_r, bot_y, stroke, color);
        Self::draw_line(canvas, body_r, top_y, body_r, bot_y, stroke, color);

        // 'x' inside
        let cross_size = h * 0.45;
        let cross_cx = cx + w * 0.2;
        Self::draw_line(
            canvas,
            cross_cx - cross_size,
            cy - cross_size,
            cross_cx + cross_size,
            cy + cross_size,
            stroke,
            color,
        );
        Self::draw_line(
            canvas,
            cross_cx - cross_size,
            cy + cross_size,
            cross_cx + cross_size,
            cy - cross_size,
            stroke,
            color,
        );
    }

    fn draw_icon_enter(
        canvas: &mut Canvas,
        cx: f32,
        cy: f32,
        size: f32,
        stroke: f32,
        color: Color,
    ) {
        let w = size * 0.5;
        let h = size * 0.4;
        // Arrow tip
        let tip_x = cx - w * 0.6;
        let tip_y = cy + h * 0.3;
        Self::draw_line(
            canvas,
            tip_x,
            tip_y,
            tip_x + w * 0.4,
            tip_y - h * 0.4,
            stroke,
            color,
        );
        Self::draw_line(
            canvas,
            tip_x,
            tip_y,
            tip_x + w * 0.4,
            tip_y + h * 0.4,
            stroke,
            color,
        );
        // Shaft curving up
        Self::draw_line(canvas, tip_x, tip_y, cx + w * 0.5, tip_y, stroke, color);
        Self::draw_line(
            canvas,
            cx + w * 0.5,
            tip_y,
            cx + w * 0.5,
            cy - h * 0.6,
            stroke,
            color,
        );
    }

    fn draw_icon_globe(
        canvas: &mut Canvas,
        cx: f32,
        cy: f32,
        size: f32,
        stroke: f32,
        color: Color,
    ) {
        let r = size * 0.45;
        // Outer ring
        let steps = 24;
        for i in 0..steps {
            let a0 = (i as f32 / steps as f32) * std::f32::consts::TAU;
            let a1 = ((i + 1) as f32 / steps as f32) * std::f32::consts::TAU;
            Self::draw_line(
                canvas,
                cx + a0.cos() * r,
                cy + a0.sin() * r,
                cx + a1.cos() * r,
                cy + a1.sin() * r,
                stroke,
                color,
            );
        }
        // Equator
        Self::draw_line(canvas, cx - r, cy, cx + r, cy, stroke, color);
        // Meridian
        Self::draw_line(canvas, cx, cy - r, cx, cy + r, stroke, color);
    }

    fn draw_icon_smiley(
        canvas: &mut Canvas,
        cx: f32,
        cy: f32,
        size: f32,
        stroke: f32,
        color: Color,
    ) {
        let r = size * 0.45;
        let steps = 24;
        for i in 0..steps {
            let a0 = (i as f32 / steps as f32) * std::f32::consts::TAU;
            let a1 = ((i + 1) as f32 / steps as f32) * std::f32::consts::TAU;
            Self::draw_line(
                canvas,
                cx + a0.cos() * r,
                cy + a0.sin() * r,
                cx + a1.cos() * r,
                cy + a1.sin() * r,
                stroke,
                color,
            );
        }
        // Eyes
        let eye_r = stroke * 0.8;
        canvas.fill_rounded_rect(
            cx - r * 0.35 - eye_r,
            cy - r * 0.25 - eye_r,
            eye_r * 2.0,
            eye_r * 2.0,
            eye_r,
            color,
        );
        canvas.fill_rounded_rect(
            cx + r * 0.35 - eye_r,
            cy - r * 0.25 - eye_r,
            eye_r * 2.0,
            eye_r * 2.0,
            eye_r,
            color,
        );
        // Smile
        let smile_steps = 10;
        for i in 0..smile_steps {
            let a0 = std::f32::consts::PI * 0.15
                + (i as f32 / smile_steps as f32) * std::f32::consts::PI * 0.7;
            let a1 = std::f32::consts::PI * 0.15
                + ((i + 1) as f32 / smile_steps as f32) * std::f32::consts::PI * 0.7;
            let sr = r * 0.55;
            Self::draw_line(
                canvas,
                cx + a0.cos() * sr,
                cy + a0.sin() * sr * 0.6 + r * 0.1,
                cx + a1.cos() * sr,
                cy + a1.sin() * sr * 0.6 + r * 0.1,
                stroke,
                color,
            );
        }
    }

    fn render_standard_char(
        canvas: &mut Canvas,
        ch: char,
        cx: f32,
        cy: f32,
        size: f32,
        stroke: f32,
        color: Color,
    ) {
        let w = size * 0.35;
        let h = size * 0.45;
        let l = cx - w;
        let r = cx + w;
        let t = cy - h;
        let b = cy + h;
        let m_x = cx;
        let m_y = cy;

        match ch.to_ascii_lowercase() {
            'a' | 'а' => {
                Self::draw_line(canvas, l, b, m_x, t, stroke, color);
                Self::draw_line(canvas, r, b, m_x, t, stroke, color);
                Self::draw_line(
                    canvas,
                    l + w * 0.3,
                    m_y + h * 0.2,
                    r - w * 0.3,
                    m_y + h * 0.2,
                    stroke,
                    color,
                );
            }
            'b' | 'в' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, l, t, m_x + w * 0.2, t, stroke, color);
                Self::draw_line(canvas, m_x + w * 0.2, t, r, m_y - h * 0.1, stroke, color);
                Self::draw_line(canvas, r, m_y - h * 0.1, l, m_y, stroke, color);
                Self::draw_line(canvas, l, m_y, m_x + w * 0.3, m_y, stroke, color);
                Self::draw_line(canvas, m_x + w * 0.3, m_y, r, m_y + h * 0.5, stroke, color);
                Self::draw_line(canvas, r, m_y + h * 0.5, l, b, stroke, color);
            }
            'c' | 'с' => {
                Self::draw_line(canvas, r, t + h * 0.3, m_x, t, stroke, color);
                Self::draw_line(canvas, m_x, t, l, m_y, stroke, color);
                Self::draw_line(canvas, l, m_y, m_x, b, stroke, color);
                Self::draw_line(canvas, m_x, b, r, b - h * 0.3, stroke, color);
            }
            'd' | 'д' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, l, t, m_x, t, stroke, color);
                Self::draw_line(canvas, m_x, t, r, m_y, stroke, color);
                Self::draw_line(canvas, r, m_y, m_x, b, stroke, color);
                Self::draw_line(canvas, m_x, b, l, b, stroke, color);
            }
            'e' | 'е' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, l, t, r, t, stroke, color);
                Self::draw_line(canvas, l, m_y, m_x + w * 0.3, m_y, stroke, color);
                Self::draw_line(canvas, l, b, r, b, stroke, color);
            }
            'f' | 'ф' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, l, t, r, t, stroke, color);
                Self::draw_line(canvas, l, m_y, m_x + w * 0.3, m_y, stroke, color);
            }
            'g' | 'г' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, l, t, r, t, stroke, color);
            }
            'h' | 'н' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, r, t, r, b, stroke, color);
                Self::draw_line(canvas, l, m_y, r, m_y, stroke, color);
            }
            'i' | 'и' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, r, t, r, b, stroke, color);
                Self::draw_line(canvas, r, t, l, b, stroke, color);
            }
            'j' | 'й' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, r, t, r, b, stroke, color);
                Self::draw_line(canvas, r, t, l, b, stroke, color);
                // Brief stroke above
                Self::draw_line(
                    canvas,
                    m_x - w * 0.4,
                    t - h * 0.35,
                    m_x + w * 0.4,
                    t - h * 0.35,
                    stroke,
                    color,
                );
            }
            'k' | 'к' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, r, t, l, m_y, stroke, color);
                Self::draw_line(canvas, l, m_y, r, b, stroke, color);
            }
            'l' | 'л' => {
                Self::draw_line(canvas, l, b, m_x, t, stroke, color);
                Self::draw_line(canvas, m_x, t, r, b, stroke, color);
            }
            'm' | 'м' => {
                Self::draw_line(canvas, l, b, l, t, stroke, color);
                Self::draw_line(canvas, l, t, m_x, m_y, stroke, color);
                Self::draw_line(canvas, m_x, m_y, r, t, stroke, color);
                Self::draw_line(canvas, r, t, r, b, stroke, color);
            }
            'o' | 'о' => {
                Self::draw_line(canvas, l, t + h * 0.3, m_x, t, stroke, color);
                Self::draw_line(canvas, m_x, t, r, t + h * 0.3, stroke, color);
                Self::draw_line(canvas, r, t + h * 0.3, r, b - h * 0.3, stroke, color);
                Self::draw_line(canvas, r, b - h * 0.3, m_x, b, stroke, color);
                Self::draw_line(canvas, m_x, b, l, b - h * 0.3, stroke, color);
                Self::draw_line(canvas, l, b - h * 0.3, l, t + h * 0.3, stroke, color);
            }
            'p' | 'п' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, l, t, r, t, stroke, color);
                Self::draw_line(canvas, r, t, r, b, stroke, color);
            }
            'r' | 'р' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, l, t, r, t, stroke, color);
                Self::draw_line(canvas, r, t, r, m_y, stroke, color);
                Self::draw_line(canvas, r, m_y, l, m_y, stroke, color);
            }
            't' | 'т' => {
                Self::draw_line(canvas, l, t, r, t, stroke, color);
                Self::draw_line(canvas, m_x, t, m_x, b, stroke, color);
            }
            'u' | 'у' => {
                Self::draw_line(canvas, l, t, m_x, m_y + h * 0.2, stroke, color);
                Self::draw_line(canvas, r, t, l, b, stroke, color);
            }
            'x' | 'х' => {
                Self::draw_line(canvas, l, t, r, b, stroke, color);
                Self::draw_line(canvas, r, t, l, b, stroke, color);
            }
            'y' | 'ы' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, l, m_y, m_x - w * 0.1, m_y, stroke, color);
                Self::draw_line(canvas, m_x - w * 0.1, m_y, m_x - w * 0.1, b, stroke, color);
                Self::draw_line(canvas, m_x - w * 0.1, b, l, b, stroke, color);
                Self::draw_line(canvas, r, t, r, b, stroke, color);
            }
            'z' | 'з' => {
                Self::draw_line(canvas, l, t, r, t, stroke, color);
                Self::draw_line(canvas, r, t, m_x, m_y, stroke, color);
                Self::draw_line(canvas, m_x, m_y, r, b - h * 0.3, stroke, color);
                Self::draw_line(canvas, r, b - h * 0.3, l, b, stroke, color);
            }
            'ш' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, m_x, t, m_x, b, stroke, color);
                Self::draw_line(canvas, r, t, r, b, stroke, color);
                Self::draw_line(canvas, l, b, r, b, stroke, color);
            }
            'щ' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, m_x, t, m_x, b, stroke, color);
                Self::draw_line(canvas, r, t, r, b, stroke, color);
                Self::draw_line(canvas, l, b, r, b, stroke, color);
                Self::draw_line(canvas, r, b, r + w * 0.2, b + h * 0.3, stroke, color);
            }
            'ч' => {
                Self::draw_line(canvas, l, t, l, m_y, stroke, color);
                Self::draw_line(canvas, l, m_y, r, m_y, stroke, color);
                Self::draw_line(canvas, r, t, r, b, stroke, color);
            }
            'ц' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, r, t, r, b, stroke, color);
                Self::draw_line(canvas, l, b, r, b, stroke, color);
                Self::draw_line(canvas, r, b, r + w * 0.2, b + h * 0.3, stroke, color);
            }
            'ю' => {
                Self::draw_line(canvas, l, t, l, b, stroke, color);
                Self::draw_line(canvas, l, m_y, m_x, m_y, stroke, color);
                Self::draw_line(canvas, m_x, t, r, m_y, stroke, color);
                Self::draw_line(canvas, r, m_y, m_x, b, stroke, color);
                Self::draw_line(canvas, m_x, b, m_x, t, stroke, color);
            }
            'я' => {
                Self::draw_line(canvas, r, t, r, b, stroke, color);
                Self::draw_line(canvas, r, t, m_x, t, stroke, color);
                Self::draw_line(canvas, m_x, t, l, m_y * 0.8, stroke, color);
                Self::draw_line(canvas, l, m_y * 0.8, r, m_y, stroke, color);
                Self::draw_line(canvas, m_x, m_y, l, b, stroke, color);
            }
            '0' => {
                Self::draw_line(canvas, l, t + h * 0.3, m_x, t, stroke, color);
                Self::draw_line(canvas, m_x, t, r, t + h * 0.3, stroke, color);
                Self::draw_line(canvas, r, t + h * 0.3, r, b - h * 0.3, stroke, color);
                Self::draw_line(canvas, r, b - h * 0.3, m_x, b, stroke, color);
                Self::draw_line(canvas, m_x, b, l, b - h * 0.3, stroke, color);
                Self::draw_line(canvas, l, b - h * 0.3, l, t + h * 0.3, stroke, color);
            }
            '1' => {
                Self::draw_line(canvas, l + w * 0.2, t + h * 0.3, m_x, t, stroke, color);
                Self::draw_line(canvas, m_x, t, m_x, b, stroke, color);
                Self::draw_line(canvas, l, b, r, b, stroke, color);
            }
            '.' => {
                let dot_r = stroke * 1.1;
                canvas.fill_rounded_rect(
                    cx - dot_r,
                    b - dot_r * 2.0,
                    dot_r * 2.0,
                    dot_r * 2.0,
                    dot_r,
                    color,
                );
            }
            ',' => {
                let dot_r = stroke * 1.1;
                canvas.fill_rounded_rect(
                    cx - dot_r,
                    b - dot_r * 2.0,
                    dot_r * 2.0,
                    dot_r * 2.0,
                    dot_r,
                    color,
                );
                Self::draw_line(
                    canvas,
                    cx,
                    b - dot_r,
                    cx - dot_r,
                    b + dot_r * 1.5,
                    stroke * 0.8,
                    color,
                );
            }
            '?' => {
                Self::draw_line(canvas, l, t + h * 0.3, m_x, t, stroke, color);
                Self::draw_line(canvas, m_x, t, r, t + h * 0.3, stroke, color);
                Self::draw_line(canvas, r, t + h * 0.3, m_x, m_y, stroke, color);
                Self::draw_line(canvas, m_x, m_y, m_x, m_y + h * 0.3, stroke, color);
                let dot_r = stroke * 1.0;
                canvas.fill_rounded_rect(
                    cx - dot_r,
                    b - dot_r * 1.5,
                    dot_r * 2.0,
                    dot_r * 2.0,
                    dot_r,
                    color,
                );
            }
            '!' => {
                Self::draw_line(canvas, cx, t, cx, m_y + h * 0.3, stroke, color);
                let dot_r = stroke * 1.0;
                canvas.fill_rounded_rect(
                    cx - dot_r,
                    b - dot_r * 1.5,
                    dot_r * 2.0,
                    dot_r * 2.0,
                    dot_r,
                    color,
                );
            }
            '-' => {
                Self::draw_line(canvas, l, m_y, r, m_y, stroke, color);
            }
            '+' => {
                Self::draw_line(canvas, l, m_y, r, m_y, stroke, color);
                Self::draw_line(canvas, m_x, t + h * 0.3, m_x, b - h * 0.3, stroke, color);
            }
            '/' => {
                Self::draw_line(canvas, l, b, r, t, stroke, color);
            }
            _ => {
                // Fallback: draw neat rounded box
                canvas.fill_rounded_rect(l, t, w * 2.0, h * 2.0, stroke, color.with_alpha(100));
            }
        }
    }
}
