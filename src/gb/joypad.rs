use crate::gb::bus::InterruptRegister;
use crate::gb::utils;

/// Represents all possible input events that the emulator can receive.
/// This will be sent from the frontend and is used to update `Joypad`.
#[derive(Copy, Clone, Default, Debug)]
pub struct JoypadInputEvent {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
}

impl JoypadInputEvent {
    /// Returns true if any button is pressed.
    pub fn is_pressed(self) -> bool {
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
    pub fn reset_dpad(&mut self) {
        self.left = false;
        self.right = false;
        self.up = false;
        self.down = false;
    }

    /// Resets the state of the Action buttons.
    #[inline]
    pub fn reset_action(&mut self) {
        self.a = false;
        self.b = false;
        self.start = false;
        self.select = false;
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
enum SelectedButtons {
    #[default]
    Initial,
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
#[derive(Copy, Clone, Debug, Default)]
pub struct Joypad {
    a_right: bool,                   // bit 0, A or Right
    b_left: bool,                    // bit 1, B or Left
    select_up: bool,                 // bit 2, Select or Up
    start_down: bool,                // bit 3, Start or Down
    selection: SelectedButtons,      // bit 4-5, D-Pad keys or Action keys respectively
    pending_event: JoypadInputEvent, // pending input events that need handling
}

impl Joypad {
    /// Handles the given `JoypadInput` and sets the corresponding button state on the next write.
    #[inline(always)]
    pub fn handle_input(&mut self, event: JoypadInputEvent) {
        self.pending_event = event;
    }

    /// Reads the Joypad register and returns the current state of the buttons.
    pub fn read(&self) -> u8 {
        let mut value = 0b1100_0000; // Undocumented bits should be 1
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
            SelectedButtons::Initial => {
                value = utils::set_bit(value, 4, false);
                value = utils::set_bit(value, 5, false);
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
            // D-Pad selection
            (false, true) => {
                self.selection = SelectedButtons::DPad;
                self.b_left = self.pending_event.left;
                self.a_right = self.pending_event.right;
                self.start_down = self.pending_event.down;
                self.select_up = self.pending_event.up;
                self.pending_event.reset_dpad();
            }
            // Action selection
            (true, false) => {
                self.selection = SelectedButtons::Action;
                self.a_right = self.pending_event.a;
                self.b_left = self.pending_event.b;
                self.start_down = self.pending_event.start;
                self.select_up = self.pending_event.select;
                self.pending_event.reset_action();
            }
            // No selection
            (true, true) => {
                self.selection = SelectedButtons::None;
                return;
            }
            // Initial state
            (false, false) => return,
        }

        if self.a_right || self.b_left || self.start_down || self.select_up {
            int_reg.insert(InterruptRegister::JOYPAD);
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
