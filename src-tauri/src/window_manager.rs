use core_foundation::array::{CFArrayRef, CFArrayGetCount, CFArrayGetValueAtIndex};
use core_foundation::base::{CFTypeRef, CFRelease, CFGetTypeID, CFShow};
use core_foundation::string::{CFStringRef, CFStringCreateWithCString, CFStringGetCStringPtr, CFStringGetCString, kCFStringEncodingUTF8, CFStringGetTypeID};
use core_foundation::number::{CFNumberRef, CFNumberGetValue, kCFNumberSInt32Type, CFNumberGetTypeID, kCFNumberIntType};
use core_foundation::dictionary::{CFDictionaryRef, CFDictionaryGetValue, CFDictionaryCreate, kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks};
use core_foundation::boolean::{CFBooleanRef, kCFBooleanTrue};
use std::ffi::{CStr, CString};
use std::ptr;

// =============================================================================
// CORE GRAPHICS WINDOW API BINDINGS
// =============================================================================

type CGWindowID = u32;
type CGSConnectionID = i32;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    // Window list functions
    fn CGWindowListCopyWindowInfo(option: u32, relative_to_window: CGWindowID) -> CFArrayRef;
    fn CGSMainConnectionID() -> CGSConnectionID;
    
    // Window manipulation functions
    fn CGSGetWindowOwner(cid: CGSConnectionID, window_id: CGWindowID, owner_pid: *mut i32) -> i32;
    fn CGSSetWindowLevel(cid: CGSConnectionID, window_id: CGWindowID, level: i32) -> i32;
    fn CGSOrderWindow(cid: CGSConnectionID, window_id: CGWindowID, place: i32, relative_to: CGWindowID) -> i32;
}

// Core Graphics constants
const kCGWindowListOptionOnScreenOnly: u32 = 1 << 0;
const kCGWindowListExcludeDesktopElements: u32 = 1 << 4;
const kCGWindowListOptionAll: u32 = 1 << 2;

// Window ordering constants
const kCGSOrderAbove: i32 = 1;

// =============================================================================
// ACCESSIBILITY API BINDINGS
// =============================================================================

type AXUIElementRef = *mut std::ffi::c_void;
type AXError = i32;
type pid_t = i32;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    // Accessibility permission
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    
    // UI Element creation
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementCreateApplication(pid: pid_t) -> AXUIElementRef;
    
    // Attribute access
    fn AXUIElementCopyAttributeValue(element: AXUIElementRef, attribute: CFStringRef, value: *mut CFTypeRef) -> AXError;
    fn AXUIElementSetAttributeValue(element: AXUIElementRef, attribute: CFStringRef, value: CFTypeRef) -> AXError;
    
    // Actions
    fn AXUIElementPerformAction(element: AXUIElementRef, action: CFStringRef) -> AXError;
}

// Accessibility constants
const kAXErrorSuccess: AXError = 0;

// Accessibility attribute constants - we'll create these as CFStrings
static AX_TRUSTED_CHECK_OPTION_PROMPT: &str = "AXTrustedCheckOptionPrompt";
static AX_FOCUSED_APPLICATION_ATTRIBUTE: &str = "AXFocusedApplication";
static AX_FOCUSED_WINDOW_ATTRIBUTE: &str = "AXFocusedWindow";
static AX_WINDOWS_ATTRIBUTE: &str = "AXWindows";
static AX_WINDOW_NUMBER_ATTRIBUTE: &str = "AXWindowNumber";
static AX_RAISE_ACTION: &str = "AXRaise";

// Core Graphics window info keys
static CG_WINDOW_NUMBER: &str = "kCGWindowNumber";
static CG_WINDOW_OWNER_PID: &str = "kCGWindowOwnerPID";
static CG_WINDOW_LAYER: &str = "kCGWindowLayer";
static CG_WINDOW_NAME: &str = "kCGWindowName";
static CG_WINDOW_OWNER_NAME: &str = "kCGWindowOwnerName";
static CG_WINDOW_BOUNDS: &str = "kCGWindowBounds";

