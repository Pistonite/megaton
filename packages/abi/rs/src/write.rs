// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Megaton contributors

use std::{borrow::BorrowMut, collections::HashMap, fs::File, path::Path, sync::{Mutex, Once}};
use rand::prelude::*;

struct FileFD {
    pub fd: u32,
}

struct TCPFD {
    pub fd: u32,
}

trait FDType {
    fn write(self: &Self, buf: &[u8], len: usize);
    fn open(self: &mut Self, name: *const u8, mode: i32, flags: i32);
}


impl FDType for FileFD {
    fn write(self: &Self, buf: &[u8], len: usize) {
        libnx::write(self.fd, buf, len);
    }
    
    fn open(self: &mut Self, name: *const u8, mode: i32, flags: i32) {
        let new_fd = libnx::fsdev_open(name, flags, mode, &mut new_filefd); 
        self.fd = new_fd;
    }
}

impl FDType for TCPFD {
    fn write(self: &Self, buf: &[u8], len: usize) {
        libnx::write(self.fd, buf, len);
    }
    
    fn open(self: &mut Self, name: *const u8, mode: i32, flags: i32) {
        todo!()
    }
}

type FDMap = HashMap<u32, Box<(dyn FDType + 'static)>>;
static mut FDs: Option<Mutex<FDMap>> = None;
static INIT: Once = Once::new();

// inspiration: https://doc.rust-lang.org/std/sync/struct.Once.html#examples-1
fn global_fds<'a>() -> &'a Mutex<FDMap> {
    INIT.call_once(|| {
        // Since this access is inside a call_once, before any other accesses, it is safe
        unsafe {
            *FDs.borrow_mut() = Some(Mutex::new(HashMap::new()));
        }
    });
    
    unsafe { FDs.as_ref().unwrap() }
}

fn sys_write(fd: u32, buf: &[u8], len: usize) {
    let fds = global_fds();
    fds.lock().and_then(|fdmap| {
        match fdmap.get(&fd) {
            Some(our_fd) => Ok(our_fd.write(buf, len)),
            None => todo!(),
        }
    });
}

fn sys_open(name: *const u8, flags: i32, mode: i32) -> u32 {
    let mut new_filefd = FileFD{fd: 0};
    new_filefd.open(name, mode, flags);
    let new_fd: u32 = new_filefd.fd.clone();
    let result = global_fds().get_mut().and_then(|fds| {
        fds.insert(rand::randrange(0.0..1e9), Box::new(new_filefd));
        Ok(())
    });

    new_fd // future calls to sys_write referencing new_fd will be handled by new_filefd
}


