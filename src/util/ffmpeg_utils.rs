use std::collections::HashMap;
use std::ffi::CString;
use ffmpeg_sys_next::{av_dict_set, AVDictionary};

pub(crate) fn hashmap_to_avdictionary(opts: &Option<HashMap<CString, CString>>) -> *mut AVDictionary {
    let mut av_dict: *mut AVDictionary = std::ptr::null_mut();

    if let Some(map) = opts {
        for (key, value) in map {
            unsafe {
                av_dict_set(&mut av_dict, key.as_ptr(), value.as_ptr(), 0);
            }
        }
    }

    av_dict
}