use crate::gb::AddressSpace;
use crate::gb::constants::{AUDIO_REGISTERS_SIZE, AUDIO_REGISTERS_START};

/// This register controls CH1’s period sweep functionality
pub const AUDIO_SOUND_CHANNEL_1_SWEEP: u16 = 0xFF10;

/// This register controls both the channel’s length timer and duty cycle
/// (the ratio of the time spent low vs. high). The selected duty cycle also alters the phase,
/// although the effect is hardly noticeable except in combination with other channels
pub const AUDIO_CHANNEL_1_LENGTH: u16 = 0xFF11;

/// This register controls the digital amplitude of the “high” part of the pulse,
/// and the sweep applied to that setting
pub const AUDIO_CHANNEL_1_VOLUME: u16 = 0xFF12;

/// This register stores the low 8 bits of the channel’s 11-bit “period value”.
/// The upper 3 bits are stored in the low 3 bits of NR14.
pub const AUDIO_CHANNEL_1_PERIOD_LOW: u16 = 0xFF13;

pub const AUDIO_CHANNEL_1_PERIOD_HIGH: u16 = 0xFF14;

/// This sound channel works exactly like channel 1,
/// except that it lacks a period sweep (and thus an equivalent to NR10).
/// Please refer to the corresponding CH1 register:
pub const AUDIO_CHANNEL_2_LENGTH: u16 = 0xFF16;
pub const AUDIO_CHANNEL_2_VOLUME: u16 = 0xFF17;
pub const AUDIO_CHANNEL_2_PERIOD_LOW: u16 = 0xFF18;
pub const AUDIO_CHANNEL_2_PERIOD_HIGH: u16 = 0xFF19;

/// This register controls CH3’s DAC. Like other channels,
/// turning the DAC off immediately turns the channel off as well.
pub const AUDIO_CHANNEL_3_DAC_ENABLE: u16 = 0xFF1A;

/// This register controls the channel’s length timer.
pub const AUDIO_CHANNEL_3_LENGTH: u16 = 0xFF1B;

/// This channel lacks the envelope functionality that the other three channels have,
/// and has a much coarser volume control.
pub const AUDIO_CHANNEL_3_VOLUME: u16 = 0xFF1C;

/// This register stores the low 8 bits of the channel’s 11-bit “period value”.
/// The upper 3 bits are stored in the low 3 bits of NR34.
pub const AUDIO_CHANNEL_3_PERIOD_LOW: u16 = 0xFF1D;

pub const AUDIO_CHANNEL_3_PERIOD_HIGH: u16 = 0xFF1E;

/// This register controls the channel’s length timer.
pub const AUDIO_CHANNEL_4_LENGTH: u16 = 0xFF20;

/// This register functions exactly like NR12, so please refer to its documentation.
pub const AUDIO_CHANNEL_4_VOLUME: u16 = 0xFF21;

/// This register allows controlling the way the amplitude is randomly switched.
pub const AUDIO_CHANNEL_4_FREQ: u16 = 0xFF22;

pub const AUDIO_CHANNEL_4_CONTROL: u16 = 0xFF23;

pub const AUDIO_MASTER_VOLUME: u16 = 0xFF24;

pub const AUDIO_SOUND_PANNING: u16 = 0xFF25;

pub const AUDIO_MASTER_CONTROL: u16 = 0xFF26;

/// Wave RAM is 16 bytes long; each byte holds two “samples”, each 4 bits.
pub const AUDIO_WAVE_PATTERN_START: u16 = 0xFF30;
pub const AUDIO_WAVE_PATTERN_END: u16 = 0xFF3F;

pub struct AudioProcessor {
    r: [u8; AUDIO_REGISTERS_SIZE],
}

impl Default for AudioProcessor {
    fn default() -> Self {
        AudioProcessor {
            r: [0; AUDIO_REGISTERS_SIZE],
        }
    }
}

