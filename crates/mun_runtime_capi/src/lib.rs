mod error;
mod hub;

use std::ffi::{c_void, CStr};
use std::os::raw::c_char;

use crate::error::ErrorHandle;
use crate::hub::HUB;
use failure::err_msg;
use mun_abi::FunctionInfo;
use mun_runtime::{MunRuntime, RuntimeBuilder};

pub(crate) type Token = usize;

pub trait TypedHandle {
    fn new(token: Token) -> Self;
    fn token(&self) -> Token;
}

#[repr(C)]
pub struct RuntimeHandle(*mut c_void);

#[no_mangle]
pub unsafe extern "C" fn mun_runtime_create(
    library_path: *const c_char,
    handle: *mut RuntimeHandle,
) -> ErrorHandle {
    if library_path.is_null() {
        return HUB.errors.register(Box::new(err_msg(
            "Invalid argument: 'library_path' is null pointer.",
        )));
    }

    let library_path = match CStr::from_ptr(library_path).to_str() {
        Ok(path) => path,
        Err(_) => {
            return HUB.errors.register(Box::new(err_msg(
                "Invalid argument: 'library_path' is not UTF-8 encoded.",
            )))
        }
    };

    let handle = match handle.as_mut() {
        Some(handle) => handle,
        None => {
            return HUB.errors.register(Box::new(err_msg(
                "Invalid argument: 'handle' is null pointer.",
            )))
        }
    };

    let runtime = match RuntimeBuilder::new(library_path).spawn() {
        Ok(runtime) => runtime,
        Err(e) => return HUB.errors.register(Box::new(e)),
    };

    handle.0 = Box::into_raw(Box::new(runtime)) as *mut _;
    ErrorHandle::default()
}

#[no_mangle]
pub extern "C" fn mun_runtime_destroy(handle: RuntimeHandle) {
    if !handle.0.is_null() {
        let _runtime = unsafe { Box::from_raw(handle.0) };
    }
}

#[no_mangle]
pub unsafe extern "C" fn mun_runtime_get_function_info(
    handle: RuntimeHandle,
    fn_name: *const c_char,
    has_fn_info: *mut bool,
    fn_info: *mut FunctionInfo,
) -> ErrorHandle {
    let runtime = match (handle.0 as *mut MunRuntime).as_ref() {
        Some(runtime) => runtime,
        None => {
            return HUB.errors.register(Box::new(err_msg(
                "Invalid argument: 'runtime' is null pointer.",
            )))
        }
    };

    let fn_name = match CStr::from_ptr(fn_name).to_str() {
        Ok(name) => name,
        Err(_) => {
            return HUB.errors.register(Box::new(err_msg(
                "Invalid argument: 'fn_name' is not UTF-8 encoded.",
            )))
        }
    };

    let has_fn_info = match has_fn_info.as_mut() {
        Some(has_info) => has_info,
        None => {
            return HUB.errors.register(Box::new(err_msg(
                "Invalid argument: 'has_fn_info' is null pointer.",
            )))
        }
    };

    let fn_info = match fn_info.as_mut() {
        Some(info) => info,
        None => {
            return HUB.errors.register(Box::new(err_msg(
                "Invalid argument: 'fn_info' is null pointer.",
            )))
        }
    };

    match runtime.get_function_info(fn_name) {
        Some(info) => {
            *has_fn_info = true;
            *fn_info = info.clone();
        }
        None => *has_fn_info = false,
    }

    ErrorHandle::default()
}

#[no_mangle]
pub unsafe extern "C" fn mun_runtime_update(
    handle: RuntimeHandle,
    updated: *mut bool,
) -> ErrorHandle {
    let runtime = match (handle.0 as *mut MunRuntime).as_mut() {
        Some(runtime) => runtime,
        None => {
            return HUB.errors.register(Box::new(err_msg(
                "Invalid argument: 'runtime' is null pointer.",
            )))
        }
    };

    let updated = match updated.as_mut() {
        Some(updated) => updated,
        None => {
            return HUB.errors.register(Box::new(err_msg(
                "Invalid argument: 'updated' is null pointer.",
            )))
        }
    };

    *updated = runtime.update();
    ErrorHandle::default()
}
