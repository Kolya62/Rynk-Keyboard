//! Where language model files (`lm/<code>.rlm`) are loaded from.
//!
//! On Android they are APK assets read through `AAssetManager`; the Kotlin side hands over the
//! `AssetManager` once via `nativeSetAssetManager`. Host builds (tests, tools) read them from
//! `RYNK_LM_DIR` or the app's assets directory in the source tree.

use crate::keyboard::state::Language;

pub fn load_model_bytes(lang: Language) -> Option<Vec<u8>> {
    platform::load(&format!("{}.rlm", lang.code()))
}

#[cfg(target_os = "android")]
mod platform {
    use jni::objects::{GlobalRef, JObject};
    use jni::JNIEnv;
    use std::ffi::{c_char, c_int, c_void, CString};
    use std::sync::OnceLock;

    #[repr(C)]
    struct AAssetManager {
        _private: [u8; 0],
    }
    #[repr(C)]
    struct AAsset {
        _private: [u8; 0],
    }

    const AASSET_MODE_STREAMING: c_int = 2;

    #[link(name = "android")]
    extern "C" {
        fn AAssetManager_fromJava(env: *mut jni::sys::JNIEnv, asset_manager: jni::sys::jobject) -> *mut AAssetManager;
        fn AAssetManager_open(mgr: *mut AAssetManager, filename: *const c_char, mode: c_int) -> *mut AAsset;
        fn AAsset_read(asset: *mut AAsset, buf: *mut c_void, count: usize) -> c_int;
        fn AAsset_getLength64(asset: *mut AAsset) -> i64;
        fn AAsset_close(asset: *mut AAsset);
    }

    struct Manager {
        // Keeps the Java AssetManager alive, which keeps the native pointer valid
        _java: GlobalRef,
        ptr: usize,
    }

    static MANAGER: OnceLock<Manager> = OnceLock::new();

    pub fn set_asset_manager(env: &mut JNIEnv, assets: &JObject) {
        if MANAGER.get().is_some() {
            return;
        }
        let Ok(global) = env.new_global_ref(assets) else { return };
        // SAFETY: `assets` is a live android.content.res.AssetManager passed from Kotlin
        let ptr = unsafe { AAssetManager_fromJava(env.get_raw(), global.as_obj().as_raw()) };
        if !ptr.is_null() {
            let _ = MANAGER.set(Manager { _java: global, ptr: ptr as usize });
        }
    }

    pub fn load(file: &str) -> Option<Vec<u8>> {
        let mgr = MANAGER.get()?.ptr as *mut AAssetManager;
        let path = CString::new(format!("lm/{file}")).ok()?;
        // SAFETY: the manager pointer stays valid while the GlobalRef is held; the asset is
        // closed on every path below
        unsafe {
            let asset = AAssetManager_open(mgr, path.as_ptr(), AASSET_MODE_STREAMING);
            if asset.is_null() {
                return None;
            }
            let len = AAsset_getLength64(asset).max(0) as usize;
            let mut buf = vec![0u8; len];
            let mut filled = 0;
            while filled < len {
                let n = AAsset_read(asset, buf[filled..].as_mut_ptr() as *mut c_void, len - filled);
                if n <= 0 {
                    break;
                }
                filled += n as usize;
            }
            AAsset_close(asset);
            (filled == len).then_some(buf)
        }
    }
}

#[cfg(not(target_os = "android"))]
mod platform {
    pub fn load(file: &str) -> Option<Vec<u8>> {
        let dir = std::env::var("RYNK_LM_DIR")
            .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../app/src/main/assets/lm").to_string());
        std::fs::read(std::path::Path::new(&dir).join(file)).ok()
    }
}

#[cfg(target_os = "android")]
pub use platform::set_asset_manager;
