use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic;

/// Convert a nullable `*const c_char` to a `&str`.
/// Returns `""` for null pointers or invalid UTF-8.
unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> &'a str {
    if ptr.is_null() {
        return "";
    }
    match CStr::from_ptr(ptr).to_str() {
        Ok(s) => s,
        Err(_) => "",
    }
}

fn error_envelope(code: &str, message: &str) -> *const c_char {
    let json = format!(
        r#"{{"ok":false,"error":{{"code":"{}","message":"{}"}}}}"#,
        code, message
    );
    match CString::new(json) {
        Ok(cs) => cs.into_raw() as *const c_char,
        Err(_) => std::ptr::null(),
    }
}

/// Parse device output using a TextFSM template. Returns a JSON envelope as a
/// null-terminated C string. The caller **must** pass the returned pointer to
/// `textfsm_free` when done.
///
/// # Safety
/// All pointer arguments must be valid, null-terminated C strings (or null).
#[no_mangle]
pub unsafe extern "C" fn textfsm_parse_json(
    vendor: *const c_char,
    command_key: *const c_char,
    template_text: *const c_char,
    output_text: *const c_char,
) -> *const c_char {
    let result = panic::catch_unwind(|| {
        let v = cstr_to_str(vendor);
        let ck = cstr_to_str(command_key);
        let tt = cstr_to_str(template_text);
        let ot = cstr_to_str(output_text);

        textfsm_core::parse_json(v, ck, tt, ot)
    });

    match result {
        Ok(json_string) => match CString::new(json_string) {
            Ok(cs) => cs.into_raw() as *const c_char,
            Err(_) => error_envelope("INTERNAL_ERROR", "JSON contained interior NUL byte"),
        },
        Err(_) => error_envelope("INTERNAL_ERROR", "Internal panic caught at FFI boundary"),
    }
}

/// Free a string previously returned by `textfsm_parse_json`.
///
/// # Safety
/// `s` must be a pointer previously returned by `textfsm_parse_json`, or null.
/// Passing any other pointer is undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn textfsm_free(s: *const c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s as *mut c_char));
    }
}
