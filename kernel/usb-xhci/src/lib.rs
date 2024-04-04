#![no_std]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(box_patterns)]
#![feature(ptr_metadata)]
// #![feature(inherent_associated_types)]
#![feature(maybe_uninit_uninit_array)]

// extern crate alloc;

pub mod arraymap;

pub mod descriptor;
pub mod endpoint;
pub mod setup;

pub mod ring;

pub mod bus;
pub mod class;
pub mod device;

pub mod controller;