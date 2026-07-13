use crate::gb::bus::InterruptRegister;

#[cfg(test)]
mod tests;

bitflags! {
    /// Raw state of the JOYP register at 0xFF00, see https://gbdev.io/pandocs/Joypad_Input.html.
    ///
    /// Only bits 0-5 are used, also in the joypad register the bit values are inverted:
    /// 0 means selected and 1 means not selected.
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    struct JoypadRegister: u8 {
        const A_RIGHT = 0b0000_0001;
        const B_LEFT = 0b0000_0010;
        const SELECT_UP = 0b0000_0100;
        const START_DOWN = 0b0000_1000;
        const SELECT_DPAD = 0b0001_0000;
        const SELECT_BUTTONS = 0b0010_0000;
        const UNUSED = 0b1100_0000;

        const INPUT_LINES = Self::A_RIGHT.bits()
            | Self::B_LEFT.bits()
            | Self::SELECT_UP.bits()
            | Self::START_DOWN.bits();
    }
}

impl Default for JoypadRegister {
    fn default() -> Self {
        Self::UNUSED | Self::INPUT_LINES
    }
}

impl JoypadRegister {
    /// Updates register from the selected rows and the given `input`.
    pub fn update_from_input_event(&mut self, input: JoypadInputState) {
        let dpad_selected = self.is_dpad_selected();
        let buttons_selected = self.are_buttons_selected();
        self.set(
            JoypadRegister::A_RIGHT,
            !(buttons_selected && input.a || dpad_selected && input.right),
        );
        self.set(
            JoypadRegister::B_LEFT,
            !(buttons_selected && input.b || dpad_selected && input.left),
        );
        self.set(
            JoypadRegister::SELECT_UP,
            !(buttons_selected && input.select || dpad_selected && input.up),
        );
        self.set(
            JoypadRegister::START_DOWN,
            !(buttons_selected && input.start || dpad_selected && input.down),
        );
    }

    /// Returns true if the DPad is currently selected
    const fn is_dpad_selected(self) -> bool {
        !self.contains(Self::SELECT_DPAD)
    }

    /// Returns true if the action buttons are currently selected
    const fn are_buttons_selected(self) -> bool {
        !self.contains(Self::SELECT_BUTTONS)
    }
}

/// Represents all possible input states that the emulator can receive.
/// This will be sent from the frontend and is used to update `Joypad`.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct JoypadInputState {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
}

/// Represents Joypad at register 0xFF00
/// The eight Game Boy action/direction buttons are arranged as a 2×4 matrix.
/// Select either action or direction buttons by writing to this register,
/// then read out the bits 0-3. The lower nibble is Read-only.
/// Note that, rather unconventionally for the Game Boy,
/// a button being pressed is seen as the corresponding bit being 0, not 1.
#[derive(Clone, Debug, Default)]
pub struct Joypad {
    input: JoypadInputState,
    register: JoypadRegister,
}

impl Joypad {
    /// Uses the given `input` to update the `Joypad` including its register.
    pub fn handle_input(&mut self, input: JoypadInputState, int_reg: &mut InterruptRegister) {
        let previous = self.register;
        self.register.update_from_input_event(input);
        self.input = input;
        Self::falling_edge_irq(previous, self.register, int_reg);
    }

    /// Reads the Joypad register and returns the current state of the selected buttons.
    pub fn read(&self) -> u8 {
        self.register.bits()
    }

    /// Writes the JOYP row-selection bits and triggers an IRQ if the change causes a
    /// high to low transition.
    pub fn write(&mut self, value: u8, int_reg: &mut InterruptRegister) {
        let previous = self.register;
        let value = JoypadRegister::from_bits_retain(value);

        self.register.set(
            JoypadRegister::SELECT_DPAD,
            value.contains(JoypadRegister::SELECT_DPAD),
        );
        self.register.set(
            JoypadRegister::SELECT_BUTTONS,
            value.contains(JoypadRegister::SELECT_BUTTONS),
        );
        self.register.update_from_input_event(self.input);

        Self::falling_edge_irq(previous, self.register, int_reg);
    }

    /// Triggers an IRQ when any of the input lines change from high to low.
    fn falling_edge_irq(
        previous: JoypadRegister,
        current: JoypadRegister,
        int_reg: &mut InterruptRegister,
    ) {
        if !(previous & !current & JoypadRegister::INPUT_LINES).is_empty() {
            int_reg.insert(InterruptRegister::JOYPAD);
        }
    }
}