impl AddressSpace for AudioProcessor {
    fn write(&mut self, address: u16, value: u8) {
        let offset = (address - AUDIO_REGISTERS_START) as usize;
        match address {
            AUDIO_SOUND_CHANNEL_1_SWEEP => self.r[offset] = value,
            AUDIO_CHANNEL_1_LENGTH => self.r[offset] = value,
            AUDIO_CHANNEL_1_VOLUME => self.r[offset] = value,
            AUDIO_CHANNEL_1_PERIOD_LOW => self.r[offset] = value,
            AUDIO_CHANNEL_1_PERIOD_HIGH => self.r[offset] = value,
            0xFF15 => {} // undocumented
            AUDIO_CHANNEL_2_LENGTH => self.r[offset] = value,
            AUDIO_CHANNEL_2_VOLUME => self.r[offset] = value,
            AUDIO_CHANNEL_2_PERIOD_LOW => self.r[offset] = value,
            AUDIO_CHANNEL_2_PERIOD_HIGH => self.r[offset] = value,
            AUDIO_CHANNEL_3_DAC_ENABLE => self.r[offset] = value,
            AUDIO_CHANNEL_3_LENGTH => self.r[offset] = value,
            AUDIO_CHANNEL_3_VOLUME => self.r[offset] = value,
            AUDIO_CHANNEL_3_PERIOD_LOW => self.r[offset] = value,
            AUDIO_CHANNEL_3_PERIOD_HIGH => self.r[offset] = value,
            0xFF1F => {} // undocumented
            AUDIO_CHANNEL_4_LENGTH => self.r[offset] = value,
            AUDIO_CHANNEL_4_VOLUME => self.r[offset] = value,
            AUDIO_CHANNEL_4_FREQ => self.r[offset] = value,
            AUDIO_CHANNEL_4_CONTROL => self.r[offset] = value,
            AUDIO_MASTER_VOLUME => self.r[offset] = value,
            AUDIO_SOUND_PANNING => self.r[offset] = value,
            AUDIO_MASTER_CONTROL => self.r[offset] = value,
            0xFF27..=0xFF2F => {} // undocumented
            AUDIO_WAVE_PATTERN_START..=AUDIO_WAVE_PATTERN_END => self.r[offset] = value,
            _ => panic!(
                "Attempt to write to unmapped audio register: 0x{:X}",
                address
            ),
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        let offset = (address - AUDIO_REGISTERS_START) as usize;
        match address {
            AUDIO_SOUND_CHANNEL_1_SWEEP => self.r[offset] | 0b1000_0000, // Undocumented bits should be 1
            AUDIO_CHANNEL_1_LENGTH => self.r[offset],
            AUDIO_CHANNEL_1_VOLUME => self.r[offset],
            AUDIO_CHANNEL_1_PERIOD_LOW => self.r[offset],
            AUDIO_CHANNEL_1_PERIOD_HIGH => self.r[offset],
            0xFF15 => 0xFF, // undocumented
            AUDIO_CHANNEL_2_LENGTH => self.r[offset],
            AUDIO_CHANNEL_2_VOLUME => self.r[offset],
            AUDIO_CHANNEL_2_PERIOD_LOW => self.r[offset],
            AUDIO_CHANNEL_2_PERIOD_HIGH => self.r[offset],
            AUDIO_CHANNEL_3_DAC_ENABLE => self.r[offset] | 0b0111_1111, // Undocumented bits should be 1
            AUDIO_CHANNEL_3_LENGTH => self.r[offset],
            AUDIO_CHANNEL_3_VOLUME => self.r[offset] | 0b1001_1111, // Undocumented bits should be 1
            AUDIO_CHANNEL_3_PERIOD_LOW => self.r[offset],
            AUDIO_CHANNEL_3_PERIOD_HIGH => self.r[offset],
            0xFF1F => 0xFF,                                         // undocumented
            AUDIO_CHANNEL_4_LENGTH => self.r[offset] | 0b1100_0000, // Undocumented bits should be 1
            AUDIO_CHANNEL_4_VOLUME => self.r[offset],
            AUDIO_CHANNEL_4_FREQ => self.r[offset],
            AUDIO_CHANNEL_4_CONTROL => self.r[offset] | 0b0011_1111, // Undocumented bits should be 1
            AUDIO_MASTER_VOLUME => self.r[offset],
            AUDIO_SOUND_PANNING => self.r[offset],
            AUDIO_MASTER_CONTROL => self.r[offset] | 0b0111_1111, // Undocumented bits should be 1
            0xFF27..=0xFF2F => 0xFF,                              // undocumented
            AUDIO_WAVE_PATTERN_START..=AUDIO_WAVE_PATTERN_END => self.r[offset],
            _ => panic!(
                "Attempt to read from unmapped audio register: 0x{:X}",
                address
            ),
        }
    }
}
