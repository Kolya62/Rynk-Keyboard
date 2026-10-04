use super::key::{Key, KeyAction};
use super::state::HapticFeedbackType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchAction {
    Down = 0,
    Up = 1,
    Move = 2,
    Cancel = 3,
}

#[derive(Clone, Debug)]
pub struct PointerState {
    pub id: i32,
    pub start_x: f32,
    pub start_y: f32,
    pub current_x: f32,
    pub current_y: f32,
    pub start_time_ms: u64,
    pub active_key_id: Option<u32>,
    pub is_long_pressed: bool,
    pub selected_alternate_index: Option<usize>,
    pub is_spacebar_drag: bool,
    pub space_last_drag_x: f32,
    pub has_dragged_cursor: bool,
    pub has_swiped_language: bool,
    pub is_backspace_drag: bool,
    pub has_swiped_backspace: bool,
}

pub struct TouchTracker {
    pub pointers: Vec<PointerState>,
    pub long_press_threshold_ms: u64,
    pub space_drag_threshold_px: f32,
    pub backspace_swipe_threshold_px: f32,
}

impl Default for TouchTracker {
    fn default() -> Self {
        Self {
            pointers: Vec::with_capacity(4),
            long_press_threshold_ms: 350,
            space_drag_threshold_px: 15.0,
            backspace_swipe_threshold_px: 35.0,
        }
    }
}

pub enum TouchResult {
    None,
    KeyPress {
        key_id: u32,
        haptic: HapticFeedbackType,
    },
    KeyRelease {
        key_id: u32,
        action: KeyAction,
    },
    AlternateKeyRelease {
        character: char,
    },
    LongPressTriggered {
        key_id: u32,
        alternates: Vec<char>,
    },
    CursorMove {
        delta: i32,
    },
    DeleteWordSwipe,
    SwitchLanguageSwipe {
        is_next: bool,
    },
}

