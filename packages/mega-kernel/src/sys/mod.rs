#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

#[rustfmt::skip]
mod binding;
mod result;

pub use binding::{
    nn_fs_DirectoryEntry,
    _megakernel_close_directory
};


