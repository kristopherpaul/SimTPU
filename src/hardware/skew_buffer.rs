use crate::types::{PeAct, MMU_ROWS};
use std::collections::VecDeque;

pub struct SkewBuffer<
    A = PeAct,
    const SIZE: usize = MMU_ROWS
> {
    buffer: [VecDeque<A>; SIZE],
    out_data: [A; SIZE],
    out_valid: bool
}

pub struct SkewBufferStrobes {
    pub load: bool,
    pub reset: bool
}

pub struct SkewBufferInputs<
    A = PeAct,
    const SIZE: usize = MMU_ROWS
> {
    pub in_data: [A; SIZE],
    pub strobes: SkewBufferStrobes
}

impl<A, const SIZE: usize> SkewBuffer<A, SIZE>
where
    A: Default + Copy
{
    pub fn new() -> Result<Self, SkewBufferError> {
        Ok(Self {
            buffer: std::array::from_fn(|index| VecDeque::from(vec![A::default(); index])),
            out_data: [A::default(); SIZE],
            out_valid: false
        })
    }

    pub fn tick(&mut self, inputs: SkewBufferInputs<A>) -> Result<(), SkewBufferError> {
        if inputs.strobes.reset {
            self.buffer = std::array::from_fn(|index| VecDeque::from(vec![A::default(); index]));
            self.out_data = [A::default(); SIZE];
            self.out_valid = false;
        } else if inputs.strobes.load {
            self.buffer.iter_mut()
                       .enumerate()
                       .for_each(|(index, vec)| vec.push_back(inputs.in_data[index]));
            self.out_data.iter_mut()
                         .enumerate()
                         .for_each(|(index, slot)| *slot = self.buffer[index].pop_front().unwrap_or_default());
            self.out_valid = true;
        } else {
            self.out_valid = false;
        }
        
        Ok(())
    }

    pub fn out_data(&self) -> [A; SIZE] {
        self.out_data
    }

    pub fn out_valid(&self) -> bool {
        self.out_valid
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SkewBufferError {

}


#[cfg(test)]
mod tests {
    use super::*;

    type TestSkewBuffer = SkewBuffer<u8>;

    fn inputs(data: [u8; MMU_ROWS], load: bool, reset: bool) -> SkewBufferInputs<u8> {
        SkewBufferInputs {
            in_data: data,
            strobes: SkewBufferStrobes { load, reset },
        }
    }

    #[test]
    fn new_initializes_buffer_with_staggered_delays() {
        let buffer = TestSkewBuffer::new().unwrap();
        // Each buffer[i] should start with i default elements
        for i in 0..MMU_ROWS {
            assert_eq!(buffer.buffer[i].len(), i);
        }
        assert_eq!(buffer.out_data, [u8::default(); MMU_ROWS]);
        assert!(!buffer.out_valid);
    }

    #[test]
    fn load_strobes_data_through_buffer() {
        let mut buffer = TestSkewBuffer::new().unwrap();
        let in_data = [1, 2];
        buffer.tick(inputs(in_data, true, false)).unwrap();
        
        // After first load:
        // buffer[0] receives 1, pops immediately -> out_data[0] = 1
        // buffer[1] receives 2, pops (had 1 default) -> out_data[1] = default (0)
        assert_eq!(buffer.out_data()[0], 1);
        assert_eq!(buffer.out_data()[1], 0);
        assert!(buffer.out_valid());
    }

    #[test]
    fn staggered_output_appears_at_different_times() {
        let mut buffer = TestSkewBuffer::new().unwrap();
        let in_data = [10, 20];
        // Load cycle 1: Load initial values
        buffer.tick(inputs(in_data, true, false)).unwrap();
        let cycle1_out = buffer.out_data();
        // Load cycle 2: Load new values, and see first set come out staggered
        buffer.tick(inputs([30, 40], true, false)).unwrap();
        let cycle2_out = buffer.out_data();
        // Verify staggered output:
        // Cycle 1: First element of batch exits, rest are still in delay lines
        assert_eq!(cycle1_out[0], 10);
        // Cycle 2: Second element of first batch exits (was delayed by 1 cycle in buffer[1])
        assert_eq!(cycle2_out[1], 20);
    }

    #[test]
    fn valid_flag_set_on_load_and_cleared_on_idle() {
        let mut buffer = TestSkewBuffer::new().unwrap();

        buffer.tick(inputs([1, 2], true, false)).unwrap();
        assert!(buffer.out_valid());        
        buffer.tick(inputs([0, 0], false, false)).unwrap();
        assert!(!buffer.out_valid());
        buffer.tick(inputs([5, 6], true, false)).unwrap();
        assert!(buffer.out_valid());
    }

    #[test]
    fn reset_clears_buffer_state() {
        let mut buffer = TestSkewBuffer::new().unwrap();
        // Load some data
        buffer.tick(inputs([10, 20], true, false)).unwrap();
        assert!(buffer.out_valid());
        // Reset
        buffer.tick(inputs([0, 0], false, true)).unwrap();

        // After reset, buffer should be reinitialized
        assert!(!buffer.out_valid());
        assert_eq!(buffer.out_data(), [0; MMU_ROWS]);
        // Verify internal state is reset to initial staggered delays
        for i in 0..MMU_ROWS {
            assert_eq!(buffer.buffer[i].len(), i);
        }
    }

    #[test]
    fn idle_cycle_maintains_output_without_advancing() {
        let mut buffer = TestSkewBuffer::new().unwrap();
        buffer.tick(inputs([1, 2], true, false)).unwrap();
        let data_after_load = buffer.out_data();
        
        // Idle cycle (load and reset both false)
        buffer.tick(inputs([0, 0], false, false)).unwrap();
        let data_after_idle = buffer.out_data();    
        // Output should remain the same, but valid flag should be deasserted
        assert_eq!(data_after_load, data_after_idle);
        assert!(!buffer.out_valid());
    }

    #[test]
    fn consecutive_loads_create_pipeline_behavior() {
        let mut buffer = TestSkewBuffer::new().unwrap();
        // Simulate continuous stream of data
        let mut all_outputs = Vec::new();
        
        for cycle in 0..6 {
            let input_data = [(cycle * 2 + 1) as u8, (cycle * 2 + 2) as u8];
            buffer.tick(inputs(input_data, true, false)).unwrap();
            all_outputs.push(buffer.out_data());
        }
        // Verify that data flows through the staggered delays
        // First element appears immediately in each cycle
        for (cycle, output) in all_outputs.iter().enumerate() {
            let expected = (cycle * 2 + 1) as u8;
            assert_eq!(output[0], expected, "First element should appear immediately");
        }
        // Second element should lag by one cycle
        for cycle in 1..all_outputs.len() {
            let expected = ((cycle - 1) * 2 + 2) as u8;
            assert_eq!(all_outputs[cycle][1], expected, "Second element should lag by 1 cycle");
        }
    }

    #[test]
    fn output_contains_all_elements_across_staggered_cycles() {
        let mut buffer = TestSkewBuffer::new().unwrap();
        let input_batch = [100, 101];
        // Load the batch
        buffer.tick(inputs(input_batch, true, false)).unwrap();
        // Collect outputs over following cycles
        let mut collected_outputs = vec![buffer.out_data()];
        
        for _ in 1..MMU_ROWS {
            buffer.tick(inputs([0, 0], true, false)).unwrap();
            collected_outputs.push(buffer.out_data());
        }
        
        // Verify all input elements appear in the outputs across the staggered cycles
        let mut found = [false; 2];
        for output in collected_outputs {
            for (i, val) in output.iter().enumerate() {
                if i < 2 && *val == input_batch[i] {
                    found[i] = true;
                }
            }
        }
        
        for i in 0..2 {
            assert!(found[i], "Element {} should appear in output somewhere", i);
        }
    }
}