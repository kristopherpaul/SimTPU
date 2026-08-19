use crate::config::PeConfig;

pub struct Pe {
    
}

impl Pe {
    pub fn new(config: PeConfig) -> Result<Self, PeError> {
        Ok(Self {})
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PeError {
    
}