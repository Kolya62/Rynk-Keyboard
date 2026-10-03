use super::canvas::{Canvas, Color};

#[derive(Clone, Debug)]
pub struct RippleAnimation {
    pub cx: f32,
    pub cy: f32,
    pub max_radius: f32,
    pub start_time_ms: u64,
    pub duration_ms: u64,
    pub color: Color,
}

impl RippleAnimation {
    pub fn new(cx: f32, cy: f32, max_radius: f32, start_time_ms: u64, color: Color) -> Self {
        Self {
            cx,
            cy,
            max_radius,
            start_time_ms,
            duration_ms: 200,
            color,
        }
    }

    pub fn render(&self, canvas: &mut Canvas, current_time_ms: u64) -> bool {
        if current_time_ms < self.start_time_ms {
            return false;
        }

        let elapsed = current_time_ms - self.start_time_ms;
        if elapsed >= self.duration_ms {
            return true; // Finished
        }

        let t = elapsed as f32 / self.duration_ms as f32;
        // Ease out quad
        let progress = 1.0 - (1.0 - t) * (1.0 - t);
        let radius = self.max_radius * progress;
        let alpha = (self.color.a as f32 * (1.0 - t)) as u8;

        canvas.draw_ripple(self.cx, self.cy, radius, self.color.with_alpha(alpha));
        false
    }
}

pub struct AnimationManager {
    pub ripples: Vec<RippleAnimation>,
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self {
            ripples: Vec::with_capacity(8),
        }
    }
}

impl AnimationManager {
    pub fn add_ripple(&mut self, cx: f32, cy: f32, radius: f32, time_ms: u64, color: Color) {
        self.ripples.push(RippleAnimation::new(cx, cy, radius, time_ms, color));
    }

    pub fn render_and_update(&mut self, canvas: &mut Canvas, current_time_ms: u64) {
        self.ripples.retain(|ripple| !ripple.render(canvas, current_time_ms));
    }

    pub fn has_active_animations(&self) -> bool {
        !self.ripples.is_empty()
    }
}
