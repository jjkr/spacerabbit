use crate::cf_utils::{
    cfstring_to_string, create_cfstring, get_dict_bounds, get_dict_number, get_dict_string,
    print_full_dictionary,
};
use core_foundation::array::{CFArray, CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
use core_foundation::base::{CFGetTypeID, CFRelease, CFShow, CFTypeRef};
use core_foundation::boolean::{kCFBooleanTrue, CFBooleanRef};
use core_foundation::dictionary::{
    kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks, CFDictionaryCreate,
    CFDictionaryGetCount, CFDictionaryGetKeysAndValues, CFDictionaryGetTypeID,
    CFDictionaryGetValue, CFDictionaryRef,
};
use core_foundation::number::{
    kCFNumberIntType, kCFNumberSInt32Type, CFNumberGetTypeID, CFNumberGetValue, CFNumberRef,
};
use core_foundation::string::{
    kCFStringEncodingUTF8, CFStringCreateWithCString, CFStringGetCString, CFStringGetCStringPtr,
    CFStringGetTypeID, CFStringRef,
};
use core_graphics::window;
use log::{debug, error, info, warn};
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
    fn CGSOrderWindow(
        cid: CGSConnectionID,
        window_id: CGWindowID,
        place: i32,
        relative_to: CGWindowID,
    ) -> i32;
}

// Core Graphics constants
const kCGWindowListOptionOnScreenOnly: u32 = 1 << 0;
const kCGWindowListOptionAll: u32 = 1 << 2;
const kCGWindowListExcludeDesktopElements: u32 = 1 << 4;

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
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> AXError;
    fn AXUIElementSetAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: CFTypeRef,
    ) -> AXError;
    fn AXUIElementCopyAttributeNames(element: AXUIElementRef, value: *mut CFArrayRef) -> AXError;

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
static AX_TITLE_ATTRIBUTE: &str = "AXTitle";
static AX_POSITION_ATTRIBUTE: &str = "AXPosition";
static AX_SIZE_ATTRIBUTE: &str = "AXSize";
static AX_FRAME_ATTRIBUTE: &str = "AXFrame";
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

/// Get the title from an AXUIElement
fn get_ax_window_title(window: AXUIElementRef) -> String {
    unsafe {
        let attr_name = create_cfstring(AX_TITLE_ATTRIBUTE);
        let mut value: CFTypeRef = ptr::null_mut();

        let result = AXUIElementCopyAttributeValue(window, attr_name, &mut value);
        CFRelease(attr_name as CFTypeRef);

        if result == kAXErrorSuccess && !value.is_null() {
            let title = cfstring_to_string(value as CFStringRef);
            CFRelease(value);
            title
        } else {
            String::new()
        }
    }
}

