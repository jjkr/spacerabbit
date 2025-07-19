use core_foundation::array::{CFArrayRef, CFArrayGetCount, CFArrayGetValueAtIndex};
use core_foundation::base::{CFTypeRef, CFRelease, CFGetTypeID, CFShow};
use core_foundation::string::{CFStringRef, CFStringCreateWithCString, CFStringGetCStringPtr, CFStringGetCString, kCFStringEncodingUTF8, CFStringGetTypeID};
use core_foundation::number::{CFNumberRef, CFNumberGetValue, kCFNumberSInt32Type, CFNumberGetTypeID};
use core_foundation::dictionary::{CFDictionaryRef, CFDictionaryGetValue};
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

// Window ordering constants
const kCGSOrderAbove: i32 = 1;

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
// WINDOW CYCLING STATE MANAGEMENT
// =============================================================================

// Simple state tracking for window cycling
static mut LAST_FOCUSED_PID: Option<i32> = None;
static mut WINDOW_CYCLE_INDEX: usize = 0;

/// Update the last focused PID for cycling state
fn update_last_focused_pid(pid: i32) {
    unsafe {
        LAST_FOCUSED_PID = Some(pid);
    }
}

/// Get the last focused PID
fn get_last_focused_pid() -> Option<i32> {
    unsafe {
        LAST_FOCUSED_PID
    }
}

// =============================================================================
// PUBLIC API
// =============================================================================

/// Get all visible windows on the current workspace
pub fn get_current_workspace_windows() -> Result<Vec<WindowInfo>, String> {
    unsafe {
        // Get window list - only on-screen windows, excluding desktop elements
        let window_list = CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
            0 // kCGNullWindowID
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
                
                // Print the full dictionary for this window
                //print_full_dictionary(window_dict, i as usize);
                
                // Get window properties
                let window_id = get_dict_number(window_dict, "kCGWindowNumber") as CGWindowID;
                let pid = get_dict_number(window_dict, "kCGWindowOwnerPID");
                let layer = get_dict_number(window_dict, "kCGWindowLayer");
                let title = get_dict_string(window_dict, "kCGWindowName");
                let app_name = get_dict_string(window_dict, "kCGWindowOwnerName");
                let bounds = get_dict_bounds(window_dict, "kCGWindowBounds");

                println!("Found window: {} (PID: {}, Layer: {}, Title: {}, AppName: {}, Bounds: {:?})", title, pid, layer, title, app_name, bounds);

                // Filter out windows without titles or that are too small
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
        
        // Sort by layer (stacking order) - lower layer numbers are typically on top
        windows.sort_by(|a, b| a.layer.cmp(&b.layer));
        
        Ok(windows)
    }
}

/// Focus a specific window by bringing it to front using AppleScript
pub fn focus_window(window: &WindowInfo) -> Result<(), String> {
    // Try AppleScript approach for better compatibility
    let script = if !window.title.is_empty() {
        format!(
            r#"tell application "{}" to activate
tell application "System Events" to tell process "{}" to set frontmost to true"#,
            window.app_name, window.app_name
        )
    } else {
        format!(
            r#"tell application "System Events" to tell process "{}" to set frontmost to true"#,
            window.app_name
        )
    };
    
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("Successfully focused window: {} ({})", window.title, window.app_name);
                // Update our tracking state
                update_last_focused_pid(window.pid);
                Ok(())
            } else {
                let error = String::from_utf8_lossy(&result.stderr);
                // Fallback to Core Graphics if AppleScript fails
                println!("AppleScript failed ({}), trying Core Graphics fallback", error);
                focus_window_cg(window)
            }
        }
        Err(e) => {
            println!("Failed to execute AppleScript ({}), trying Core Graphics fallback", e);
            focus_window_cg(window)
        }
    }
}

/// Fallback Core Graphics window focusing
fn focus_window_cg(window: &WindowInfo) -> Result<(), String> {
    unsafe {
        let connection_id = CGSMainConnectionID();
        
        let result = CGSOrderWindow(
            connection_id,
            window.window_id,
            kCGSOrderAbove,
            0 // Above all windows
        );
        
        if result == 0 {
            println!("Successfully focused window (CG fallback): {} ({})", window.title, window.app_name);
            // Update our tracking state
            update_last_focused_pid(window.pid);
            Ok(())
        } else {
            Err(format!("Both AppleScript and CG focus failed: CG error {}", result))
        }
    }
}

/// Get the currently focused window using improved layer-based detection
pub fn get_focused_window() -> Result<Option<WindowInfo>, String> {
    let windows = get_current_workspace_windows()?;
    
    // Check if we have a recently focused window that's still visible
    if let Some(last_pid) = get_last_focused_pid() {
        if let Some(window) = windows.iter().find(|w| w.pid == last_pid) {
            return Ok(Some(window.clone()));
        }
    }
    
    // Fallback to the window with the lowest layer (most likely to be on top)
    Ok(windows.into_iter().min_by_key(|w| w.layer))
}

/// Cycle to the next window with improved focus detection and filtering
pub fn cycle_next_window() -> Result<(), String> {
    let mut windows = get_current_workspace_windows()?;
    
    if windows.is_empty() {
        return Err("No windows found on current workspace".to_string());
    }
    
    // Filter to only focusable windows
    windows.retain(|w| {
        // More permissive filtering: reasonable bounds and either has a title or is a known app
        let has_reasonable_bounds = w.bounds.2 > 100.0 && w.bounds.3 > 100.0;
        let has_title_or_known_app = !w.title.is_empty() || 
            (!w.app_name.is_empty() && !["Window Server", "Control Center", "Hidden Bar", "Calendr"].contains(&w.app_name.as_str()));
        
        has_reasonable_bounds && has_title_or_known_app
    });
    
    if windows.is_empty() {
        return Err("No focusable windows found on current workspace".to_string());
    }
    
    if windows.len() == 1 {
        // Only one window, focus it anyway
        return focus_window(&windows[0]);
    }
    
    // Sort windows by a combination of layer and app name for consistent ordering
    windows.sort_by(|a, b| {
        a.layer.cmp(&b.layer)
            .then_with(|| a.app_name.cmp(&b.app_name))
            .then_with(|| a.title.cmp(&b.title))
    });
    
    // Try to get the currently focused window
    let current_focused = get_focused_window()?;
    
    let next_window = if let Some(focused) = current_focused {
        // Find the index of the currently focused window
        let current_index = windows.iter().position(|w| w.pid == focused.pid && w.window_id == focused.window_id).unwrap_or(0);
        
        // Move to the next window, wrapping around if necessary
        let next_index = (current_index + 1) % windows.len();
        &windows[next_index]
    } else {
        // No focused window found, focus the first one
        &windows[0]
    };
    
    focus_window(next_window)
}
