use std::time::Instant;

#[derive(Debug, PartialEq, Eq)]
pub enum InputType {
    Press,
    LongPress,
    Move(Instant),
    Release,
}
impl InputType {
    fn is_move(&self) -> bool {
        match self {
            InputType::Move(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum PressType {
    ShortPress,
    ShortPressRelease,
    LongPress,
    LongPressRelease,
    Swipe,
    SwipeRelease,
}

pub struct InputHandler {
    prev_input_type: InputType,
}

impl InputHandler {
    pub fn new() -> InputHandler {
        InputHandler {
            prev_input_type: InputType::Release,
        }
    }
    pub fn input(&mut self, input: InputType) -> PressType {
        let output_type;
        if self.prev_input_type == InputType::Release && input == InputType::Press {
            output_type = PressType::ShortPress;
        } else if self.prev_input_type == InputType::Press && input == InputType::LongPress {
            output_type = PressType::LongPress;
        } else if self.prev_input_type == InputType::Press && input == InputType::Release {
            output_type = PressType::ShortPressRelease;
        } else if self.prev_input_type.is_move() && input == InputType::Release {
            output_type = PressType::SwipeRelease;
        } else if self.prev_input_type.is_move() && input == InputType::LongPress {
            output_type = PressType::LongPress;
        } else if self.prev_input_type == InputType::LongPress && input == InputType::Release {
            output_type = PressType::LongPressRelease;
        } else if input.is_move() {
            output_type = PressType::Swipe;
        } else {
            println!("Awkward! This should not happen!");
            output_type = PressType::ShortPressRelease;
        }
        self.prev_input_type = input;
        output_type
    }
}