/// Get the bounds from an AXUIElement (returns x, y, width, height)
fn get_ax_window_bounds(window: AXUIElementRef) -> (f64, f64, f64, f64) {
    unsafe {
        // Try AXFrame first (more comprehensive)
        let frame_attr = create_cfstring(AX_FRAME_ATTRIBUTE);
        let mut frame_value: CFTypeRef = ptr::null_mut();
        let frame_result = AXUIElementCopyAttributeValue(window, frame_attr, &mut frame_value);
        CFRelease(frame_attr as CFTypeRef);

        if frame_result == kAXErrorSuccess && !frame_value.is_null() {
            // AXFrame should be a CGRect structure, but let's be careful about accessing it
            debug!("Successfully got AXFrame value");

            // Check if it's actually a dictionary before treating it as one
            let type_id = CFGetTypeID(frame_value);
            debug!("AXFrame value TypeID: {}", type_id);
            debug!("Dictionary TypeID: {}", CFDictionaryGetTypeID());
            // If it's a dictionary, we can extract bounds from it
            if type_id == CFDictionaryGetTypeID() {
                let bounds_dict = frame_value as CFDictionaryRef;
                let x = get_dict_number(bounds_dict, "X") as f64;
                let y = get_dict_number(bounds_dict, "Y") as f64;
                let width = get_dict_number(bounds_dict, "Width") as f64;
                let height = get_dict_number(bounds_dict, "Height") as f64;
                debug!(
                    "Returning bounds from AXFrame: ({}, {}, {}, {})",
                    x, y, width, height
                );
                return (x, y, width, height);
            }

            // For now, just release and fall back to position/size approach
            CFRelease(frame_value);
        } else {
            debug!("Failed to get AXFrame (result: {})", frame_result);
        }

        // Use AXPosition + AXSize approach (safer)
        let pos_attr = create_cfstring(AX_POSITION_ATTRIBUTE);
        let size_attr = create_cfstring(AX_SIZE_ATTRIBUTE);
        let mut pos_value: CFTypeRef = ptr::null_mut();
        let mut size_value: CFTypeRef = ptr::null_mut();

        let pos_result = AXUIElementCopyAttributeValue(window, pos_attr, &mut pos_value);
        let size_result = AXUIElementCopyAttributeValue(window, size_attr, &mut size_value);

        CFRelease(pos_attr as CFTypeRef);
        CFRelease(size_attr as CFTypeRef);

        let mut x = 0.0;
        let mut y = 0.0;
        let mut width = 0.0;
        let mut height = 0.0;

        if pos_result == kAXErrorSuccess && !pos_value.is_null() {
            debug!("Successfully got AXPosition value");
            let type_id = CFGetTypeID(pos_value);
            debug!("AXPosition value TypeID: {}", type_id);

            // For now, just use default values and release
            CFRelease(pos_value);
        } else {
            debug!("Failed to get AXPosition (result: {})", pos_result);
        }

        if size_result == kAXErrorSuccess && !size_value.is_null() {
            debug!("Successfully got AXSize value");
            let type_id = CFGetTypeID(size_value);
            debug!("AXSize value TypeID: {}", type_id);

            // For now, just use default values and release
            CFRelease(size_value);
        } else {
            debug!("Failed to get AXSize (result: {})", size_result);
        }

        debug!("Returning bounds: ({}, {}, {}, {})", x, y, width, height);
        (x, y, width, height)
    }
}

/// Match an AX window to a CG window using multiple strategies
fn match_ax_window_to_cg_window(
    ax_window: AXUIElementRef,
    target_cg_title: &str,
    target_cg_bounds: (f64, f64, f64, f64),
    ax_index: usize,
) -> bool {
    debug!("Matching AX window {} against CG window", ax_index);

    // Get AX window properties
    let ax_title = get_ax_window_title(ax_window);
    let ax_bounds = get_ax_window_bounds(ax_window);

    debug!(
        "AX window {}: title='{}', bounds={:?}",
        ax_index, ax_title, ax_bounds
    );
    debug!(
        "Target CG window: title='{}', bounds={:?}",
        target_cg_title, target_cg_bounds
    );

    // Strategy 1: Title matching (if both have non-empty titles)
    if !ax_title.is_empty() && !target_cg_title.is_empty() {
        if ax_title == target_cg_title {
            debug!(
                "MATCH: Title match - '{}' == '{}'",
                ax_title, target_cg_title
            );
            return true;
        } else {
            debug!("Title mismatch - '{}' != '{}'", ax_title, target_cg_title);
        }
    } else {
        debug!("Skipping title match (one or both titles empty)");
    }

    // Strategy 2: Bounds matching (with tolerance for slight differences)
    let tolerance = 5.0; // Allow 5 pixel difference
    let x_match = (ax_bounds.0 - target_cg_bounds.0).abs() <= tolerance;
    let y_match = (ax_bounds.1 - target_cg_bounds.1).abs() <= tolerance;
    let w_match = (ax_bounds.2 - target_cg_bounds.2).abs() <= tolerance;
    let h_match = (ax_bounds.3 - target_cg_bounds.3).abs() <= tolerance;

    if x_match && y_match && w_match && h_match {
        debug!(
            "MATCH: Bounds match within tolerance of {} pixels",
            tolerance
        );
        return true;
    } else {
        debug!(
            "Bounds mismatch - x:{}, y:{}, w:{}, h:{}",
            x_match, y_match, w_match, h_match
        );
    }

    // Strategy 3: For single-window applications, just match if it's the only window
    // This will be handled by the caller

    false
}

