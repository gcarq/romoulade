use crate::gb::bus::InterruptRegister;
use crate::gb::utils;

/// Represents all possible Joypad Inputs that the emulator can receive.
#[derive(Copy, Clone, Default, Debug)]
pub struct JoypadInput {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
}

impl JoypadInput {
    /// Returns true if any button is pressed.
    #[inline]
    pub const fn is_pressed(self) -> bool {
        self.left
            || self.right
            || self.up
            || self.down
            || self.a
            || self.b
            || self.start
            || self.select
    }

    /// Resets the state of the D-Pad buttons.
    #[inline]
    pub const fn reset_dpad(&mut self) {
        self.left = false;
        self.right = false;
        self.up = false;
        self.down = false;
    }

    /// Resets the state of the Action buttons.
    #[inline]
    pub const fn reset_action(&mut self) {
        self.a = false;
        self.b = false;
        self.start = false;
        self.select = false;
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
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
#[derive(Copy, Clone, Debug)]
pub struct Joypad {
    a_right: bool,              // bit 0, A or Right
    b_left: bool,               // bit 1, B or Left
    select_up: bool,            // bit 2, Select or Up
    start_down: bool,           // bit 3, Start or Down
    selection: SelectedButtons, // bit 4-5, D-Pad keys or Action keys respectively
    pending_event: JoypadInput, // pending input events that need handling
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            a_right: false,
            b_left: false,
            select_up: false,
            start_down: false,
            selection: SelectedButtons::None,
            pending_event: JoypadInput::default(),
        }
    }
}

impl Joypad {
    /// Handles the given `JoypadInput` and sets the corresponding button state on the next write.
    #[inline(always)]
    pub const fn handle_input(&mut self, event: JoypadInput) {
        self.pending_event = event;
    }

    /// Reads the Joypad register and returns the current state of the buttons.
    pub const fn read(&self) -> u8 {
        let mut value = 0b1100_0000;
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
                value = utils::set_bit(value, 5, false);
            }
            SelectedButtons::None => {
                value = utils::set_bit(value, 4, true);
                value = utils::set_bit(value, 5, true);
            }
        }
        value
    }

    /// Writes the given value to the Joypad register, handles the pending Joypad input and
    /// requests an interrupt if a button is pressed.
    pub fn write(&mut self, value: u8, int_reg: &mut InterruptRegister) {
        self.reset();

        // In the joypad register the bit values are inverted,
        // 0 means selected and 1 means not selected.
        match (utils::bit_at(value, 4), utils::bit_at(value, 5)) {
            (false, true) => {
                self.selection = SelectedButtons::DPad;
                self.b_left = self.pending_event.left;
                self.a_right = self.pending_event.right;
                self.start_down = self.pending_event.down;
                self.select_up = self.pending_event.up;
                self.pending_event.reset_dpad();
            }
            (true, false) => {
                self.selection = SelectedButtons::Action;
                self.a_right = self.pending_event.a;
                self.b_left = self.pending_event.b;
                self.start_down = self.pending_event.start;
                self.select_up = self.pending_event.select;
                self.pending_event.reset_action();
            }
            (true, true) => {
                self.selection = SelectedButtons::None;
                return;
            }
            (false, false) => return,
        }

        if self.a_right || self.b_left || self.start_down || self.select_up {
            int_reg.insert(InterruptRegister::JOYPAD);
        }
    }

    /// Resets the joypad state of the lower nibble
    #[inline]
    const fn reset(&mut self) {
        self.a_right = false;
        self.b_left = false;
        self.select_up = false;
        self.start_down = false;
    }
}
