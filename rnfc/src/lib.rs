#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

// This must go FIRST so that other mods see its macros.
mod fmt;

pub use rnfc_traits as traits;

pub mod iso14443a;
pub mod iso_dep;
