use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic;

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
        r#"{{"ok":false,"error":{{"code":"{code}","message":"{message}"}}}}"#
    );
    match CString::new(json) {
        Ok(cs) => cs.into_raw() as *const c_char,
        Err(_) => std::ptr::null(),
    }
}

/// Parse network device CLI output into a structured JSON envelope.
///
/// # Safety
/// All pointer arguments must be valid, null-terminated C strings (or null).
/// The caller **must** free the returned pointer with [`netcli_free`].
#[no_mangle]
pub unsafe extern "C" fn netcli_parse_json(
    platform: *const c_char,
    command_key: *const c_char,
    output_text: *const c_char,
) -> *const c_char {
    let result = panic::catch_unwind(|| {
        let p = cstr_to_str(platform);
        let ck = cstr_to_str(command_key);
        let ot = cstr_to_str(output_text);

        netcli_core::parse_json(p, ck, ot)
    });

    match result {
        Ok(json_string) => match CString::new(json_string) {
            Ok(cs) => cs.into_raw() as *const c_char,
            Err(_) => error_envelope("INTERNAL_ERROR", "JSON contained interior NUL byte"),
        },
        Err(_) => error_envelope("INTERNAL_ERROR", "Internal panic caught at FFI boundary"),
    }
}

/// Parse network device CLI output using a raw command string (e.g. "show version").
///
/// The command is normalized to a registry key internally (spaces become underscores,
/// lowercased). Otherwise identical to [`netcli_parse_json`].
///
/// # Safety
/// All pointer arguments must be valid, null-terminated C strings (or null).
/// The caller **must** free the returned pointer with [`netcli_free`].
#[no_mangle]
pub unsafe extern "C" fn netcli_parse_command_json(
    platform: *const c_char,
    command: *const c_char,
    output_text: *const c_char,
) -> *const c_char {
    let result = panic::catch_unwind(|| {
        let p = cstr_to_str(platform);
        let cmd = cstr_to_str(command);
        let ot = cstr_to_str(output_text);

        netcli_core::parse_command_json(p, cmd, ot)
    });

    match result {
        Ok(json_string) => match CString::new(json_string) {
            Ok(cs) => cs.into_raw() as *const c_char,
            Err(_) => error_envelope("INTERNAL_ERROR", "JSON contained interior NUL byte"),
        },
        Err(_) => error_envelope("INTERNAL_ERROR", "Internal panic caught at FFI boundary"),
    }
}