// =============================================================================
// WINDOW INFORMATION STRUCTURE
// =============================================================================

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub window_id: CGWindowID,
    pub pid: i32,
    pub title: String,
    pub app_name: String,
    pub layer: i32,
    pub bounds: (f64, f64, f64, f64), // x, y, width, height
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Create a CFString from a Rust string
fn create_cfstring(s: &str) -> CFStringRef {
    let c_str = CString::new(s).unwrap();
    unsafe {
        CFStringCreateWithCString(
            ptr::null(),
            c_str.as_ptr(),
            kCFStringEncodingUTF8,
        )
    }
}

/// Convert CFString to Rust String
fn cfstring_to_string(cf_str: CFStringRef) -> String {
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
            kCFStringEncodingUTF8
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
fn get_dict_string(dict: CFDictionaryRef, key: &str) -> String {
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
                println!("WARNING: Key '{}' is not a CFString (TypeID: {} vs expected: {})", key, type_id, string_type_id);
                
                // Try to convert other types to string representation
                let number_type_id = CFNumberGetTypeID();
                if type_id == number_type_id {
                    // It's a number, convert to string
                    let mut number_value: i32 = 0;
                    if CFNumberGetValue(
                        value_ref as CFNumberRef,
                        kCFNumberSInt32Type,
                        &mut number_value as *mut i32 as *mut std::ffi::c_void
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
fn get_dict_number(dict: CFDictionaryRef, key: &str) -> i32 {
    unsafe {
        let key_cfstr = create_cfstring(key);
        let value_ref = CFDictionaryGetValue(dict, key_cfstr as *const std::ffi::c_void);
        CFRelease(key_cfstr as CFTypeRef);
        
        if !value_ref.is_null() {
            let mut number_value: i32 = 0;
            if CFNumberGetValue(
                value_ref as CFNumberRef,
                kCFNumberSInt32Type,
                &mut number_value as *mut i32 as *mut std::ffi::c_void
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
fn get_dict_bounds(dict: CFDictionaryRef, key: &str) -> (f64, f64, f64, f64) {
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
fn print_full_dictionary(dict: CFDictionaryRef, window_index: usize) {
    println!("=== Window {} Full Dictionary ===", window_index);
    unsafe {
        // Use CFShow to print the entire dictionary structure
        CFShow(dict as CFTypeRef);
    }
    
    // Also manually inspect some key fields to understand their types
    println!("--- Type Analysis for Window {} ---", window_index);
    let keys_to_check = [
        "kCGWindowOwnerName",
        "kCGWindowName", 
        "kCGWindowNumber",
        "kCGWindowOwnerPID",
        "kCGWindowLayer",
        "kCGWindowBounds"
    ];
    
    unsafe {
        for key in &keys_to_check {
            let key_cfstr = create_cfstring(key);
            let value_ref = CFDictionaryGetValue(dict, key_cfstr as *const std::ffi::c_void);
            CFRelease(key_cfstr as CFTypeRef);
            
            if !value_ref.is_null() {
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
                
                println!("  {}: TypeID {} ({})", key, type_id, type_name);
                
                // Show the actual value using CFShow for non-strings
                if type_id != string_type_id {
                    print!("    Value: ");
                    CFShow(value_ref);
                }
            } else {
                println!("  {}: NULL", key);
            }
        }
    }
    
    println!("=== End Window {} Dictionary ===\n", window_index);
}

// =============================================================================
// ACCESSIBILITY HELPER FUNCTIONS
// =============================================================================

/// Check if accessibility permission is granted
fn check_accessibility_permission() -> Result<(), String> {
    unsafe {
        // Create options dictionary for accessibility check
        let prompt_key = create_cfstring(AX_TRUSTED_CHECK_OPTION_PROMPT);
        let keys = [prompt_key as *const std::ffi::c_void];
        let values = [kCFBooleanTrue as *const std::ffi::c_void];
        
        let options = CFDictionaryCreate(
            ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks,
            &kCFTypeDictionaryValueCallBacks,
        );
        
        let is_trusted = AXIsProcessTrustedWithOptions(options);
        CFRelease(options as CFTypeRef);
        CFRelease(prompt_key as CFTypeRef);
        
        if !is_trusted {
            return Err("Accessibility permission required".to_string());
        }
        
        Ok(())
    }
}

/// Get the integer window number from an AXUIElement
fn get_ax_window_number(window: AXUIElementRef) -> Result<i32, String> {
    println!("DEBUG: get_ax_window_number: Starting for window at address {:p}", window);
    unsafe {
        let attr_name = create_cfstring(AX_WINDOW_NUMBER_ATTRIBUTE);
        let mut value: CFTypeRef = ptr::null_mut();
        
        println!("DEBUG: get_ax_window_number: Calling AXUIElementCopyAttributeValue for attribute '{}'", AX_WINDOW_NUMBER_ATTRIBUTE);
        let result = AXUIElementCopyAttributeValue(window, attr_name, &mut value);
        CFRelease(attr_name as CFTypeRef);
        
        println!("DEBUG: get_ax_window_number: AXUIElementCopyAttributeValue result: {}", result);
        if result != kAXErrorSuccess {
            let error_msg = format!("AXUIElementCopyAttributeValue failed with error: {}", result);
            println!("DEBUG: get_ax_window_number: {}", error_msg);
            return Err(error_msg);
        }
        
        println!("DEBUG: get_ax_window_number: Value pointer: {:p}", value);
        if value.is_null() {
            let error_msg = "AXUIElementCopyAttributeValue returned null value".to_string();
            println!("DEBUG: get_ax_window_number: {}", error_msg);
            return Err(error_msg);
        }
        
        // Check the type of the returned value
        let type_id = CFGetTypeID(value);
        let number_type_id = CFNumberGetTypeID();
        
        println!("DEBUG: get_ax_window_number: Value TypeID: {}, Expected CFNumber TypeID: {}", type_id, number_type_id);
        if type_id != number_type_id {
            CFRelease(value);
            let error_msg = format!("Window number attribute is not a CFNumber (TypeID: {} vs expected: {})", type_id, number_type_id);
            println!("DEBUG: get_ax_window_number: {}", error_msg);
            return Err(error_msg);
        }
        
        let mut window_number: i32 = 0;
        println!("DEBUG: get_ax_window_number: Calling CFNumberGetValue to extract integer value");
        let success = CFNumberGetValue(
            value as CFNumberRef,
            kCFNumberIntType,
            &mut window_number as *mut i32 as *mut std::ffi::c_void,
        );
        
        CFRelease(value);
        
        println!("DEBUG: get_ax_window_number: CFNumberGetValue success: {}", success);
        if !success {
            let error_msg = "CFNumberGetValue failed to convert window number".to_string();
            println!("DEBUG: get_ax_window_number: {}", error_msg);
            return Err(error_msg);
        }
        
        println!("DEBUG: get_ax_window_number: Successfully extracted window number: {}", window_number);
        Ok(window_number)
    }
}

// =============================================================================
// CORE WINDOW SWITCHING IMPLEMENTATION (following C example)
// =============================================================================

/// Get all visible layer-0 window numbers in front-to-back order
fn get_visible_window_numbers() -> Result<Vec<i32>, String> {
    println!("DEBUG: Getting visible window numbers...");
    unsafe {
        let window_list = CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
            0, // kCGNullWindowID
        );
        
        if window_list.is_null() {
            println!("DEBUG: Failed to get window list from CGWindowListCopyWindowInfo");
            return Err("Failed to get window list".to_string());
        }
        
        let window_count = CFArrayGetCount(window_list);
        println!("DEBUG: Found {} total windows in list", window_count);
        let mut window_numbers = Vec::new();
        
        for i in 0..window_count {
            let window_dict_ref = CFArrayGetValueAtIndex(window_list, i);
            if !window_dict_ref.is_null() {
                let window_dict = window_dict_ref as CFDictionaryRef;
                
                // Get window info for debugging
                let window_number = get_dict_number(window_dict, CG_WINDOW_NUMBER);
                let layer = get_dict_number(window_dict, CG_WINDOW_LAYER);
                let pid = get_dict_number(window_dict, CG_WINDOW_OWNER_PID);
                let app_name = get_dict_string(window_dict, CG_WINDOW_OWNER_NAME);
                let window_name = get_dict_string(window_dict, CG_WINDOW_NAME);
                
                println!("DEBUG: Window {}: ID={}, Layer={}, PID={}, App='{}', Title='{}'", 
                    i, window_number, layer, pid, app_name, window_name);
                
                // Only layer 0 windows
                if layer != 0 {
                    println!("DEBUG: Skipping window {} (layer {})", window_number, layer);
                    continue;
                }
                
                // Extract the window number
                window_numbers.push(window_number);
                println!("DEBUG: Added layer-0 window {} to list", window_number);
            }
        }
        
        CFRelease(window_list as CFTypeRef);
        
        println!("DEBUG: Final visible window numbers: {:?}", window_numbers);
        
        if window_numbers.is_empty() {
            println!("DEBUG: No visible layer-0 windows found");
            return Err("No visible windows found".to_string());
        }
        
        Ok(window_numbers)
    }
}

/// Get the currently focused window number using Accessibility APIs
fn get_focused_window_number() -> Result<i32, String> {
    println!("DEBUG: Getting focused window number...");
    unsafe {
        let sys_wide = AXUIElementCreateSystemWide();
        if sys_wide.is_null() {
            println!("DEBUG: Failed to create system-wide element");
            return Err("Failed to create system-wide element".to_string());
        }
        
        // DEBUG: Print all properties of sys_wide
        println!("DEBUG: ========== sys_wide element properties ==========");
        println!("DEBUG: sys_wide element address: {:p}", sys_wide);
        
        // First check accessibility permission status
        let accessibility_check = check_accessibility_permission();
        match &accessibility_check {
            Ok(_) => println!("DEBUG: Accessibility permission check: GRANTED"),
            Err(e) => println!("DEBUG: Accessibility permission check: FAILED - {}", e),
        }
        
        // Get all available attributes for sys_wide
        let mut attribute_names: CFTypeRef = ptr::null_mut();
        let attr_names_attr = create_cfstring("AXAttributeNames");
        let attr_result = AXUIElementCopyAttributeValue(sys_wide, attr_names_attr, &mut attribute_names);
        CFRelease(attr_names_attr as CFTypeRef);
        
        println!("DEBUG: AXUIElementCopyAttributeValue for AXAttributeNames result: {}", attr_result);
        
        // Decode the error code
        let error_description = match attr_result {
            0 => "kAXErrorSuccess",
            -25200 => "kAXErrorFailure",
            -25201 => "kAXErrorIllegalArgument", 
            -25202 => "kAXErrorInvalidUIElement",
            -25203 => "kAXErrorInvalidUIElementObserver",
            -25204 => "kAXErrorCannotComplete",
            -25205 => "kAXErrorAPIDisabled",
            -25206 => "kAXErrorNoValue",
            -25207 => "kAXErrorParameterizedAttributeUnsupported",
            -25208 => "kAXErrorNotEnoughPrecision",
            _ => "Unknown error code",
        };
        println!("DEBUG: Error code {} means: {}", attr_result, error_description);
        
        if attr_result == kAXErrorSuccess && !attribute_names.is_null() {
            let attr_count = CFArrayGetCount(attribute_names as CFArrayRef);
            println!("DEBUG: sys_wide has {} attributes:", attr_count);
            
            for j in 0..attr_count {
                let attr_name_ref = CFArrayGetValueAtIndex(attribute_names as CFArrayRef, j);
                if !attr_name_ref.is_null() {
                    let attr_name = cfstring_to_string(attr_name_ref as CFStringRef);
                    println!("DEBUG:   - {}", attr_name);
                    
                    // Try to get the value of each attribute for debugging
                    let attr_cfstr = create_cfstring(&attr_name);
                    let mut attr_value: CFTypeRef = ptr::null_mut();
                    let value_result = AXUIElementCopyAttributeValue(sys_wide, attr_cfstr, &mut attr_value);
                    CFRelease(attr_cfstr as CFTypeRef);
                    
                    if value_result == kAXErrorSuccess && !attr_value.is_null() {
                        let type_id = CFGetTypeID(attr_value);
                        let string_type_id = CFStringGetTypeID();
                        let number_type_id = CFNumberGetTypeID();
                        
                        let type_name = if type_id == string_type_id {
                            "CFString"
                        } else if type_id == number_type_id {
                            "CFNumber"
                        } else {
                            "Unknown"
                        };
                        
                        println!("DEBUG:     Value TypeID: {} ({})", type_id, type_name);
                        
                        // Show the actual value using CFShow
                        print!("DEBUG:     Value: ");
                        CFShow(attr_value);
                        
                        CFRelease(attr_value);
                    } else {
                        let value_error_desc = match value_result {
                            0 => "kAXErrorSuccess",
                            -25200 => "kAXErrorFailure",
                            -25201 => "kAXErrorIllegalArgument", 
                            -25202 => "kAXErrorInvalidUIElement",
                            -25203 => "kAXErrorInvalidUIElementObserver",
                            -25204 => "kAXErrorCannotComplete",
                            -25205 => "kAXErrorAPIDisabled",
                            -25206 => "kAXErrorNoValue",
                            -25207 => "kAXErrorParameterizedAttributeUnsupported",
                            -25208 => "kAXErrorNotEnoughPrecision",
                            _ => "Unknown error code",
                        };
                        println!("DEBUG:     Failed to get value (result: {} - {})", value_result, value_error_desc);
                    }
                }
            }
            CFRelease(attribute_names);
        } else {
            println!("DEBUG: sys_wide: failed to get attribute names");
            if attr_result == -25205 {
                println!("DEBUG: This is kAXErrorAPIDisabled - Accessibility API is disabled!");
                println!("DEBUG: The application needs accessibility permissions to work.");
                println!("DEBUG: Go to System Preferences > Security & Privacy > Privacy > Accessibility");
                println!("DEBUG: and make sure your application is listed and enabled.");
            }
        }
        
        // Try to directly access the focused application attribute anyway
        println!("DEBUG: Attempting direct access to AXFocusedApplication attribute...");
        let focused_app_attr = create_cfstring(AX_FOCUSED_APPLICATION_ATTRIBUTE);
        let mut focused_app_test: CFTypeRef = ptr::null_mut();
        let direct_result = AXUIElementCopyAttributeValue(sys_wide, focused_app_attr, &mut focused_app_test);
        CFRelease(focused_app_attr as CFTypeRef);
        
        let direct_error_desc = match direct_result {
            0 => "kAXErrorSuccess",
            -25200 => "kAXErrorFailure",
            -25201 => "kAXErrorIllegalArgument", 
            -25202 => "kAXErrorInvalidUIElement",
            -25203 => "kAXErrorInvalidUIElementObserver",
            -25204 => "kAXErrorCannotComplete",
            -25205 => "kAXErrorAPIDisabled",
            -25206 => "kAXErrorNoValue",
            -25207 => "kAXErrorParameterizedAttributeUnsupported",
            -25208 => "kAXErrorNotEnoughPrecision",
            _ => "Unknown error code",
        };
        println!("DEBUG: Direct AXFocusedApplication access result: {} ({})", direct_result, direct_error_desc);
        
        if direct_result == kAXErrorSuccess && !focused_app_test.is_null() {
            println!("DEBUG: Successfully got focused application directly!");
            CFRelease(focused_app_test);
        }
        
        println!("DEBUG: ========== End sys_wide properties ==========");
        
        // Get focused application
        let focused_app_attr = create_cfstring(AX_FOCUSED_APPLICATION_ATTRIBUTE);
        let mut focused_app: CFTypeRef = ptr::null_mut();
        let result = AXUIElementCopyAttributeValue(sys_wide, focused_app_attr, &mut focused_app);
        CFRelease(sys_wide as CFTypeRef);
        CFRelease(focused_app_attr as CFTypeRef);
        
        if result != kAXErrorSuccess || focused_app.is_null() {
            println!("DEBUG: Cannot get focused application (result: {})", result);
            return Err("Cannot get focused application".to_string());
        }
        
        println!("DEBUG: Successfully got focused application");
        
        // Get focused window
        let focused_win_attr = create_cfstring(AX_FOCUSED_WINDOW_ATTRIBUTE);
        let mut focused_win: CFTypeRef = ptr::null_mut();
        let result = AXUIElementCopyAttributeValue(
            focused_app as AXUIElementRef,
            focused_win_attr,
            &mut focused_win,
        );
        CFRelease(focused_win_attr as CFTypeRef);
        
        if result != kAXErrorSuccess || focused_win.is_null() {
            println!("DEBUG: Cannot get focused window (result: {})", result);
            CFRelease(focused_app);
            return Err("Cannot get focused window".to_string());
        }
        
        println!("DEBUG: Successfully got focused window");
        
        // Get window number
        let window_number = get_ax_window_number(focused_win as AXUIElementRef);
        
        CFRelease(focused_win);
        CFRelease(focused_app);
        
        match &window_number {
            Ok(num) => println!("DEBUG: Focused window number: {}", num),
            Err(e) => println!("DEBUG: Failed to get window number: {}", e),
        }
        
        window_number
    }
}

/// Focus a window by its window number using Accessibility APIs
fn focus_window_by_number(target_window_number: i32) -> Result<(), String> {
    println!("DEBUG: Attempting to focus window {}", target_window_number);
    unsafe {
        // First, find the PID that owns this window
        println!("DEBUG: Looking up window info for window {}", target_window_number);
        let window_list = CGWindowListCopyWindowInfo(kCGWindowListOptionAll, target_window_number as CGWindowID);
        if window_list.is_null() {
            println!("DEBUG: Failed to get window info for window {}", target_window_number);
            return Err("Failed to get window info".to_string());
        }
        
        let window_count = CFArrayGetCount(window_list);
        println!("DEBUG: Found {} window entries for window {}", window_count, target_window_number);
        if window_count == 0 {
            CFRelease(window_list as CFTypeRef);
            println!("DEBUG: No window entries found for window {}", target_window_number);
            return Err(format!("Cannot find window {}", target_window_number));
        }
        
        let window_dict = CFArrayGetValueAtIndex(window_list, 0) as CFDictionaryRef;
        let target_pid = get_dict_number(window_dict, CG_WINDOW_OWNER_PID);
        let app_name = get_dict_string(window_dict, CG_WINDOW_OWNER_NAME);
        let window_name = get_dict_string(window_dict, CG_WINDOW_NAME);
        println!("DEBUG: Window {} belongs to PID {} (app: '{}', title: '{}')", 
            target_window_number, target_pid, app_name, window_name);
        CFRelease(window_list as CFTypeRef);
        
        if target_pid == 0 {
            println!("DEBUG: Invalid PID (0) for window {}", target_window_number);
            return Err(format!("Cannot determine PID for window {}", target_window_number));
        }
        
        // Create the app element and find the AX window element with matching window number
        println!("DEBUG: Creating application element for PID {}", target_pid);
        let app_element = AXUIElementCreateApplication(target_pid);
        if app_element.is_null() {
            println!("DEBUG: Failed to create application element for PID {}", target_pid);
            return Err("Failed to create application element".to_string());
        }
        
        println!("DEBUG: Getting windows list from application element");
        let windows_attr = create_cfstring(AX_WINDOWS_ATTRIBUTE);
        let mut app_windows: CFTypeRef = ptr::null_mut();
        let result = AXUIElementCopyAttributeValue(app_element, windows_attr, &mut app_windows);
        CFRelease(windows_attr as CFTypeRef);
        
        if result != kAXErrorSuccess || app_windows.is_null() {
            println!("DEBUG: Cannot get application windows (result: {})", result);
            return Err("Cannot get application windows".to_string());
        }
        
        let window_count = CFArrayGetCount(app_windows as CFArrayRef);
        println!("DEBUG: Application has {} AX windows", window_count);
        let mut target_window: AXUIElementRef = ptr::null_mut();
        
        for i in 0..window_count {
            let window = CFArrayGetValueAtIndex(app_windows as CFArrayRef, i) as AXUIElementRef;
            println!("DEBUG: Examining AX window {} at address {:p}", i, window);
            
            // Let's try to get all available attributes for this window
            let mut attribute_names: CFTypeRef = ptr::null_mut();
            let attr_names_attr = create_cfstring("AXAttributeNames");
            let attr_result = AXUIElementCopyAttributeValue(window, attr_names_attr, &mut attribute_names);
            CFRelease(attr_names_attr as CFTypeRef);
            
            if attr_result == kAXErrorSuccess && !attribute_names.is_null() {
                let attr_count = CFArrayGetCount(attribute_names as CFArrayRef);
                println!("DEBUG: AX window {} has {} attributes:", i, attr_count);
                for j in 0..attr_count {
                    let attr_name_ref = CFArrayGetValueAtIndex(attribute_names as CFArrayRef, j);
                    if !attr_name_ref.is_null() {
                        let attr_name = cfstring_to_string(attr_name_ref as CFStringRef);
                        println!("DEBUG:   - {}", attr_name);
                    }
                }
                CFRelease(attribute_names);
            } else {
                println!("DEBUG: AX window {}: failed to get attribute names (result: {})", i, attr_result);
            }
            
            match get_ax_window_number(window) {
                Ok(window_num) => {
                    println!("DEBUG: AX window {}: window number = {}", i, window_num);
                    if window_num == target_window_number {
                        println!("DEBUG: Found matching AX window at index {}", i);
                        target_window = window;
                        break;
                    }
                }
                Err(e) => {
                    println!("DEBUG: AX window {}: failed to get window number: {}", i, e);
                }
            }
        }
        
        if target_window.is_null() {
            println!("DEBUG: Could not find AX window matching number {} among {} AX windows", 
                target_window_number, window_count);
            CFRelease(app_windows);
            return Err(format!("Couldn't find AX window for number {}", target_window_number));
        }
        
        // Focus the window using Accessibility APIs
        println!("DEBUG: Performing AXRaise action on window {}", target_window_number);
        let raise_action = create_cfstring(AX_RAISE_ACTION);
        let raise_result = AXUIElementPerformAction(target_window, raise_action);
        CFRelease(raise_action as CFTypeRef);
        println!("DEBUG: AXRaise result: {}", raise_result);
        
        println!("DEBUG: Setting focused window attribute");
        let focused_win_attr = create_cfstring(AX_FOCUSED_WINDOW_ATTRIBUTE);
        let focus_result = AXUIElementSetAttributeValue(app_element, focused_win_attr, target_window as CFTypeRef);
        CFRelease(focused_win_attr as CFTypeRef);
        println!("DEBUG: Set focused window result: {}", focus_result);
        
        CFRelease(app_windows);
        
        println!("DEBUG: Successfully focused window {}", target_window_number);
        Ok(())
    }
}

// =============================================================================
// PUBLIC API
// =============================================================================

/// Get all visible windows on the current workspace (for compatibility)
pub fn get_current_workspace_windows() -> Result<Vec<WindowInfo>, String> {
    unsafe {
        let window_list = CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
            0,
        );
        
        if window_list.is_null() {
            return Err("Failed to get window list".to_string());
        }
        
        let window_count = CFArrayGetCount(window_list);
        let mut windows = Vec::new();
        
        for i in 0..window_count {
            let window_dict_ref = CFArrayGetValueAtIndex(window_list, i);
            if !window_dict_ref.is_null() {
                let window_dict = window_dict_ref as CFDictionaryRef;
                
                let window_id = get_dict_number(window_dict, CG_WINDOW_NUMBER) as CGWindowID;
                let pid = get_dict_number(window_dict, CG_WINDOW_OWNER_PID);
                let layer = get_dict_number(window_dict, CG_WINDOW_LAYER);
                let title = get_dict_string(window_dict, CG_WINDOW_NAME);
                let app_name = get_dict_string(window_dict, CG_WINDOW_OWNER_NAME);
                let bounds = get_dict_bounds(window_dict, CG_WINDOW_BOUNDS);
                
                if layer == 0 {
                    windows.push(WindowInfo {
                        window_id,
                        pid,
                        title,
                        app_name,
                        layer,
                        bounds,
                    });
                }
            }
        }
        
        CFRelease(window_list as CFTypeRef);
        Ok(windows)
    }
}

/// Get the currently focused window (for compatibility)
pub fn get_focused_window() -> Result<Option<WindowInfo>, String> {
    match get_focused_window_number() {
        Ok(window_number) => {
            let windows = get_current_workspace_windows()?;
            Ok(windows.into_iter().find(|w| w.window_id == window_number as u32))
        }
        Err(_) => Ok(None),
    }
}

/// Focus a specific window (for compatibility)
pub fn focus_window(window: &WindowInfo) -> Result<(), String> {
    focus_window_by_number(window.window_id as i32)
}

/// Cycle to the next window using the proper accessibility-based approach
pub fn cycle_next_window() -> Result<(), String> {
    println!("DEBUG: ========== Starting window cycling ==========");
    
    // Check accessibility permission first
    println!("DEBUG: Checking accessibility permission...");
    check_accessibility_permission()?;
    println!("DEBUG: Accessibility permission OK");
    
    // Get all visible layer-0 window numbers in front-to-back order
    let window_numbers = get_visible_window_numbers()?;
    
    if window_numbers.is_empty() {
        println!("DEBUG: No visible windows found, aborting");
        return Err("No visible windows found".to_string());
    }
    
    println!("DEBUG: Found {} visible windows", window_numbers.len());
    
    if window_numbers.len() == 1 {
        // Only one window, focus it
        println!("DEBUG: Only one window available, focusing it: {}", window_numbers[0]);
        return focus_window_by_number(window_numbers[0]);
    }
    
    // Get the current focused window number
    let current_window_number = match get_focused_window_number() {
        Ok(num) => {
            println!("DEBUG: Current focused window: {}", num);
            num
        }
        Err(e) => {
            println!("DEBUG: Could not get focused window ({}), focusing first available: {}", e, window_numbers[0]);
            // If we can't get the focused window, just focus the first one
            return focus_window_by_number(window_numbers[0]);
        }
    };
    
    // Find the current window in our list and pick the next one
    let current_index = window_numbers.iter().position(|&num| num == current_window_number);
    let next_index = match current_index {
        Some(idx) => {
            let next = (idx + 1) % window_numbers.len();
            println!("DEBUG: Current window {} is at index {}, next index: {}", current_window_number, idx, next);
            next
        }
        None => {
            println!("DEBUG: Current window {} not found in visible list, starting from beginning", current_window_number);
            0 // Current window not found, start from beginning
        }
    };
    
    let target_window_number = window_numbers[next_index];
    println!("DEBUG: Target window to focus: {}", target_window_number);
    
    let result = focus_window_by_number(target_window_number);
    match &result {
        Ok(_) => println!("DEBUG: ========== Window cycling completed successfully =========="),
        Err(e) => println!("DEBUG: ========== Window cycling failed: {} ==========", e),
    }
    
    result
}
