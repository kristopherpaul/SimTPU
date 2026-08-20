use crate::types::{PeAct, PePsum};

pub struct Pe<W = PeAct, A = PeAct, P = PePsum> {
    weight: W,
    out_act: A,
    out_psum: P
}

impl<W, A, P> Pe<W, A, P>
where
    W: Default + Into<P>,
    A: Default + Into<P>,
    P: Default
{
    pub fn new() -> Result<Self, PeError> {
        Ok(Self {
            weight: W::default(),
            out_act: A::default(),
            out_psum: P::default()
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PeError {
    
}