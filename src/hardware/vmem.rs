use crate::types::{PeAct, VMEM_SIZE};

pub struct Vmem<const SIZE: usize = VMEM_SIZE> {
    mem: [PeAct; SIZE],
    out_data: Vec<PeAct>,
    out_valid: bool
}

pub struct VmemStrobes {
    pub read: bool,
    pub write: bool,
    pub reset: bool
}

pub struct VmemInputs {
    pub addr: usize,
    pub size: usize,
    pub wr_data: Vec<PeAct>,
    pub strobes: VmemStrobes
}

impl<const SIZE: usize> Vmem<SIZE>
where
    PeAct: Default + Copy
{
    pub fn new() -> Result<Self, VmemError> {
        Ok(Self {
            mem: [PeAct::default(); SIZE],
            out_data: Vec::new(),
            out_valid: false
        })
    }

    pub fn tick(&mut self, inputs: VmemInputs) -> Result<(), VmemError> {
        if inputs.strobes.reset {
            self.mem = [PeAct::default(); SIZE];
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

    pub fn out_data(&self) -> Vec<PeAct> {
        self.out_data.clone()
    }

    pub fn out_valid(&self) -> bool {
        self.out_valid
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VmemError {

}