#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ScrollState {
    y: u16,
    x: u16,
    max_y: u16,
    max_x: u16,
}

impl ScrollState {
    pub(crate) fn y(self) -> u16 {
        self.y
    }

    pub(crate) fn x(self) -> u16 {
        self.x
    }

    pub(crate) fn reset_position(&mut self) {
        self.y = 0;
        self.x = 0;
    }

    pub(crate) fn reset_all(&mut self) {
        self.reset_position();
        self.max_y = 0;
        self.max_x = 0;
    }

    pub(crate) fn update_limits(&mut self, max_y: u16, max_x: u16) {
        self.max_y = max_y;
        self.max_x = max_x;
        self.clamp_to_limits();
    }

    pub(crate) fn set_y(&mut self, y: u16) {
        self.y = y.min(self.max_y);
    }

    pub(crate) fn move_up(&mut self, lines: u16) {
        self.y = self.y.saturating_sub(lines);
    }

    pub(crate) fn move_down(&mut self, lines: u16) {
        self.y = self.y.saturating_add(lines).min(self.max_y);
    }

    pub(crate) fn move_left(&mut self, cols: u16) {
        self.x = self.x.saturating_sub(cols);
    }

    pub(crate) fn move_right(&mut self, cols: u16) {
        self.x = self.x.saturating_add(cols).min(self.max_x);
    }

    pub(crate) fn move_left_edge(&mut self) {
        self.x = 0;
    }

    pub(crate) fn move_right_edge(&mut self) {
        self.x = self.max_x;
    }

    fn clamp_to_limits(&mut self) {
        self.y = self.y.min(self.max_y);
        self.x = self.x.min(self.max_x);
    }
}
