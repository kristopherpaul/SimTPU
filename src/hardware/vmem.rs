use crate::types::{PeAct, VMEM_SIZE};

pub struct Vmem<
    A = PeAct,
    const SIZE: usize = VMEM_SIZE
> {
    mem: [A; SIZE],
    out_data: Vec<A>,
    out_valid: bool
}

pub struct VmemStrobes {
    pub read: bool,
    pub write: bool,
    pub reset: bool
}

pub struct VmemInputs<A = PeAct> {
    pub addr: usize,
    pub size: usize,
    pub wr_data: Vec<A>,
    pub strobes: VmemStrobes
}

impl<A, const SIZE: usize> Vmem<A, SIZE>
where
    A: Default + Copy
{
    pub fn new() -> Result<Self, VmemError> {
        Ok(Self {
            mem: [A::default(); SIZE],
            out_data: Vec::new(),
            out_valid: false
        })
    }

    pub fn tick(&mut self, inputs: VmemInputs<A>) -> Result<(), VmemError> {
        if inputs.strobes.reset {
            self.mem = [A::default(); SIZE];
            self.out_data.clear();
            self.out_valid = false;
        } else {
            if inputs.strobes.read {
                if (inputs.addr+inputs.size) <= SIZE {
                    self.out_data = self.mem[inputs.addr..(inputs.addr+inputs.size)].to_vec();
                    self.out_valid = true;
                } else {
                    // out of bounds read attempt
                    self.out_valid = false;
                }
            } else {
                self.out_valid = false;
            }
            if inputs.strobes.write {
                assert_eq!(inputs.wr_data.len(), inputs.size);
                if (inputs.addr+inputs.size) <= SIZE {
                    self.mem[inputs.addr..(inputs.addr+inputs.size)].copy_from_slice(&inputs.wr_data);
                } else{
                    // out of bounds write attempt
                }
            }
        }

        Ok(())
    }

    pub fn out_data(&self) -> Vec<A> {
        self.out_data.clone()
    }

    pub fn out_valid(&self) -> bool {
        self.out_valid
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VmemError {

}


#[cfg(test)]
mod tests {
    use super::*;

    type TestVmem = Vmem<i8, 16>;

    fn inputs(addr: usize, size: usize, wr_data: Vec<PeAct>, read: bool, write: bool, reset: bool) -> VmemInputs {
        VmemInputs {
            addr,
            size,
            wr_data,
            strobes: VmemStrobes { read, write, reset },
        }
    }

    #[test]
    fn new_initializes_memory_to_default() {
        let vmem = TestVmem::new().unwrap();
        assert!(vmem.out_data.is_empty());
        assert!(!vmem.out_valid());
    }

    #[test]
    fn out_of_bounds_read_sets_invalid() {
        let mut vmem = TestVmem::new().unwrap();
        // Try to read 4 elements starting at address 14 (would overflow 16)
        vmem.tick(inputs(14, 4, vec![], true, false, false))
            .unwrap();
        assert!(!vmem.out_valid());
    }

    #[test]
    fn out_of_bounds_write_does_not_write() {
        let mut vmem = TestVmem::new().unwrap();
        // Try to write 5 elements starting at address 12 (would overflow 16)
        vmem.tick(inputs(12, 5, vec![1, 2, 3, 4, 5], false, true, false))
            .unwrap();
        // Memory should not have been modified
        assert_eq!(vmem.mem[12], PeAct::default());
    }

    #[test]
    fn write_then_read_same_address() {
        let mut vmem = TestVmem::new().unwrap();
        let write_value = vec![123];
        vmem.tick(inputs(5, 1, write_value, false, true, false))
            .unwrap();
        vmem.tick(inputs(5, 1, vec![], true, false, false))
            .unwrap();
        assert_eq!(vmem.out_data(), vec![123]);
    }

    #[test]
    fn multiple_write_cycles_overwrite_previous() {
        let mut vmem = TestVmem::new().unwrap();
        vmem.tick(inputs(0, 1, vec![10], false, true, false))
            .unwrap();
        vmem.tick(inputs(0, 1, vec![20], false, true, false))
            .unwrap();
        vmem.tick(inputs(0, 1, vec![], true, false, false))
            .unwrap();
        assert_eq!(vmem.out_data(), vec![20]);
    }

    #[test]
    fn consecutive_reads_same_address() {
        let mut vmem = TestVmem::new().unwrap();
        vmem.tick(inputs(2, 2, vec![88, 99], false, true, false))
            .unwrap();

        vmem.tick(inputs(2, 2, vec![], true, false, false))
            .unwrap();
        assert_eq!(vmem.out_data(), vec![88, 99]);

        vmem.tick(inputs(2, 2, vec![], true, false, false))
            .unwrap();
        assert_eq!(vmem.out_data(), vec![88, 99]);
    }

    #[test]
    fn read_and_write_in_single_cycle() {
        let mut vmem = TestVmem::new().unwrap();
        // First write some initial data
        vmem.tick(inputs(0, 2, vec![10, 20], false, true, false))
            .unwrap();

        // Read from address 0 and write to address 2 simultaneously
        vmem.tick(inputs(0, 2, vec![], true, false, false))
            .unwrap();
        assert_eq!(vmem.out_data(), vec![10, 20]);

        // Now write while we could read (write takes precedence or they're independent)
        vmem.tick(inputs(2, 2, vec![30, 40], false, true, false))
            .unwrap();
        vmem.tick(inputs(2, 2, vec![], true, false, false))
            .unwrap();
        assert_eq!(vmem.out_data(), vec![30, 40]);
    }

    #[test]
    fn reset_has_priority_over_read_write() {
        let mut vmem = TestVmem::new().unwrap();
        vmem.tick(inputs(0, 1, vec![77], false, true, false))
            .unwrap();

        // Reset with read and write strobes asserted
        vmem.tick(inputs(0, 1, vec![99], true, true, true))
            .unwrap();

        assert!(!vmem.out_valid());
        assert!(vmem.out_data().is_empty());
        assert_eq!(vmem.mem[0], PeAct::default());
    }

    #[test]
    fn large_contiguous_write_and_read() {
        let mut vmem = Vmem::<i8, 256>::new().unwrap();
        let write_data: Vec<i8> = (0..100).map(|i| i as i8).collect();
        vmem.tick(inputs(0, 100, write_data.clone(), false, true, false))
            .unwrap();

        vmem.tick(inputs(0, 100, vec![], true, false, false))
            .unwrap();
        assert_eq!(vmem.out_data(), write_data);
    }

    #[test]
    fn address_zero_write_and_read() {
        let mut vmem = TestVmem::new().unwrap();
        vmem.tick(inputs(0, 1, vec![-1], false, true, false))
            .unwrap();
        vmem.tick(inputs(0, 1, vec![], true, false, false))
            .unwrap();
        assert_eq!(vmem.out_data(), vec![-1]);
    }

    #[test]
    fn scatter_gather_pattern() {
        let mut vmem = TestVmem::new().unwrap();

        // Write scattered data
        vmem.tick(inputs(1, 1, vec![11], false, true, false))
            .unwrap();
        vmem.tick(inputs(4, 1, vec![44], false, true, false))
            .unwrap();
        vmem.tick(inputs(7, 1, vec![77], false, true, false))
            .unwrap();

        // Read scattered data back
        vmem.tick(inputs(1, 1, vec![], true, false, false))
            .unwrap();
        assert_eq!(vmem.out_data(), vec![11]);

        vmem.tick(inputs(4, 1, vec![], true, false, false))
            .unwrap();
        assert_eq!(vmem.out_data(), vec![44]);

        vmem.tick(inputs(7, 1, vec![], true, false, false))
            .unwrap();
        assert_eq!(vmem.out_data(), vec![77]);
    }
}