/// Get the integer window number from an AXUIElement (DEPRECATED - AXWindowNumber doesn't exist)
fn get_ax_window_number(window: AXUIElementRef) -> Result<i32, String> {
    debug!(
        "get_ax_window_number: Starting for window at address {:p}",
        window
    );
    debug!("WARNING: AXWindowNumber attribute does not exist on AX windows!");
    debug!("This function is deprecated and will always fail.");

    unsafe {
        let attr_name = create_cfstring(AX_WINDOW_NUMBER_ATTRIBUTE);
        let mut value: CFTypeRef = ptr::null_mut();

        debug!(
            "get_ax_window_number: Calling AXUIElementCopyAttributeValue for attribute '{}'",
            AX_WINDOW_NUMBER_ATTRIBUTE
        );
        let result = AXUIElementCopyAttributeValue(window, attr_name, &mut value);
        CFRelease(attr_name as CFTypeRef);

        debug!(
            "get_ax_window_number: AXUIElementCopyAttributeValue result: {}",
            result
        );
        if result != kAXErrorSuccess {
            let error_msg = format!("AXUIElementCopyAttributeValue failed with error: {} (EXPECTED - AXWindowNumber doesn't exist)", result);
            debug!("get_ax_window_number: {}", error_msg);
            return Err(error_msg);
        }

        // This code should never be reached since AXWindowNumber doesn't exist
        debug!("get_ax_window_number: Unexpected success - this shouldn't happen!");
        if !value.is_null() {
            CFRelease(value);
        }

        Err("AXWindowNumber attribute does not exist".to_string())
    }
}

// =============================================================================
// CORE WINDOW SWITCHING IMPLEMENTATION (following C example)
// =============================================================================

/// Get all visible layer-0 window numbers in front-to-back order
fn get_visible_window_numbers() -> Result<Vec<i32>, String> {
    debug!("Getting visible window numbers...");
    unsafe {
        let window_list = CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
            0, // kCGNullWindowID
        );

        if window_list.is_null() {
            debug!("Failed to get window list from CGWindowListCopyWindowInfo");
            return Err("Failed to get window list".to_string());
        }

        let window_count = CFArrayGetCount(window_list);
        debug!("Found {} total windows in list", window_count);
        let mut window_numbers = Vec::new();

        for i in 0..window_count {
            let window_dict_ref = CFArrayGetValueAtIndex(window_list, i);
            if !window_dict_ref.is_null() {
                let window_dict = window_dict_ref as CFDictionaryRef;

                // Print full dictionary for debugging
                print_full_dictionary(window_dict);

                // Get window info for debugging
                let window_number = get_dict_number(window_dict, CG_WINDOW_NUMBER);
                let layer = get_dict_number(window_dict, CG_WINDOW_LAYER);
                let pid = get_dict_number(window_dict, CG_WINDOW_OWNER_PID);
                let app_name = get_dict_string(window_dict, CG_WINDOW_OWNER_NAME);
                let window_name = get_dict_string(window_dict, CG_WINDOW_NAME);
                let bounds = get_dict_bounds(window_dict, CG_WINDOW_BOUNDS);

                debug!(
                    "Window {}: ID={}, Layer={}, PID={}, App='{}', Title='{}', Bounds={:?}",
                    i, window_number, layer, pid, app_name, window_name, bounds
                );

                // Only layer 0 windows
                if layer != 0 {
                    debug!("Skipping window {} (layer {})", window_number, layer);
                    continue;
                }

                // Extract the window number
                window_numbers.push(window_number);
                debug!("Added layer-0 window {} to list", window_number);
            }
        }

        CFRelease(window_list as CFTypeRef);

        debug!("Final visible window numbers: {:?}", window_numbers);

        if window_numbers.is_empty() {
            debug!("No visible layer-0 windows found");
            return Err("No visible windows found".to_string());
        }

        Ok(window_numbers)
    }
}

