use arraydeque::{ArrayDeque, Wrapping};
use std::time::Instant;

const SAMPLE_SIZE: usize = 64;

/// A `PerformanceCounter` tracks the time between `update()` calls.
/// It holds a fixed-size buffer of samples and calculates the .
#[derive(Default)]
pub struct PerformanceCounter {
    last_update: Option<Instant>,
    samples: ArrayDeque<f32, SAMPLE_SIZE, Wrapping>,
}

impl PerformanceCounter {
    /// Updates the performance counter with the delta since the update call.
    pub fn update(&mut self) {
        let last_update = self.last_update.unwrap_or_else(Instant::now);
        let _ = self.samples.push_back(last_update.elapsed().as_secs_f32());
        self.last_update = Some(Instant::now());
    }

    /// Returns the average rate of updates per second based on the samples collected.
    pub fn rate(&self) -> f32 {
        match self.samples.iter().sum::<f32>() {
            0.0 => 0.0,
            sum => self.samples.len() as f32 / sum,
        }
    }
}