/// Free a string previously returned by [`netcli_parse_json`].
///
/// # Safety
/// `s` must be a pointer previously returned by `netcli_parse_json`, or null.
#[no_mangle]
pub unsafe extern "C" fn netcli_free(s: *const c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s as *mut c_char));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn make_c(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn ffi_round_trip_success() {
        let platform = make_c("cisco_ios");
        let cmd = make_c("show_version");
        let output = make_c("some device output");

        unsafe {
            let ptr = netcli_parse_json(
                platform.as_ptr(),
                cmd.as_ptr(),
                output.as_ptr(),
            );
            assert!(!ptr.is_null());

            let json_str = CStr::from_ptr(ptr).to_str().unwrap();
            let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
            assert_eq!(v["ok"], true);
            assert_eq!(v["platform"], "cisco_ios");

            netcli_free(ptr);
        }
    }

    #[test]
    fn ffi_null_platform_returns_error() {
        let cmd = make_c("show_version");
        let output = make_c("text");

        unsafe {
            let ptr = netcli_parse_json(std::ptr::null(), cmd.as_ptr(), output.as_ptr());
            assert!(!ptr.is_null());

            let json_str = CStr::from_ptr(ptr).to_str().unwrap();
            let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
            assert_eq!(v["ok"], false);

            netcli_free(ptr);
        }
    }

    #[test]
    fn ffi_command_round_trip_success() {
        let platform = make_c("cisco_ios");
        let cmd = make_c("show version");
        let output = make_c("some device output");

        unsafe {
            let ptr = netcli_parse_command_json(
                platform.as_ptr(),
                cmd.as_ptr(),
                output.as_ptr(),
            );
            assert!(!ptr.is_null());

            let json_str = CStr::from_ptr(ptr).to_str().unwrap();
            let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
            assert_eq!(v["ok"], true);
            assert_eq!(v["platform"], "cisco_ios");
            assert_eq!(v["commandKey"], "show_version");

            netcli_free(ptr);
        }
    }

    #[test]
    fn ffi_null_command_key_returns_error() {
        let platform = make_c("cisco_ios");
        let output = make_c("text");

        unsafe {
            let ptr = netcli_parse_json(platform.as_ptr(), std::ptr::null(), output.as_ptr());
            assert!(!ptr.is_null());

            let json_str = CStr::from_ptr(ptr).to_str().unwrap();
            let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
            assert_eq!(v["ok"], false);
            assert_eq!(v["error"]["code"], "INVALID_INPUT");

            netcli_free(ptr);
        }
    }

    #[test]
    fn ffi_null_output_returns_error() {
        let platform = make_c("cisco_ios");
        let cmd = make_c("show_version");

        unsafe {
            let ptr = netcli_parse_json(platform.as_ptr(), cmd.as_ptr(), std::ptr::null());
            assert!(!ptr.is_null());

            let json_str = CStr::from_ptr(ptr).to_str().unwrap();
            let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
            assert_eq!(v["ok"], false);
            assert_eq!(v["error"]["code"], "INVALID_INPUT");

            netcli_free(ptr);
        }
    }

    #[test]
    fn ffi_all_null_returns_error() {
        unsafe {
            let ptr = netcli_parse_json(std::ptr::null(), std::ptr::null(), std::ptr::null());
            assert!(!ptr.is_null());

            let json_str = CStr::from_ptr(ptr).to_str().unwrap();
            let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
            assert_eq!(v["ok"], false);

            netcli_free(ptr);
        }
    }

    #[test]
    fn ffi_command_null_platform_returns_error() {
        let cmd = make_c("show version");
        let output = make_c("text");

        unsafe {
            let ptr = netcli_parse_command_json(std::ptr::null(), cmd.as_ptr(), output.as_ptr());
            assert!(!ptr.is_null());

            let json_str = CStr::from_ptr(ptr).to_str().unwrap();
            let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
            assert_eq!(v["ok"], false);

            netcli_free(ptr);
        }
    }

    #[test]
    fn ffi_command_null_command_returns_error() {
        let platform = make_c("cisco_ios");
        let output = make_c("text");

        unsafe {
            let ptr = netcli_parse_command_json(platform.as_ptr(), std::ptr::null(), output.as_ptr());
            assert!(!ptr.is_null());

            let json_str = CStr::from_ptr(ptr).to_str().unwrap();
            let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
            assert_eq!(v["ok"], false);
            assert_eq!(v["error"]["code"], "INVALID_INPUT");

            netcli_free(ptr);
        }
    }

    #[test]
    fn ffi_command_null_output_returns_error() {
        let platform = make_c("cisco_ios");
        let cmd = make_c("show version");

        unsafe {
            let ptr = netcli_parse_command_json(platform.as_ptr(), cmd.as_ptr(), std::ptr::null());
            assert!(!ptr.is_null());

            let json_str = CStr::from_ptr(ptr).to_str().unwrap();
            let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
            assert_eq!(v["ok"], false);
            assert_eq!(v["error"]["code"], "INVALID_INPUT");

            netcli_free(ptr);
        }
    }

    #[test]
    fn ffi_returns_valid_json_for_unknown_platform() {
        let platform = make_c("nonexistent_os");
        let cmd = make_c("show_version");
        let output = make_c("some output");

        unsafe {
            let ptr = netcli_parse_json(platform.as_ptr(), cmd.as_ptr(), output.as_ptr());
            assert!(!ptr.is_null());

            let json_str = CStr::from_ptr(ptr).to_str().unwrap();
            let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
            assert_eq!(v["ok"], false);
            assert_eq!(v["error"]["code"], "TEMPLATE_NOT_FOUND");

            netcli_free(ptr);
        }
    }

    #[test]
    fn ffi_free_null_is_safe() {
        unsafe {
            netcli_free(std::ptr::null());
        }
    }
}