/// Get the currently focused window number using Accessibility APIs
fn get_focused_window_number() -> Result<i32, String> {
    debug!("Getting focused window number...");
    unsafe {
        let sys_wide = AXUIElementCreateSystemWide();
        if sys_wide.is_null() {
            debug!("Failed to create system-wide element");
            return Err("Failed to create system-wide element".to_string());
        }

        // DEBUG: Print all properties of sys_wide
        debug!("========== sys_wide element properties ==========");
        debug!("sys_wide element address: {:p}", sys_wide);

        // First check accessibility permission status
        let accessibility_check = check_accessibility_permission();
        match &accessibility_check {
            Ok(_) => debug!("Accessibility permission check: GRANTED"),
            Err(e) => debug!("Accessibility permission check: FAILED - {}", e),
        }

        // Get all available attributes for sys_wide
        let mut attribute_names: CFTypeRef = ptr::null_mut();
        let attr_names_attr = create_cfstring("AXAttributeNames");
        let attr_result =
            AXUIElementCopyAttributeValue(sys_wide, attr_names_attr, &mut attribute_names);
        CFRelease(attr_names_attr as CFTypeRef);

        debug!(
            "AXUIElementCopyAttributeValue for AXAttributeNames result: {}",
            attr_result
        );

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
        debug!("Error code {} means: {}", attr_result, error_description);

        if attr_result == kAXErrorSuccess && !attribute_names.is_null() {
            let attr_count = CFArrayGetCount(attribute_names as CFArrayRef);
            debug!("DEBUG: sys_wide has {} attributes:", attr_count);

            for j in 0..attr_count {
                let attr_name_ref = CFArrayGetValueAtIndex(attribute_names as CFArrayRef, j);
                if !attr_name_ref.is_null() {
                    let attr_name = cfstring_to_string(attr_name_ref as CFStringRef);
                    debug!("DEBUG:   - {}", attr_name);

                    // Try to get the value of each attribute for debugging
                    let attr_cfstr = create_cfstring(&attr_name);
                    let mut attr_value: CFTypeRef = ptr::null_mut();
                    let value_result =
                        AXUIElementCopyAttributeValue(sys_wide, attr_cfstr, &mut attr_value);
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

                        debug!("DEBUG:     Value TypeID: {} ({})", type_id, type_name);

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
                        debug!(
                            "DEBUG:     Failed to get value (result: {} - {})",
                            value_result, value_error_desc
                        );
                    }
                }
            }
            CFRelease(attribute_names);
        } else {
            debug!("DEBUG: sys_wide: failed to get attribute names");
            if attr_result == -25205 {
                debug!("DEBUG: This is kAXErrorAPIDisabled - Accessibility API is disabled!");
                debug!("DEBUG: The application needs accessibility permissions to work.");
                debug!("DEBUG: Go to System Preferences > Security & Privacy > Privacy > Accessibility");
                debug!("DEBUG: and make sure your application is listed and enabled.");
            }
        }

        // Try to directly access the focused application attribute anyway
        debug!("DEBUG: Attempting direct access to AXFocusedApplication attribute...");
        let focused_app_attr = create_cfstring(AX_FOCUSED_APPLICATION_ATTRIBUTE);
        let mut focused_app_test: CFTypeRef = ptr::null_mut();
        let direct_result =
            AXUIElementCopyAttributeValue(sys_wide, focused_app_attr, &mut focused_app_test);
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
        debug!(
            "DEBUG: Direct AXFocusedApplication access result: {} ({})",
            direct_result, direct_error_desc
        );

        if direct_result == kAXErrorSuccess && !focused_app_test.is_null() {
            debug!("DEBUG: Successfully got focused application directly!");
            CFRelease(focused_app_test);
        }

        debug!("DEBUG: ========== End sys_wide properties ==========");

        // Get focused application
        let focused_app_attr = create_cfstring(AX_FOCUSED_APPLICATION_ATTRIBUTE);
        let mut focused_app: CFTypeRef = ptr::null_mut();
        let result = AXUIElementCopyAttributeValue(sys_wide, focused_app_attr, &mut focused_app);
        CFRelease(sys_wide as CFTypeRef);
        CFRelease(focused_app_attr as CFTypeRef);

        if result != kAXErrorSuccess || focused_app.is_null() {
            debug!("DEBUG: Cannot get focused application (result: {})", result);
            return Err("Cannot get focused application".to_string());
        }

        debug!("DEBUG: Successfully got focused application");

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
            debug!("DEBUG: Cannot get focused window (result: {})", result);
            CFRelease(focused_app);
            return Err("Cannot get focused window".to_string());
        }

        debug!("DEBUG: Successfully got focused window");

        // Get window number
        let window_number = get_ax_window_number(focused_win as AXUIElementRef);

        CFRelease(focused_win);
        CFRelease(focused_app);

        match &window_number {
            Ok(num) => debug!("DEBUG: Focused window number: {}", num),
            Err(e) => debug!("DEBUG: Failed to get window number: {}", e),
        }

        window_number
    }
}

