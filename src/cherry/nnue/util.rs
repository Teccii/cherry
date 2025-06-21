use std::ops::{Deref, DerefMut};
use super::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C, align(64))]
pub struct Align64<T>(pub T);

impl<T> Deref for Align64<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Align64<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}