use winit::dpi::PhysicalPosition;

/// Tracks window drag state for moving the pet around the screen.
pub struct DragState {
    dragging: bool,
    /// Last known cursor position (updated on every CursorMoved)
    last_cursor: PhysicalPosition<f64>,
}

impl DragState {
    pub fn new() -> Self {
        Self {
            dragging: false,
            last_cursor: PhysicalPosition::new(0.0, 0.0),
        }
    }

    /// Start a drag. Call this on MouseInput Pressed.
    pub fn start_drag(&mut self) {
        self.dragging = true;
    }

    /// End a drag. Call this on MouseInput Released.
    pub fn end_drag(&mut self) {
        self.dragging = false;
    }

    /// Update cursor tracking. Always call this on CursorMoved.
    /// If dragging, returns the new window position (delta-based).
    pub fn on_cursor_moved(
        &mut self,
        cursor_pos: PhysicalPosition<f64>,
        current_window_pos: PhysicalPosition<i32>,
    ) -> Option<PhysicalPosition<i32>> {
        let delta_x = cursor_pos.x - self.last_cursor.x;
        let delta_y = cursor_pos.y - self.last_cursor.y;

        // Always update last_cursor so it's accurate when drag starts
        self.last_cursor = cursor_pos;

        if !self.dragging {
            return None;
        }

        Some(PhysicalPosition::new(
            current_window_pos.x + delta_x as i32,
            current_window_pos.y + delta_y as i32,
        ))
    }

    #[allow(dead_code)]
    pub fn is_dragging(&self) -> bool {
        self.dragging
    }
}

impl Default for DragState {
    fn default() -> Self {
        Self::new()
    }
}