/// Focus a window by its window number using Accessibility APIs
fn focus_window_by_number(target_window_number: i32) -> Result<(), String> {
    debug!("DEBUG: Attempting to focus window {}", target_window_number);
    unsafe {
        // First, find the PID that owns this window
        debug!(
            "DEBUG: Looking up window info for window {}",
            target_window_number
        );
        let window_list = CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
            0,
        );
        if window_list.is_null() {
            debug!(
                "DEBUG: Failed to get window info for window {}",
                target_window_number
            );
            return Err("Failed to get window info".to_string());
        }

        let window_count = CFArrayGetCount(window_list);
        debug!(
            "DEBUG: Found {} window entries for window {}",
            window_count, target_window_number
        );
        if window_count == 0 {
            CFRelease(window_list as CFTypeRef);
            debug!(
                "DEBUG: No window entries found for window {}",
                target_window_number
            );
            return Err(format!("Cannot find window {}", target_window_number));
        }

        let mut window_dict: CFDictionaryRef = ptr::null();
        for i in 0..window_count {
            let w = CFArrayGetValueAtIndex(window_list, i);
            debug!("DEBUG: Checking window entry {} at address {:p}", i, w);
            if w == ptr::null() || w.is_null() {
                debug!("DEBUG: Skipping null window entry");
                continue;
            }

            debug!("DEBUG: Window entry {} is valid, checking contents", i);
            let window_number = get_dict_number(w as CFDictionaryRef, CG_WINDOW_NUMBER);
            debug!(
                "DEBUG: Window entry {} has window number {}",
                i, window_number
            );

            if window_number != target_window_number {
                debug!(
                    "DEBUG: Skipping window entry {} (not target window {})",
                    i, target_window_number
                );
                continue;
            }

            debug!(
                "DEBUG: Found matching window {} at index {}",
                target_window_number, i
            );
            window_dict = w as CFDictionaryRef;

            // let layer = get_dict_number(window_dict, CG_WINDOW_LAYER);
            // let pid = get_dict_number(window_dict, CG_WINDOW_OWNER_PID);
            // let app_name = get_dict_string(window_dict, CG_WINDOW_OWNER_NAME);
            // let window_name = get_dict_string(window_dict, CG_WINDOW_NAME);
            // debug!("DEBUG: Window {} (PID: {}) - App: '{}', Title: '{}'",
            //     window_number, pid, app_name, window_name);
        }

        if window_dict.is_null() {
            CFRelease(window_list as CFTypeRef);
            debug!(
                "DEBUG: No valid window dictionary found for window {}",
                target_window_number
            );
            return Err(format!(
                "Cannot find valid window dictionary for window {}",
                target_window_number
            ));
        }

        //let window_dict = CFArrayGetValueAtIndex(window_list, 0) as CFDictionaryRef;
        let target_pid = get_dict_number(window_dict, CG_WINDOW_OWNER_PID);
        let app_name = get_dict_string(window_dict, CG_WINDOW_OWNER_NAME);
        let window_name = get_dict_string(window_dict, CG_WINDOW_NAME);
        let target_cg_bounds = get_dict_bounds(window_dict, CG_WINDOW_BOUNDS);
        debug!(
            "DEBUG: Window {} belongs to PID {} (app: '{}', title: '{}')",
            target_window_number, target_pid, app_name, window_name
        );
        CFRelease(window_list as CFTypeRef);

        if target_pid == 0 {
            debug!("DEBUG: Invalid PID (0) for window {}", target_window_number);
            return Err(format!(
                "Cannot determine PID for window {}",
                target_window_number
            ));
        }

        // Create the app element and find the AX window element with matching window number
        debug!("DEBUG: Creating application element for PID {}", target_pid);
        let app_element = AXUIElementCreateApplication(target_pid);
        if app_element.is_null() {
            debug!(
                "DEBUG: Failed to create application element for PID {}",
                target_pid
            );
            return Err("Failed to create application element".to_string());
        }

        debug!("DEBUG: Getting windows list from application element");
        let windows_attr = create_cfstring(AX_WINDOWS_ATTRIBUTE);
        let mut app_windows: CFTypeRef = ptr::null_mut();
        let result = AXUIElementCopyAttributeValue(app_element, windows_attr, &mut app_windows);
        CFRelease(windows_attr as CFTypeRef);

        if result != kAXErrorSuccess || app_windows.is_null() {
            debug!("DEBUG: Cannot get application windows (result: {})", result);
            return Err("Cannot get application windows".to_string());
        }

        let window_count = CFArrayGetCount(app_windows as CFArrayRef);
        debug!("DEBUG: Application has {} AX windows", window_count);
        let mut target_window: AXUIElementRef = ptr::null_mut();

        // Strategy 1: If there's only one AX window, use it (common case)
        if window_count == 1 {
            target_window = CFArrayGetValueAtIndex(app_windows as CFArrayRef, 0) as AXUIElementRef;
            debug!("DEBUG: Only one AX window available, using it directly");
        } else {
            // Strategy 2: Try to match using title and bounds
            debug!("DEBUG: Multiple AX windows found, attempting to match by title/bounds");

            // Set the window title
            let title_attr = create_cfstring(AX_TITLE_ATTRIBUTE);
            let tmp_title = create_cfstring("TEST JJK");
            let set_title_result =
                AXUIElementSetAttributeValue(app_element, title_attr, tmp_title as CFTypeRef);
            CFRelease(tmp_title as CFTypeRef);
            debug!("DEBUG: Set window title result: {}", set_title_result);

            for i in 0..window_count {
                let window = CFArrayGetValueAtIndex(app_windows as CFArrayRef, i) as AXUIElementRef;
                debug!("DEBUG: Examining AX window {} at address {:p}", i, window);

                // Let's try to get all available attributes for this window
                let mut attribute_names: CFArrayRef = ptr::null_mut();
                let attr_result = AXUIElementCopyAttributeNames(window, &mut attribute_names);

                if attr_result == kAXErrorSuccess && !attribute_names.is_null() {
                    let attr_count = CFArrayGetCount(attribute_names as CFArrayRef);
                    debug!("DEBUG: AX window {} has {} attributes:", i, attr_count);
                    for j in 0..attr_count {
                        let attr_name_ref =
                            CFArrayGetValueAtIndex(attribute_names as CFArrayRef, j);
                        if !attr_name_ref.is_null() {
                            let attr_name = cfstring_to_string(attr_name_ref as CFStringRef);
                            debug!("DEBUG:   - {}", attr_name);
                        }
                    }
                    CFRelease(attribute_names as CFTypeRef);
                } else {
                    debug!(
                        "DEBUG: AX window {}: failed to get attribute names (result: {})",
                        i, attr_result
                    );
                }

                // Try to match this AX window to our target CG window
                if match_ax_window_to_cg_window(window, &window_name, target_cg_bounds, i as usize)
                {
                    debug!("DEBUG: Found matching AX window at index {}", i);
                    target_window = window;
                    break;
                }
            }

            // Strategy 3: If no match found, use the first window as fallback
            if target_window.is_null() {
                debug!("DEBUG: No exact match found, using first AX window as fallback");
                target_window =
                    CFArrayGetValueAtIndex(app_windows as CFArrayRef, 0) as AXUIElementRef;
            }
        }

        if target_window.is_null() {
            debug!(
                "DEBUG: Could not get any AX window from {} available windows",
                window_count
            );
            CFRelease(app_windows);
            return Err(format!("Couldn't get any AX window for application"));
        }

        // Focus the window using Accessibility APIs
        debug!(
            "DEBUG: Performing AXRaise action on window {}",
            target_window_number
        );
        let raise_action = create_cfstring(AX_RAISE_ACTION);
        let raise_result = AXUIElementPerformAction(target_window, raise_action);
        CFRelease(raise_action as CFTypeRef);
        debug!("DEBUG: AXRaise result: {}", raise_result);

        debug!("DEBUG: Setting focused window attribute");
        let focused_win_attr = create_cfstring(AX_FOCUSED_WINDOW_ATTRIBUTE);
        let focus_result =
            AXUIElementSetAttributeValue(app_element, focused_win_attr, target_window as CFTypeRef);
        CFRelease(focused_win_attr as CFTypeRef);
        debug!("DEBUG: Set focused window result: {}", focus_result);

        CFRelease(app_windows);

        debug!(
            "DEBUG: Successfully focused window {}",
            target_window_number
        );
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
            Ok(windows
                .into_iter()
                .find(|w| w.window_id == window_number as u32))
        }
        Err(_) => Ok(None),
    }
}

