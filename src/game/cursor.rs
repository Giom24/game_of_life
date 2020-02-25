pub struct Cursor {
    x: i32,
    y: i32,
}

impl Cursor {
    pub fn new(x: i32, y:i32) -> Cursor {
        Cursor { x: x, y: y }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn up(&mut self) {
        self.y -= 1;
    }

    pub fn down(&mut self) {
        self.y += 1;
    }

    pub fn left(&mut self) {
        self.x -= 1;
    }

    pub fn right(&mut self) {
        self.x += 1;
    }
}
