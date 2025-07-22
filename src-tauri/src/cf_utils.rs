use std::ffi::{CStr, CString};

use core_foundation::array::{CFArray, CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
use core_foundation::base::{CFGetTypeID, CFRelease, CFShow, CFTypeRef};
use core_foundation::dictionary::{
    kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks, CFDictionaryCreate,
    CFDictionaryGetCount, CFDictionaryGetKeysAndValues, CFDictionaryGetValue, CFDictionaryRef,
};
use core_foundation::number::{
    kCFNumberIntType, kCFNumberSInt32Type, CFNumberGetTypeID, CFNumberGetValue, CFNumberRef,
};
use core_foundation::string::{
    kCFStringEncodingUTF8, CFStringCreateWithCString, CFStringGetCString, CFStringGetCStringPtr,
    CFStringGetTypeID, CFStringRef,
};
use std::ptr;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Create a CFString from a Rust string
pub fn create_cfstring(s: &str) -> CFStringRef {
    let c_str = CString::new(s).unwrap();
    unsafe { CFStringCreateWithCString(ptr::null(), c_str.as_ptr(), kCFStringEncodingUTF8) }
}

/// Convert CFString to Rust String
pub fn cfstring_to_string(cf_str: CFStringRef) -> String {
    if cf_str.is_null() {
        return String::new();
    }

    unsafe {
        // First try the fast path with CFStringGetCStringPtr
        let c_str_ptr = CFStringGetCStringPtr(cf_str, kCFStringEncodingUTF8);
        if !c_str_ptr.is_null() {
            let c_str = CStr::from_ptr(c_str_ptr);
            return c_str.to_string_lossy().into_owned();
        }

        // If that fails, we need to copy the string data
        // Get the length first
        let length = core_foundation::string::CFStringGetLength(cf_str);
        if length == 0 {
            return String::new();
        }

        // Allocate a buffer for the C string (length + 1 for null terminator)
        let max_size = (length * 4 + 1) as usize; // UTF-8 can be up to 4 bytes per character
        let mut buffer = vec![0u8; max_size];

        // Copy the string data
        let success = core_foundation::string::CFStringGetCString(
            cf_str,
            buffer.as_mut_ptr() as *mut i8,
            max_size as isize,
            kCFStringEncodingUTF8,
        );

        if success != 0 {
            // Find the null terminator and create a string from the buffer
            if let Some(null_pos) = buffer.iter().position(|&x| x == 0) {
                buffer.truncate(null_pos);
            }
            String::from_utf8_lossy(&buffer).into_owned()
        } else {
            println!("WARNING: Failed to convert CFString to C string");
            String::new()
        }
    }
}

/// Get string value from CFDictionary with type checking
pub fn get_dict_string(dict: CFDictionaryRef, key: &str) -> String {
    unsafe {
        let key_cfstr = create_cfstring(key);
        let value_ref = CFDictionaryGetValue(dict, key_cfstr as *const std::ffi::c_void);
        CFRelease(key_cfstr as CFTypeRef);

        if !value_ref.is_null() {
            let type_id = CFGetTypeID(value_ref);
            let string_type_id = CFStringGetTypeID();

            if type_id == string_type_id {
                // It's a CFString, extract it normally
                cfstring_to_string(value_ref as CFStringRef)
            } else {
                // It's not a CFString, let's see what it is
                println!(
                    "WARNING: Key '{}' is not a CFString (TypeID: {} vs expected: {})",
                    key, type_id, string_type_id
                );

                // Try to convert other types to string representation
                let number_type_id = CFNumberGetTypeID();
                if type_id == number_type_id {
                    // It's a number, convert to string
                    let mut number_value: i32 = 0;
                    if CFNumberGetValue(
                        value_ref as CFNumberRef,
                        kCFNumberSInt32Type,
                        &mut number_value as *mut i32 as *mut std::ffi::c_void,
                    ) {
                        format!("{}", number_value)
                    } else {
                        format!("<CFNumber conversion failed>")
                    }
                } else {
                    // Unknown type, use CFShow to get a description
                    println!("Unknown type for key '{}', using CFShow:", key);
                    CFShow(value_ref);
                    format!("<Unknown type: TypeID {}>", type_id)
                }
            }
        } else {
            String::new()
        }
    }
}

/// Get number value from CFDictionary
pub fn get_dict_number(dict: CFDictionaryRef, key: &str) -> i32 {
    unsafe {
        let key_cfstr = create_cfstring(key);
        let value_ref = CFDictionaryGetValue(dict, key_cfstr as *const std::ffi::c_void);
        CFRelease(key_cfstr as CFTypeRef);

        if !value_ref.is_null() {
            let mut number_value: i32 = 0;
            if CFNumberGetValue(
                value_ref as CFNumberRef,
                kCFNumberSInt32Type,
                &mut number_value as *mut i32 as *mut std::ffi::c_void,
            ) {
                number_value
            } else {
                0
            }
        } else {
            0
        }
    }
}

/// Get bounds from CFDictionary
pub fn get_dict_bounds(dict: CFDictionaryRef, key: &str) -> (f64, f64, f64, f64) {
    unsafe {
        let key_cfstr = create_cfstring(key);
        let bounds_ref = CFDictionaryGetValue(dict, key_cfstr as *const std::ffi::c_void);
        CFRelease(key_cfstr as CFTypeRef);

        if !bounds_ref.is_null() {
            let bounds_dict = bounds_ref as CFDictionaryRef;
            let x = get_dict_number(bounds_dict, "X") as f64;
            let y = get_dict_number(bounds_dict, "Y") as f64;
            let width = get_dict_number(bounds_dict, "Width") as f64;
            let height = get_dict_number(bounds_dict, "Height") as f64;
            (x, y, width, height)
        } else {
            (0.0, 0.0, 0.0, 0.0)
        }
    }
}

/// Print all key-value pairs in a CFDictionary
pub fn print_full_dictionary(dict: CFDictionaryRef) {
    unsafe {
        // Use CFShow to print the entire dictionary structure
        CFShow(dict as CFTypeRef);
    }

    // Get all keys and values from the dictionary
    unsafe {
        use core_foundation::dictionary::{CFDictionaryGetCount, CFDictionaryGetKeysAndValues};

        let count = CFDictionaryGetCount(dict);
        println!("  Dictionary has {} key-value pairs:", count);

        if count > 0 {
            // Allocate arrays for keys and values
            let mut keys: Vec<*const std::ffi::c_void> = vec![std::ptr::null(); count as usize];
            let mut values: Vec<*const std::ffi::c_void> = vec![std::ptr::null(); count as usize];

            // Get all keys and values
            CFDictionaryGetKeysAndValues(dict, keys.as_mut_ptr(), values.as_mut_ptr());

            // Iterate through all key-value pairs
            for i in 0..count {
                let key_ref = keys[i as usize];
                let value_ref = values[i as usize];

                if !key_ref.is_null() && !value_ref.is_null() {
                    // Convert key to string
                    let key_string = cfstring_to_string(key_ref as CFStringRef);

                    // Get type information for the value
                    let type_id = CFGetTypeID(value_ref);
                    let string_type_id = CFStringGetTypeID();
                    let number_type_id = CFNumberGetTypeID();

                    let type_name = if type_id == string_type_id {
                        "CFString"
                    } else if type_id == number_type_id {
                        "CFNumber"
                    } else {
                        "Unknown"
                    };

                    println!("  {}: TypeID {} ({})", key_string, type_id, type_name);

                    // Show the actual value using CFShow for non-strings
                    if type_id != string_type_id {
                        print!("    Value: ");
                        CFShow(value_ref);
                    } else {
                        // For strings, also show the converted value
                        let string_value = cfstring_to_string(value_ref as CFStringRef);
                        println!("    Value: \"{}\"", string_value);
                    }
                } else {
                    println!("  Key or value at index {} is NULL", i);
                }
            }
        }
    }
}
