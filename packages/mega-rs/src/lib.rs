mod hermit;

#[path = "megacpp_sys_gen.rs"]
#[allow(unused)]
#[allow(non_camel_case_types)]
mod megacpp_sys;

pub use mega_rspmacro::main;
