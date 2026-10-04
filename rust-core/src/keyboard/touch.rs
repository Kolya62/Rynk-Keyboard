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
    /// Space bar held or dragged: horizontal movement moves the text cursor
    pub cursor_mode: bool,
    /// Words selected for deletion by dragging left from backspace
    pub backspace_words: u32,
}

pub struct TouchTracker {
    pub pointers: Vec<PointerState>,
    pub long_press_threshold_ms: u64,
    pub space_drag_threshold_px: f32,
    pub backspace_swipe_threshold_px: f32,
    pub space_swipe_threshold_px: f32,
    pub long_press_slop_sq: f32,
    pub density: f32,
    /// A quick space swipe switches language; when off, swiping moves the cursor
    pub space_swipe_switches_language: bool,
    /// Drag distance per cursor step / per selected word
    pub cursor_step_px: f32,
    pub word_step_px: f32,
}

impl Default for TouchTracker {
    fn default() -> Self {
        Self::with_density(1.0)
    }
}

impl TouchTracker {
    pub fn with_density(density: f32) -> Self {
        let d = density.max(0.5);
        Self {
            pointers: Vec::with_capacity(4),
            long_press_threshold_ms: 350,
            space_drag_threshold_px: (12.0 * d).clamp(12.0, 25.0),
            backspace_swipe_threshold_px: (24.0 * d).clamp(24.0, 70.0),
            space_swipe_threshold_px: (16.0 * d).clamp(16.0, 35.0),
            long_press_slop_sq: (18.0 * d).clamp(18.0, 36.0).powi(2),
            density: d,
            space_swipe_switches_language: true,
            cursor_step_px: 9.0 * d,
            word_step_px: 28.0 * d,
        }
    }

    pub fn update_density(&mut self, density: f32) {
        let d = density.max(0.5);
        self.density = d;
        self.space_drag_threshold_px = (12.0 * d).clamp(12.0, 25.0);
        self.backspace_swipe_threshold_px = (24.0 * d).clamp(24.0, 70.0);
        self.space_swipe_threshold_px = (16.0 * d).clamp(16.0, 35.0);
        self.long_press_slop_sq = (18.0 * d).clamp(18.0, 36.0).powi(2);
        self.cursor_step_px = 9.0 * d;
        self.word_step_px = 28.0 * d;
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
    /// Space bar held long enough: dragging now moves the cursor
    CursorModeStarted,
    /// Dragging left from backspace selects this many words before the cursor (0: none)
    BackspaceSelect {
        words: u32,
    },
    /// Released after selecting words from backspace: delete them
    BackspaceSelectCommit,
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
                    cursor_mode: false,
                    backspace_words: 0,
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

                    if pointer.is_spacebar_drag && pointer.cursor_mode {
                        let steps = ((x - pointer.space_last_drag_x) / self.cursor_step_px).trunc() as i32;
                        if steps != 0 {
                            pointer.space_last_drag_x += steps as f32 * self.cursor_step_px;
                            return TouchResult::CursorMove { delta: steps };
                        }
                        return TouchResult::None;
                    }

                    // Space bar: a quick swipe switches language (if enabled), otherwise dragging
                    // moves the cursor
                    if pointer.is_spacebar_drag && !pointer.has_swiped_language {
                        let total_dx = x - pointer.start_x;
                        if total_dx.abs() >= self.space_swipe_threshold_px {
                            if self.space_swipe_switches_language {
                                pointer.has_swiped_language = true;
                                return TouchResult::SwitchLanguageSwipe { is_next: total_dx > 0.0 };
                            }
                            pointer.cursor_mode = true;
                            pointer.has_dragged_cursor = true;
                            pointer.space_last_drag_x = pointer.start_x;
                            return TouchResult::CursorModeStarted;
                        }
                    }

                    // Backspace: dragging left selects whole words, one per step
                    if pointer.is_backspace_drag {
                        let dx = pointer.start_x - x;
                        let words = if dx < self.backspace_swipe_threshold_px {
                            0
                        } else {
                            1 + ((dx - self.backspace_swipe_threshold_px) / self.word_step_px) as u32
                        };
                        if words != pointer.backspace_words {
                            pointer.backspace_words = words;
                            pointer.has_swiped_backspace = true;
                            return TouchResult::BackspaceSelect { words };
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
                    // Suppress tap release if spacebar was swiped
                    if p.has_swiped_language {
                        return TouchResult::None;
                    }

                    if p.cursor_mode || p.has_dragged_cursor {
                        return TouchResult::None;
                    }

                    // Backspace drag: delete the selected words, or nothing if dragged back
                    if p.has_swiped_backspace {
                        return if p.backspace_words > 0 { TouchResult::BackspaceSelectCommit } else { TouchResult::None };
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
                            return TouchResult::KeyRelease {
                                key_id: key.id,
                                action: key.action,
                            };
                        }
                    }
                }
                TouchResult::None
            }

            TouchAction::Cancel => {
                let had_selection = self.pointers.iter().any(|p| p.id == pointer_id && p.backspace_words > 0);
                self.pointers.retain(|p| p.id != pointer_id);
                if had_selection {
                    TouchResult::BackspaceSelect { words: 0 }
                } else {
                    TouchResult::None
                }
            }
        }
    }

    pub fn check_long_press(&mut self, current_time_ms: u64, keys: &[Key]) -> Option<TouchResult> {
        for pointer in self.pointers.iter_mut() {
            // Holding the space bar still turns it into a cursor control
            if pointer.is_spacebar_drag && !pointer.cursor_mode && !pointer.has_swiped_language {
                let elapsed = current_time_ms.saturating_sub(pointer.start_time_ms);
                let dist_sq = (pointer.current_x - pointer.start_x).powi(2) + (pointer.current_y - pointer.start_y).powi(2);
                if elapsed >= self.long_press_threshold_ms && dist_sq < self.long_press_slop_sq {
                    pointer.cursor_mode = true;
                    pointer.space_last_drag_x = pointer.current_x;
                    return Some(TouchResult::CursorModeStarted);
                }
            }
            if !pointer.is_long_pressed && !pointer.is_spacebar_drag && !pointer.is_backspace_drag {
                let elapsed = current_time_ms.saturating_sub(pointer.start_time_ms);
                if elapsed >= self.long_press_threshold_ms {
                    let dist_sq = (pointer.current_x - pointer.start_x).powi(2)
                        + (pointer.current_y - pointer.start_y).powi(2);
                    if dist_sq < self.long_press_slop_sq {
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
