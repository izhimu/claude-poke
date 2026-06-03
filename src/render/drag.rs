use winit::dpi::PhysicalPosition;

/// Tracks window drag state for moving the pet around the screen.
pub struct DragState {
    dragging: bool,
    /// Cursor position in window coords at the moment of mouse press
    drag_start: PhysicalPosition<f64>,
    /// Window position in screen coords at the moment of mouse press
    window_start: PhysicalPosition<i32>,
    /// Last known cursor position (updated on every CursorMoved)
    last_cursor: PhysicalPosition<f64>,
}

impl DragState {
    pub fn new() -> Self {
        Self {
            dragging: false,
            drag_start: PhysicalPosition::new(0.0, 0.0),
            window_start: PhysicalPosition::new(0, 0),
            last_cursor: PhysicalPosition::new(0.0, 0.0),
        }
    }

    /// Start a drag. Call this on MouseInput Pressed.
    /// `window_pos`: current outer position of the window.
    pub fn start_drag(&mut self, window_pos: PhysicalPosition<i32>) {
        self.dragging = true;
        self.drag_start = self.last_cursor;
        self.window_start = window_pos;
    }

    /// End a drag. Call this on MouseInput Released.
    pub fn end_drag(&mut self) {
        self.dragging = false;
    }

    /// If dragging, compute the new window position based on cursor movement.
    /// `cursor_pos`: cursor position in window coords.
    pub fn on_cursor_moved(
        &mut self,
        cursor_pos: PhysicalPosition<f64>,
    ) -> Option<PhysicalPosition<i32>> {
        self.last_cursor = cursor_pos;

        if !self.dragging {
            return None;
        }

        let dx = cursor_pos.x - self.drag_start.x;
        let dy = cursor_pos.y - self.drag_start.y;

        Some(PhysicalPosition::new(
            (self.window_start.x as f64 + dx) as i32,
            (self.window_start.y as f64 + dy) as i32,
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
