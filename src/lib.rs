#![cfg_attr(not(test), no_std)]
#![feature(int_bits_const)]

mod buddy;
mod common;

pub use crate::buddy::*;
pub use crate::common::*;