/// Focus a specific window (for compatibility)
// pub fn focus_window(window: &WindowInfo) -> Result<(), String> {
//     focus_window_by_number(window.window_id as i32)
// }

/// Cycle to the next window using the proper accessibility-based approach
pub fn cycle_next_window() -> Result<(), String> {
    debug!("DEBUG: ========== Starting window cycling ==========");

    // Check accessibility permission first
    debug!("DEBUG: Checking accessibility permission...");
    check_accessibility_permission()?;
    debug!("DEBUG: Accessibility permission OK");

    // Get all visible layer-0 window numbers in front-to-back order
    let windows = get_current_workspace_windows()?;

    if windows.is_empty() {
        debug!("DEBUG: No visible windows found, aborting");
        return Err("No visible windows found".to_string());
    }

    debug!("DEBUG: Found {} visible windows", windows.len());

    if windows.len() == 1 {
        // Only one window, focus it
        debug!(
            "DEBUG: Only one window available, doing nothing: {}, {}",
            windows[0].app_name, windows[0].title
        );
        return Ok(());
    }

    let result = focus_window_by_number(windows[1].window_id as i32);
    match &result {
        Ok(_) => debug!("DEBUG: ========== Window cycling completed successfully =========="),
        Err(e) => debug!("DEBUG: ========== Window cycling failed: {} ==========", e),
    }

    result

    // // Get the current focused window number
    // let current_window_number = match get_focused_window_number() {
    //     Ok(num) => {
    //         debug!("DEBUG: Current focused window: {}", num);
    //         num
    //     }
    //     Err(e) => {
    //         debug!(
    //             "DEBUG: Could not get focused window ({}), focusing first available: {}",
    //             e, windows[0].window_id
    //         );
    //         // If we can't get the focused window, just focus the first one
    //         return focus_window_by_number(window_numbers[0]);
    //     }
    // };

    // // Find the current window in our list and pick the next one
    // let current_index = window_numbers
    //     .iter()
    //     .position(|&num| num == current_window_number);
    // let next_index = match current_index {
    //     Some(idx) => {
    //         let next = (idx + 1) % window_numbers.len();
    //         debug!(
    //             "DEBUG: Current window {} is at index {}, next index: {}",
    //             current_window_number, idx, next
    //         );
    //         next
    //     }
    //     None => {
    //         debug!(
    //             "DEBUG: Current window {} not found in visible list, starting from beginning",
    //             current_window_number
    //         );
    //         0 // Current window not found, start from beginning
    //     }
    // };

    // let target_window_number = window_numbers[next_index];
    // debug!("DEBUG: Target window to focus: {}", target_window_number);

    // let result = focus_window_by_number(target_window_number);
    // match &result {
    //     Ok(_) => debug!("DEBUG: ========== Window cycling completed successfully =========="),
    //     Err(e) => debug!("DEBUG: ========== Window cycling failed: {} ==========", e),
    // }

    // result
}