impl TouchTracker {
    pub fn handle_touch(
        &mut self,
        action: TouchAction,
        pointer_id: i32,
        x: f32,
        y: f32,
        time_ms: u64,
        keys: &[Key],
    ) -> TouchResult {
        match action {
            TouchAction::Down => {
                let hit_key = keys.iter().find(|k| k.contains(x, y)).or_else(|| {
                    // Fallback to nearest key by Euclidean distance to center
                    keys.iter().min_by(|a, b| {
                        let (ax, ay) = a.center();
                        let (bx, by) = b.center();
                        let dist_a = (x - ax).powi(2) + (y - ay).powi(2);
                        let dist_b = (x - bx).powi(2) + (y - by).powi(2);
                        dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
                    })
                });
                let active_key_id = hit_key.map(|k| k.id);
                let is_space = hit_key.is_some_and(|k| matches!(k.action, KeyAction::Space));
                let is_backspace =
                    hit_key.is_some_and(|k| matches!(k.action, KeyAction::Backspace));

                self.pointers.retain(|p| p.id != pointer_id);
                self.pointers.push(PointerState {
                    id: pointer_id,
                    start_x: x,
                    start_y: y,
                    current_x: x,
                    current_y: y,
                    start_time_ms: time_ms,
                    active_key_id,
                    is_long_pressed: false,
                    selected_alternate_index: None,
                    is_spacebar_drag: is_space,
                    space_last_drag_x: x,
                    has_dragged_cursor: false,
                    has_swiped_language: false,
                    is_backspace_drag: is_backspace,
                    has_swiped_backspace: false,
                });

                if let Some(key) = hit_key {
                    let haptic = match key.action {
                        KeyAction::Character(_) => HapticFeedbackType::KeyClick,
                        KeyAction::Shift | KeyAction::Backspace | KeyAction::Enter => {
                            HapticFeedbackType::KeyHeavyClick
                        }
                        _ => HapticFeedbackType::KeyTick,
                    };
                    return TouchResult::KeyPress {
                        key_id: key.id,
                        haptic,
                    };
                }
                TouchResult::None
            }

            TouchAction::Move => {
                if let Some(pointer) = self.pointers.iter_mut().find(|p| p.id == pointer_id) {
                    pointer.current_x = x;
                    pointer.current_y = y;

                    // Spacebar swipe left/right to switch language layout
                    if pointer.is_spacebar_drag && !pointer.has_swiped_language {
                        let total_dx = x - pointer.start_x;
                        if total_dx.abs() >= 40.0 {
                            pointer.has_swiped_language = true;
                            pointer.has_dragged_cursor = true;
                            return TouchResult::SwitchLanguageSwipe {
                                is_next: total_dx > 0.0,
                            };
                        }
                    }

                    // Spacebar cursor drag
                    if pointer.is_spacebar_drag && !pointer.has_swiped_language {
                        let dx = x - pointer.space_last_drag_x;
                        if dx.abs() >= self.space_drag_threshold_px {
                            let steps = (dx / self.space_drag_threshold_px) as i32;
                            pointer.space_last_drag_x = x;
                            pointer.has_dragged_cursor = true;
                            return TouchResult::CursorMove { delta: steps };
                        }
                        return TouchResult::None;
                    }

                    // Backspace swipe left to delete word
                    if pointer.is_backspace_drag && !pointer.has_swiped_backspace {
                        let dx = x - pointer.start_x;
                        if dx <= -self.backspace_swipe_threshold_px {
                            pointer.has_swiped_backspace = true;
                            return TouchResult::DeleteWordSwipe;
                        }
                    }

                    // Long press alternate selection
                    if pointer.is_long_pressed {
                        if let Some(key_id) = pointer.active_key_id {
                            if let Some(key) = keys.iter().find(|k| k.id == key_id) {
                                if !key.alternate_chars.is_empty() {
                                    let num_alts = key.alternate_chars.len();
                                    let item_w = (key.width * 1.1).max(38.0);
                                    let total_w = num_alts as f32 * item_w;
                                    let mut start_x = key.center().0 - total_w * 0.5;
                                    if start_x < 6.0 {
                                        start_x = 6.0;
                                    }
                                    let rel_x = x - start_x;
                                    let idx = if rel_x <= 0.0 {
                                        0
                                    } else {
                                        ((rel_x / item_w) as usize).min(num_alts - 1)
                                    };
                                    pointer.selected_alternate_index = Some(idx);
                                }
                            }
                        }
                    }
                }
                TouchResult::None
            }

            TouchAction::Up => {
                let pointer = self.pointers.iter().find(|p| p.id == pointer_id).cloned();
                self.pointers.retain(|p| p.id != pointer_id);

                if let Some(p) = pointer {
                    // Suppress tap release if spacebar was swiped or dragged
                    if p.has_swiped_language || p.has_dragged_cursor {
                        return TouchResult::None;
                    }

                    // Suppress tap release if backspace was swiped to delete word
                    if p.has_swiped_backspace {
                        return TouchResult::None;
                    }

                    // Alternate character released
                    if p.is_long_pressed {
                        if let Some(idx) = p.selected_alternate_index {
                            if let Some(key_id) = p.active_key_id {
                                if let Some(key) = keys.iter().find(|k| k.id == key_id) {
                                    if let Some(&ch) = key.alternate_chars.get(idx) {
                                        return TouchResult::AlternateKeyRelease { character: ch };
                                    }
                                }
                            }
                        }
                        return TouchResult::None;
                    }

                    // Normal tap release
                    if let Some(key_id) = p.active_key_id {
                        if let Some(key) = keys.iter().find(|k| k.id == key_id) {
                            if key.contains(p.current_x, p.current_y) || !p.is_long_pressed {
                                return TouchResult::KeyRelease {
                                    key_id: key.id,
                                    action: key.action,
                                };
                            }
                        }
                    }
                }
                TouchResult::None
            }

            TouchAction::Cancel => {
                self.pointers.retain(|p| p.id != pointer_id);
                TouchResult::None
            }
        }
    }

    pub fn check_long_press(&mut self, current_time_ms: u64, keys: &[Key]) -> Option<TouchResult> {
        for pointer in self.pointers.iter_mut() {
            if !pointer.is_long_pressed && !pointer.is_spacebar_drag && !pointer.is_backspace_drag {
                let elapsed = current_time_ms.saturating_sub(pointer.start_time_ms);
                if elapsed >= self.long_press_threshold_ms {
                    let dist_sq = (pointer.current_x - pointer.start_x).powi(2)
                        + (pointer.current_y - pointer.start_y).powi(2);
                    if dist_sq < 600.0 {
                        // < 24px movement
                        if let Some(key_id) = pointer.active_key_id {
                            if let Some(key) = keys.iter().find(|k| k.id == key_id) {
                                if !key.alternate_chars.is_empty() {
                                    pointer.is_long_pressed = true;
                                    pointer.selected_alternate_index = Some(0);
                                    return Some(TouchResult::LongPressTriggered {
                                        key_id,
                                        alternates: key.alternate_chars.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
