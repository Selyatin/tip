pub struct Word {
    pub value: String,
    pub x: u16,
    pub y: u16
}

impl Word {
    pub fn new(value: impl Into<String>, x: u16, y: u16) -> Self {
        Self {
            value: value.into(),
            x,
            y
        }
    }
}
