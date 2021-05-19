use std::ffi::c_void;

use jni::{JavaVM, JNIEnv};
use jni::objects::{JClass, JObject};
use jni::sys::{jint, JNI_ERR, JNI_VERSION_1_6};

use crate::config::ENGINE_NAME;
use crate::jni::utils::get_config;

mod logger;
mod window;
mod utils;

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn JNI_OnLoad(_vm: *mut JavaVM, _reserved: *mut c_void) -> jint {
    match pretty_env_logger::try_init() {
        Ok(_) => {
            log::trace!(target: ENGINE_NAME, "Loading library...");
            log::trace!(target: ENGINE_NAME, "Logger initialized");
            JNI_VERSION_1_6
        },
        Err(err) => {
            eprintln!("Logger initialization error: {}", err);
            JNI_ERR
        }
    }
}

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn JNI_OnUnload(_vm: *mut JavaVM, _reserved: *mut c_void) {
    log::trace!(target: ENGINE_NAME, "Unloading library...")
}

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn Java_com_tuguzT_native_Entry_initialize(env: JNIEnv, _class: JClass, config: JObject) {
    let config = get_config(env, config);
    match config {
        Ok(config) => log::info!(target: ENGINE_NAME, "{:#?}", config),
        Err(err) => log::error!(target: ENGINE_NAME, "Initialization error: {:?}", err)
    }
}