use crate::types::{PePsum, MMU_COLS};
use std::collections::VecDeque;

pub struct Acc<
    P = PePsum,
    const SIZE: usize = MMU_COLS
> {
    buffer: [VecDeque<P>; SIZE],
    out_data: [P; SIZE],
    out_valid: bool
}

pub struct AccStrobes {
    pub load: bool,
    pub reset: bool
}

pub struct AccInputs<
    P = PePsum,
    const SIZE: usize = MMU_COLS
> {
    pub in_data: [P; SIZE],
    pub strobes: AccStrobes
}

impl<P, const SIZE: usize> Acc<P, SIZE>
where
    P: Default + Copy
{
    pub fn new() -> Result<Self, AccError> {
        Ok(Self {
            buffer: std::array::from_fn(|index| VecDeque::from(vec![P::default(); SIZE-index-1])),
            out_data: [P::default(); SIZE],
            out_valid: false
        })
    }

    pub fn tick(&mut self, inputs: AccInputs<P, SIZE>) -> Result<(), AccError> {
        if inputs.strobes.reset {
            self.buffer = std::array::from_fn(|index| VecDeque::from(vec![P::default(); SIZE-index-1]));
            self.out_data = [P::default(); SIZE];
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

    pub fn out_data(&self) -> [P; SIZE] {
        self.out_data
    }

    pub fn out_valid(&self) -> bool {
        self.out_valid
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AccError {

}


#[cfg(test)]
mod tests {
    use super::*;

    type TestAcc = Acc<u8, 2>;
    const TEST_SIZE: usize = 2;

    fn inputs(data: [u8; TEST_SIZE], load: bool, reset: bool) -> AccInputs<u8, TEST_SIZE> {
        AccInputs {
            in_data: data,
            strobes: AccStrobes { load, reset },
        }
    }

    #[test]
    fn new_initializes_buffer_with_inverse_staggered_delays() {
        let buffer = TestAcc::new().unwrap();
        // Each buffer[i] should start with (SIZE-i-1) default elements
        // This is the inverse of SkewBuffer, creating deskew behavior
        for i in 0..TEST_SIZE {
            assert_eq!(buffer.buffer[i].len(), TEST_SIZE - i - 1);
        }
        assert_eq!(buffer.out_data, [u8::default(); TEST_SIZE]);
        assert!(!buffer.out_valid);
    }

    #[test]
    fn load_strobes_data_through_buffer() {
        let mut buffer = TestAcc::new().unwrap();
        let in_data = [1, 2];
        buffer.tick(inputs(in_data, true, false)).unwrap();
        // After first load with deskew delays:
        // buffer[0] has 1 default initially, receives 1, pops default -> out_data[0] = 0
        // buffer[1] has 0 defaults initially, receives 2, pops immediately -> out_data[1] = 2
        assert_eq!(buffer.out_data()[0], 0);
        assert_eq!(buffer.out_data()[1], 2);
        assert!(buffer.out_valid());
    }

    #[test]
    fn deskew_inverts_skew_behavior() {
        let mut buffer = TestAcc::new().unwrap();
        // Feed data that would come from a skew buffer in staggered form:
        // First element delayed, second element immediate
        let staggered_data_1 = [10, 20];
        buffer.tick(inputs(staggered_data_1, true, false)).unwrap();
        let cycle1_out = buffer.out_data();
        // Second load: first element from previous cycle appears now, new second element arrives
        let staggered_data_2 = [30, 40];
        buffer.tick(inputs(staggered_data_2, true, false)).unwrap();
        let cycle2_out = buffer.out_data();
        
        // Verify deskew alignment:
        // cycle1: First element delayed (default exits), second element immediate (20)
        // cycle2: First element from previous cycle (10 exits from delay), second element immediate (40)
        assert_eq!(cycle1_out[0], 0);  // default from initial delay
        assert_eq!(cycle1_out[1], 20); // immediate
        assert_eq!(cycle2_out[0], 10); // first element from cycle 1 exits delay
        assert_eq!(cycle2_out[1], 40); // immediate
    }

    #[test]
    fn valid_flag_set_on_load_and_cleared_on_idle() {
        let mut buffer = TestAcc::new().unwrap();
        buffer.tick(inputs([1, 2], true, false)).unwrap();
        assert!(buffer.out_valid());        
        buffer.tick(inputs([0, 0], false, false)).unwrap();
        assert!(!buffer.out_valid());
        buffer.tick(inputs([5, 6], true, false)).unwrap();
        assert!(buffer.out_valid());
    }

    #[test]
    fn reset_clears_buffer_state() {
        let mut buffer = TestAcc::new().unwrap();
        // Load some data
        buffer.tick(inputs([10, 20], true, false)).unwrap();
        assert!(buffer.out_valid());
        // Reset
        buffer.tick(inputs([0, 0], false, true)).unwrap();

        // After reset, buffer should be reinitialized
        assert!(!buffer.out_valid());
        assert_eq!(buffer.out_data(), [0; TEST_SIZE]);
        // Verify internal state is reset to initial deskew delays
        for i in 0..TEST_SIZE {
            assert_eq!(buffer.buffer[i].len(), TEST_SIZE - i - 1);
        }
    }

    #[test]
    fn staggered_input_becomes_aligned_output() {
        let mut buffer = TestAcc::new().unwrap();
        
        // Simulate a batch [100, 101] coming through staggered:
        // element[0] is delayed by 1 cycle, element[1] is immediate
        // Cycle 1: input elem[0]=100, elem[1]=101
        buffer.tick(inputs([100, 101], true, false)).unwrap();
        let cycle1_out = buffer.out_data();
        // Cycle 2: input elem[0]=200, elem[1]=201
        buffer.tick(inputs([200, 201], true, false)).unwrap();
        let cycle2_out = buffer.out_data();
        
        // Verify alignment:
        // Cycle 1: elem[0] exits from delay line (default/0), elem[1] is immediate (101)
        assert_eq!(cycle1_out[0], 0);   // default exits from delay
        assert_eq!(cycle1_out[1], 101); // immediate
        // Cycle 2: elem[0] from cycle 1 exits delay (100), elem[1] immediate (201)
        assert_eq!(cycle2_out[0], 100); // from delay line
        assert_eq!(cycle2_out[1], 201); // immediate
    }

    #[test]
    fn multiple_batches_maintain_deskew_alignment() {
        let mut buffer = TestAcc::new().unwrap();
        
        // First batch: [10, 11]
        buffer.tick(inputs([10, 11], true, false)).unwrap();
        let batch1_cycle1 = buffer.out_data();
        buffer.tick(inputs([20, 21], true, false)).unwrap();
        let batch1_cycle2 = buffer.out_data();
        // Second batch continues: [30, 31]
        buffer.tick(inputs([30, 31], true, false)).unwrap();
        let batch2_cycle1 = buffer.out_data();
        buffer.tick(inputs([40, 41], true, false)).unwrap();
        let batch2_cycle2 = buffer.out_data();
        
        // Verify alignment for first batch
        assert_eq!(batch1_cycle1[0], 0);  // default from delay
        assert_eq!(batch1_cycle1[1], 11); // immediate
        assert_eq!(batch1_cycle2[0], 10); // from delay
        assert_eq!(batch1_cycle2[1], 21); // immediate
        // Verify alignment for second batch
        assert_eq!(batch2_cycle1[0], 20); // from delay
        assert_eq!(batch2_cycle1[1], 31); // immediate
        assert_eq!(batch2_cycle2[0], 30); // from delay
        assert_eq!(batch2_cycle2[1], 41); // immediate
    }
}