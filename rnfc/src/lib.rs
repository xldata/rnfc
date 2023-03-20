#![no_std]
#![feature(async_fn_in_trait, impl_trait_projections)]
#![allow(incomplete_features)]

// This must go FIRST so that other mods see its macros.
mod fmt;

pub use rnfc_traits as traits;

pub mod iso14443a;
pub mod iso_dep;
