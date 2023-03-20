#![cfg_attr(feature = "portable_simd", feature(portable_simd))]

pub mod count;
pub mod lzss;
pub mod normalization;
pub mod r_ans;
pub mod spreads;
pub mod t_ans;

#[cfg(test)]
mod tests;
