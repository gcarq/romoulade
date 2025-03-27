use crate::gb::utils;

/// Represents all possible Joypad Inputs that the emulator can receive.
#[derive(Copy, Clone, PartialEq)]
pub enum JoypadInput {
    DPad(DPadInput),
    Action(ActionInput),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DPadInput {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ActionInput {
    A,
    B,
    Start,
    Select,
}

#[derive(Copy, Clone, PartialEq)]
enum SelectedButtons {
    DPad,
    Action,
    None,
}

/// Represents Joypad at register 0xFF00
/// The eight Game Boy action/direction buttons are arranged as a 2×4 matrix.
/// Select either action or direction buttons by writing to this register,
/// then read out the bits 0-3. The lower nibble is Read-only.
/// Note that, rather unconventionally for the Game Boy,
/// a button being pressed is seen as the corresponding bit being 0, not 1.
#[derive(Copy, Clone)]
pub struct Joypad {
    a_right: bool,              // bit 0, A or Right
    b_left: bool,               // bit 1, B or Left
    select_up: bool,            // bit 2, Select or Up
    start_down: bool,           // bit 3, Start or Down
    selection: SelectedButtons, // bit 4-5, D-Pad keys or Action keys respectively
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            a_right: false,
            b_left: false,
            select_up: false,
            start_down: false,
            selection: SelectedButtons::None,
        }
    }
}

impl Joypad {
    /// Reads the Joypad register and returns the current state of the buttons.
    pub fn read(&self) -> u8 {
        let mut value = 0b0000_0000;
        value = utils::set_bit(value, 0, !self.a_right);
        value = utils::set_bit(value, 1, !self.b_left);
        value = utils::set_bit(value, 2, !self.select_up);
        value = utils::set_bit(value, 3, !self.start_down);
        match self.selection {
            SelectedButtons::DPad => {
                value = utils::set_bit(value, 4, false);
                value = utils::set_bit(value, 5, true);
            }
            SelectedButtons::Action => {
                value = utils::set_bit(value, 4, true);
                value = utils::set_bit(value, 5, false)
            }
            SelectedButtons::None => {
                value = utils::set_bit(value, 4, true);
                value = utils::set_bit(value, 5, true);
            }
        };
        value
    }

    /// Writes the given value to the Joypad register and returns true if a button has been pressed
    /// and a Joypad interrupt should be requested.
    pub fn write(&mut self, value: u8, pending_event: Option<JoypadInput>) -> bool {
        if !utils::bit_at(value, 4) {
            self.selection = SelectedButtons::DPad;
        } else if !utils::bit_at(value, 5) {
            self.selection = SelectedButtons::Action;
        } else {
            self.selection = SelectedButtons::None;
            self.reset();
        }
        if let Some(event) = pending_event {
            self.handle(event)
        } else {
            false
        }
    }

    /// Handles the given Joypad event and returns true if a button has been pressed
    /// and a Joypad interrupt should be requested.
    fn handle(&mut self, event: JoypadInput) -> bool {
        self.reset();
        match event {
            JoypadInput::DPad(input) if self.selection == SelectedButtons::DPad => {
                match input {
                    DPadInput::Left => self.b_left = true,
                    DPadInput::Right => self.a_right = true,
                    DPadInput::Up => self.select_up = true,
                    DPadInput::Down => self.start_down = true,
                }
                println!("handling DPad: {:?}", input);
                true
            }
            JoypadInput::Action(input) if self.selection == SelectedButtons::Action => {
                match input {
                    ActionInput::A => self.a_right = true,
                    ActionInput::B => self.b_left = true,
                    ActionInput::Start => self.start_down = true,
                    ActionInput::Select => self.select_up = true,
                }
                println!("handling Action: {:?}", input);
                true
            }
            _ => false,
        }
    }

    /// Resets the joypad state of the lower nibble
    #[inline]
    fn reset(&mut self) {
        self.a_right = false;
        self.b_left = false;
        self.select_up = false;
        self.start_down = false;
    }
}
