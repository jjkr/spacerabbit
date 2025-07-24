use crate::window_manager::get_current_workspace_windows;
use core_foundation::array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
use core_foundation::base::{CFRelease, CFTypeRef};
use core_foundation::dictionary::CFDictionaryRef;
use core_foundation::number::{kCFNumberSInt64Type, CFNumberGetValue, CFNumberRef};
use core_foundation::string::{kCFStringEncodingUTF8, CFStringRef};
use core_graphics::display::{CGDirectDisplayID, CGDisplayPixelsHigh, CGDisplayPixelsWide};
use core_graphics::event::{CGEvent, CGEventTapLocation};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::{CGPoint, CGRect};
use foreign_types_shared::ForeignType;
use log::{debug, error, info, warn};
use std::thread;
use std::time::Duration;

// =============================================================================
// CORE GRAPHICS PRIVATE API BINDINGS
// =============================================================================

type CGSConnectionID = i32;
type CGEventRef = *mut std::ffi::c_void;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CFDictionaryGetValue(
        theDict: CFDictionaryRef,
        key: *const std::ffi::c_void,
    ) -> *const std::ffi::c_void;
    fn CFStringCreateWithCString(
        alloc: *const std::ffi::c_void,
        cStr: *const std::ffi::c_char,
        encoding: u32,
    ) -> CFStringRef;

    fn CGSMainConnectionID() -> CGSConnectionID;
    fn CGSCopyManagedDisplaySpaces(cid: CGSConnectionID) -> CFArrayRef;
    fn CGEventSetIntegerValueField(event: CGEventRef, field: u32, value: i64);
    fn CGEventSetDoubleValueField(event: CGEventRef, field: u32, value: f64);

    fn CGGetDisplaysWithPoint(
        point: CGPoint,
        max_displays: u32,
        displays: *mut u32,
        display_count: *mut u32,
    ) -> i32;
    fn CGMainDisplayID() -> u32;
    fn CGSGetActiveSpace(cid: CGSConnectionID, display_id: u32) -> u64;
    fn CGDisplayBounds(display: u32) -> CGRect;

    // Mouse event functions
    fn CGEventCreateMouseEvent(
        source: *const std::ffi::c_void,
        mouseType: u32,
        mouseCursorPosition: CGPoint,
        mouseButton: u32,
    ) -> CGEventRef;
    fn CGEventPost(tap: u32, event: CGEventRef);
    fn CGWarpMouseCursorPosition(newCursorPosition: CGPoint) -> i32;

    // Cursor position functions
    fn CGEventCreate(source: *const std::ffi::c_void) -> CGEventRef;
    fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
}

// =============================================================================
// CONSTANTS AND TYPES
// =============================================================================

const TOP_EDGE_OFFSET: f64 = 20.0; // Pixels from top edge to trigger desktop thumbnails

const kCGEventMouseMoved: u32 = 5;
const kCGMouseButtonLeft: u32 = 0;
const kCGHIDEventTap: u32 = 0;

// Type unions for float/int bit pattern conversion
#[repr(C)]
union FloatBits {
    float_value: f32,
    int_bits: i32,
}

#[repr(C)]
union DoubleBits {
    double_value: f64,
    int_bits: i64,
}

// =============================================================================
// DISPLAY AND WORKSPACE DETECTION
// =============================================================================

#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub display_id: u32,
    pub bounds: CGRect,
    pub current_desktop: u32,
}

/// Get current mouse cursor position in global coordinates
pub fn get_cursor_position() -> Result<CGPoint, String> {
    unsafe {
        // Create a “dummy” event (no specific type)
        let e = CGEventCreate(std::ptr::null());
        // Ask it where the cursor is right now
        let loc = CGEventGetLocation(e);
        CFRelease(e);
        //let mouse_location = NSEvent::mouseLocation(nil);
        //let cursor_point = CGPoint::new(mouse_location.x as f64, mouse_location.y as f64);
        Ok(loc)
    }
}

/// Find which display contains the given point
pub fn find_display_at_point(point: CGPoint) -> Result<u32, String> {
    unsafe {
        let mut found_display_id: u32 = 0;
        let mut display_count: u32 = 0;

        let result = CGGetDisplaysWithPoint(point, 1, &mut found_display_id, &mut display_count);

        if result == 0 && display_count > 0 {
            Ok(found_display_id)
        } else {
            // Fallback to main display if point lookup fails
            Ok(CGMainDisplayID())
        }
    }
}

/// Get properly ordered desktop spaces using CGSCopyManagedDisplaySpaces
pub fn get_ordered_desktop_spaces(display_id: u32) -> Result<u32, String> {
    unsafe {
        let connection_id = CGSMainConnectionID();
        let current_space_id = CGSGetActiveSpace(connection_id, display_id);

        // Use CGSCopyManagedDisplaySpaces to get properly ordered spaces
        let managed_spaces = CGSCopyManagedDisplaySpaces(connection_id);
        if managed_spaces.is_null() {
            return Err("CGSCopyManagedDisplaySpaces returned null".to_string());
        }

        let count = CFArrayGetCount(managed_spaces);

        // Find the entry for our specific display
        for i in 0..count {
            let entry_ref = CFArrayGetValueAtIndex(managed_spaces, i);
            if entry_ref.is_null() {
                continue;
            }

            // Use raw CFDictionary access
            let entry_dict = entry_ref as CFDictionaryRef;

            // // First, check if this entry is for our target display
            // let display_key_cstr = std::ffi::CString::new("Display").unwrap();
            // let display_key_cfstr = CFStringCreateWithCString(
            //     std::ptr::null(),
            //     display_key_cstr.as_ptr(),
            //     kCFStringEncodingUTF8
            // );

            // let display_value_ref = CFDictionaryGetValue(entry_dict, display_key_cfstr as *const std::ffi::c_void);
            // CFRelease(display_key_cfstr as CFTypeRef);

            // if !display_value_ref.is_null() {
            //     let mut entry_display_id: i64 = 0;
            //     let success = CFNumberGetValue(
            //         display_value_ref as CFNumberRef,
            //         kCFNumberSInt64Type,
            //         &mut entry_display_id as *mut i64 as *mut std::ffi::c_void
            //     );

            //     // Only process this entry if it matches our target display
            //     if success && entry_display_id as u32 == display_id {

            // Create "Spaces" key string
            let spaces_key_cstr = std::ffi::CString::new("Spaces").unwrap();
            let spaces_key_cfstr = CFStringCreateWithCString(
                std::ptr::null(),
                spaces_key_cstr.as_ptr(),
                kCFStringEncodingUTF8,
            );

            // Try to get the spaces array from the dictionary
            let spaces_array_ref =
                CFDictionaryGetValue(entry_dict, spaces_key_cfstr as *const std::ffi::c_void);
            CFRelease(spaces_key_cfstr as CFTypeRef);

            if !spaces_array_ref.is_null() {
                let spaces_array = spaces_array_ref as CFArrayRef;
                if spaces_array.is_null() {
                    continue;
                }

                let spaces_count = CFArrayGetCount(spaces_array);
                if spaces_count <= 0 {
                    continue;
                }

                // Process all spaces to find current space
                for j in 0..spaces_count {
                    let space_ref = CFArrayGetValueAtIndex(spaces_array, j);
                    if !space_ref.is_null() {
                        // Parse space dictionary to get ManagedSpaceID
                        let space_dict = space_ref as CFDictionaryRef;

                        let space_id_key_cstr = std::ffi::CString::new("ManagedSpaceID").unwrap();
                        let space_id_key_cfstr = CFStringCreateWithCString(
                            std::ptr::null(),
                            space_id_key_cstr.as_ptr(),
                            kCFStringEncodingUTF8,
                        );

                        let space_id_value_ref = CFDictionaryGetValue(
                            space_dict,
                            space_id_key_cfstr as *const std::ffi::c_void,
                        );
                        CFRelease(space_id_key_cfstr as CFTypeRef);

                        if !space_id_value_ref.is_null() {
                            let mut space_id_value: i64 = 0;
                            let success = CFNumberGetValue(
                                space_id_value_ref as CFNumberRef,
                                kCFNumberSInt64Type,
                                &mut space_id_value as *mut i64 as *mut std::ffi::c_void,
                            );

                            if success
                                && space_id_value > 0
                                && space_id_value as u64 == current_space_id
                            {
                                let desktop_num = j + 1; // 1-based desktop numbers
                                CFRelease(managed_spaces as CFTypeRef);
                                return Ok(desktop_num as u32);
                            }
                        }
                    }
                }
            }

            //     }
            // }
        }

        CFRelease(managed_spaces as CFTypeRef);
        Err("Could not find current space in any display's ordered spaces".to_string())
    }
}

/// Get current desktop number for a specific display
pub fn get_desktop_for_display(display_id: u32) -> Result<u32, String> {
    get_ordered_desktop_spaces(display_id)
}

/// Get display info for the display containing the cursor
pub fn get_cursor_display_info() -> Result<DisplayInfo, String> {
    let cursor_pos = get_cursor_position()?;
    let display_id = find_display_at_point(cursor_pos)?;
    let current_desktop = get_desktop_for_display(display_id)?;
    let display_bounds = unsafe { CGDisplayBounds(display_id) };

    Ok(DisplayInfo {
        display_id,
        bounds: display_bounds,
        current_desktop,
    })
}

/// Get current desktop number for the display containing the cursor
pub fn get_current_desktop() -> Result<u32, String> {
    let display_info = get_cursor_display_info()?;
    Ok(display_info.current_desktop)
}

/// Count total number of desktops/spaces for a specific display
pub fn count_desktops_for_display(display_id: u32) -> Result<u32, String> {
    unsafe {
        let connection_id = CGSMainConnectionID();
        let managed_spaces = CGSCopyManagedDisplaySpaces(connection_id);

        if managed_spaces.is_null() {
            return Err("Failed to get managed display spaces".to_string());
        }

        let display_count = CFArrayGetCount(managed_spaces);

        // Search through display entries to find our specific display
        for i in 0..display_count {
            let entry_ref = CFArrayGetValueAtIndex(managed_spaces, i);
            if entry_ref.is_null() {
                debug!("Skipping null entry at index {}", i);
                continue;
            }

            let entry_dict = entry_ref as CFDictionaryRef;
            debug!("Processing display entry {}", i);

            // First, check if this entry is for our target display
            // let display_key_cstr = std::ffi::CString::new("Display").unwrap();
            // let display_key_cfstr = CFStringCreateWithCString(
            //     std::ptr::null(),
            //     display_key_cstr.as_ptr(),
            //     kCFStringEncodingUTF8
            // );

            // let display_value_ref = CFDictionaryGetValue(entry_dict, display_key_cfstr as *const std::ffi::c_void);
            // CFRelease(display_key_cfstr as CFTypeRef);
            // if display_value_ref.is_null() {
            //     println!("Skipping entry at index {}: no display value", i);
            //     continue;
            // }

            // if !display_value_ref.is_null() {
            //     let mut entry_display_id: i64 = 0;
            //     let success = CFNumberGetValue(
            //         display_value_ref as CFNumberRef,
            //         kCFNumberSInt64Type,
            //         &mut entry_display_id as *mut i64 as *mut std::ffi::c_void
            //     );

            //     // Only process this entry if it matches our target display
            //     if success && entry_display_id as u32 == display_id {

            // Create "Spaces" key for dictionary lookup
            let spaces_key_cstr = std::ffi::CString::new("Spaces").unwrap();
            let spaces_key_cfstr = CFStringCreateWithCString(
                std::ptr::null(),
                spaces_key_cstr.as_ptr(),
                kCFStringEncodingUTF8,
            );

            // Get spaces array from dictionary
            let spaces_array_ref =
                CFDictionaryGetValue(entry_dict, spaces_key_cfstr as *const std::ffi::c_void);
            CFRelease(spaces_key_cfstr as CFTypeRef);

            if !spaces_array_ref.is_null() {
                let spaces_array = spaces_array_ref as CFArrayRef;
                if !spaces_array.is_null() {
                    let spaces_count = CFArrayGetCount(spaces_array);
                    if spaces_count > 0 {
                        CFRelease(managed_spaces as CFTypeRef);
                        return Ok(spaces_count as u32);
                    }
                }
            }
            //}
            //}
        }

        CFRelease(managed_spaces as CFTypeRef);
        Err("No spaces found for display".to_string())
    }
}

/// Count total desktops for the display containing the cursor
pub fn count_total_desktops() -> Result<u32, String> {
    let cursor_pos = get_cursor_position()?;
    let display_id = find_display_at_point(cursor_pos)?;
    count_desktops_for_display(display_id)
}

/// Get current and total desktop counts for cursor's display
pub fn get_desktop_bounds() -> Result<(u32, u32), String> {
    let cursor_pos = get_cursor_position()?;
    debug!("Cursor position: {:?}", cursor_pos);
    let display_id = find_display_at_point(cursor_pos)?;
    debug!("Display ID for cursor: {}", display_id);
    let current_desktop = get_desktop_for_display(display_id)?;
    debug!(
        "Current desktop for display {}: {}",
        display_id, current_desktop
    );
    let total_desktops = count_desktops_for_display(display_id)?;
    debug!(
        "Total desktops for display {}: {}",
        display_id, total_desktops
    );
    Ok((current_desktop, total_desktops))
}

// =============================================================================
// MOUSE MOVEMENT FUNCTIONS
// =============================================================================

/// Create and post a synthetic mouse move event at the specified point
///
/// This function creates a CGEvent for mouse movement, posts it to the system,
/// and properly releases the event memory. Used to trigger desktop thumbnails
/// and focus windows in Mission Control.
///
/// # Arguments
/// * `point` - The target position for the mouse move event
///
/// # Returns
/// * `Ok(())` on success, or an error message if the event could not be created or posted
fn post_mouse_move_event(point: CGPoint) -> Result<(), String> {
    unsafe {
        let move_evt: CGEventRef = CGEventCreateMouseEvent(
            std::ptr::null(), // No event source, use system default
            kCGEventMouseMoved,
            point,
            kCGMouseButtonLeft,
        );
        
        if move_evt.is_null() {
            return Err("Failed to create mouse move event".to_string());
        }
        
        CGEventPost(kCGHIDEventTap, move_evt);
        CFRelease(move_evt);
    }
    Ok(())
}

/// Move mouse cursor to specified coordinates using CGWarpMouseCursorPosition
///
/// This function provides instant mouse teleportation to the target position.
/// Uses the most direct Core Graphics API for cursor positioning.
///
/// # Arguments
/// * `x` - Target X coordinate in global screen coordinates
/// * `y` - Target Y coordinate in global screen coordinates
///
/// # Returns
/// * `Ok(())` on success, or an error message if the operation failed
pub fn move_mouse_to_position(x: f64, y: f64) -> Result<(), String> {
    let target_point = CGPoint::new(x, y);

    unsafe {
        let result = CGWarpMouseCursorPosition(target_point);
        if result == 0 {
            Ok(())
        } else {
            Err(format!(
                "Failed to move mouse cursor: CGWarpMouseCursorPosition returned {}",
                result
            ))
        }
    }
}

/// Restore mouse cursor to its original position
///
/// # Arguments
/// * `original_position` - The position to restore the cursor to
///
/// # Returns
/// * `Ok(())` on success, or an error message if the operation failed
pub fn restore_mouse_position(original_position: CGPoint) -> Result<(), String> {
    move_mouse_to_position(original_position.x, original_position.y)
}

/// Create and send a synthetic gesture event for workspace switching
///
/// Generates CGEvents that mimic a 3-finger horizontal swipe gesture.
/// This bypasses the need for actual touch input by directly posting the
/// essential gesture event fields that macOS recognizes for workspace switching.
///
/// # Arguments:
/// * `event_source` - The CGEventSource to use for creating the event.
/// * `gesture_phase` - The phase of the gesture (1 for begin, 2 for update, 4 for end/snap).
/// * `gesture_type` - The type of the gesture (e.g., 1 for horizontal swipe, 2 for vertical swipe).
/// * `positive_direction` - Whether the swipe is in the positive direction (right/up) or negative (left/down).
/// # Returns:
/// * `Ok(())` on success, or an error message if the event could not be created or posted.
////// # Notes:
/// This function uses private CGEvent fields to simulate the gesture.
fn send_gesture_event(
    event_source: &CGEventSource,
    gesture_phase: i64,
    gesture_type: i64,
    positive_direction: bool,
) -> Result<(), String> {
    // Create gesture phase event and tracking event
    let phase_event =
        CGEvent::new(event_source.clone()).map_err(|_| "Failed to create phase event")?;

    // Calculate movement values
    let delta_sign = if positive_direction { 1.0 } else { -1.0 };

    unsafe {
        // === PHASE EVENT: Gesture Phase Transition ===

        // Field 0x37 (55): Event type - 0x1e (30) for gesture phase transition
        CGEventSetIntegerValueField(phase_event.as_ptr() as CGEventRef, 0x37, 0x1e);
        // Field 0x6e (110): Gesture subtype - 0x17 (23) for horizontal swipe gesture
        CGEventSetIntegerValueField(phase_event.as_ptr() as CGEventRef, 0x6e, 0x17);
        // Field 0x84 (132): Gesture phase - 1 for begin, 2 for update, 4 for end/snap to workspace
        CGEventSetIntegerValueField(phase_event.as_ptr() as CGEventRef, 0x84, gesture_phase);
        // Field 0x86 (134): Mirror of gesture phase
        CGEventSetIntegerValueField(phase_event.as_ptr() as CGEventRef, 0x86, gesture_phase);
        // Field 0x8a (138): 3 fingers gesture - 0x3 for 3-finger swipe
        CGEventSetIntegerValueField(phase_event.as_ptr() as CGEventRef, 0x8a, 0x03);

        // Gesture state flags
        // Field 0x7b (123): Gesture active flag - 1 indicates gesture is active
        CGEventSetIntegerValueField(phase_event.as_ptr() as CGEventRef, 0x7b, gesture_type);
        // Field 0xa5 (165): Gesture type - 1 for horizontal swipe, 2 for vertical swipe
        CGEventSetIntegerValueField(phase_event.as_ptr() as CGEventRef, 0xa5, gesture_type);

        // Position data (only during snap phase)
        if gesture_phase == 4 {
            // Field 0x7c (124): Movement delta as double
            CGEventSetDoubleValueField(phase_event.as_ptr() as CGEventRef, 0x7c, 1.0 * delta_sign);
            // Field 0x81 (129): Final position, set to a large number for fast transition
            CGEventSetDoubleValueField(
                phase_event.as_ptr() as CGEventRef,
                0x81,
                9100.0 * delta_sign,
            );
            // Field 0x82 (130): Final position copy
            CGEventSetDoubleValueField(
                phase_event.as_ptr() as CGEventRef,
                0x82,
                9100.0 * delta_sign,
            );
        } else {
            // Field 0x7c (124): Movement delta as double
            CGEventSetDoubleValueField(
                phase_event.as_ptr() as CGEventRef,
                0x7c,
                0.000001 * delta_sign,
            );
        }
    }

    // Post event to system
    phase_event.post(CGEventTapLocation::HID);

    Ok(())
}

pub fn activate_mission_control() -> Result<(), String> {
    let event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| "Failed to create CGEventSource")?;

    // Get current cursor position to restore later
    let original_position = get_cursor_position()?;

    // Move mouse to a position near the top edge to trigger desktop thumbnails
    move_mouse_to_position(50 as f64, TOP_EDGE_OFFSET)?;
    // Sleep to allow system to register the position change
    thread::sleep(Duration::from_millis(5));

    // Move to 60 with an event that will trigger desktop thumbnails
    let top_point = CGPoint::new(60.0, TOP_EDGE_OFFSET);
    post_mouse_move_event(top_point)?;

    thread::sleep(Duration::from_millis(5));

    // Simulate a 3-finger swipe up gesture to activate Mission Control
    send_gesture_event(&event_source, 1, 2, true)?;
    send_gesture_event(&event_source, 2, 2, true)?;
    send_gesture_event(&event_source, 4, 2, true)?;

    let top_point = CGPoint::new(70.0, TOP_EDGE_OFFSET);
    post_mouse_move_event(top_point)?;

    // Wait for desktop thumbnails to appear
    thread::sleep(Duration::from_millis(100));
    let windows = get_current_workspace_windows()?;
    if windows.is_empty() {
        // Restore original mouse position if the desktop is empty
        debug!("No windows found on current workspace, restoring mouse position");
        restore_mouse_position(original_position)?;
    } else {
        // Get the center point of the first window
        if let Some(first_window) = windows.first() {
            let window_rect = first_window.bounds;
            let center_x = window_rect.0 + window_rect.2 / 2.0;
            let center_y = window_rect.1 + window_rect.3 / 2.0 + 20.0; // Shift down so the pointer sits below the label
                                                                       // Move mouse to the center of the first window
                                                                       //move_mouse_to_position(center_x, center_y)?;
            let window_center = CGPoint::new(center_x, center_y);
            post_mouse_move_event(window_center)?;
        }

        // Print all the windows found in the current workspace
        for window in windows {
            info!("Window: {:?}", window);
        }
    }

    Ok(())
}

/// Activate Mission Control with desktop thumbnails (same as activate_mission_control)
///
/// This is an alias for the enhanced activate_mission_control function that includes
/// automatic mouse movement to trigger desktop thumbnails.
pub fn activate_mission_control_with_thumbnails() -> Result<(), String> {
    activate_mission_control()
}

/// Activate Mission Control without desktop thumbnails (original behavior)
///
/// This function provides the original Mission Control activation behavior
/// without the mouse movement to trigger desktop thumbnails.
pub fn activate_mission_control_basic() -> Result<(), String> {
    let event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| "Failed to create CGEventSource")?;

    // Simulate a 3-finger swipe up gesture to activate Mission Control
    send_gesture_event(&event_source, 1, 2, true)?;
    send_gesture_event(&event_source, 2, 2, true)?;
    send_gesture_event(&event_source, 4, 2, true)?;

    Ok(())
}

pub fn deactivate_mission_control() -> Result<(), String> {
    let event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| "Failed to create CGEventSource")?;

    // Simulate a 3-finger swipe down gesture to deactivate Mission Control
    send_gesture_event(&event_source, 1, 2, false)?;
    send_gesture_event(&event_source, 2, 2, false)?;
    send_gesture_event(&event_source, 4, 2, false)?;

    Ok(())
}

// =============================================================================
// PUBLIC WORKSPACE SWITCHING API
// =============================================================================

/// Switch to adjacent macOS workspace using synthetic gesture simulation
///
/// Simulates a 3-finger horizontal swipe by posting synthetic CGEvents.
/// 1. Begin gesture (phase 1) - initiates workspace transition animation
/// 2. Update gesture (phase 2) - updates the gesture position
/// 3. End gesture (phase 4) - completes transition and snaps to target workspace
pub fn switch_to_adjacent_workspace(move_right: bool) -> Result<(), String> {
    let event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| "Failed to create CGEventSource")?;

    send_gesture_event(&event_source, 1, 1, move_right)?;
    send_gesture_event(&event_source, 2, 1, move_right)?;
    send_gesture_event(&event_source, 4, 1, move_right)?;

    Ok(())
}

/// Switch to the workspace on the left (with bounds checking)
pub fn switch_workspace_left() -> Result<(), String> {
    let (current_desktop, _total_desktops) = get_desktop_bounds()?;

    if current_desktop <= 1 {
        return Err("Already at the leftmost workspace".to_string());
    }

    switch_to_adjacent_workspace(false)
}

/// Switch to the workspace on the right (with bounds checking)
pub fn switch_workspace_right() -> Result<(), String> {
    let (current_desktop, total_desktops) = get_desktop_bounds()?;

    if current_desktop >= total_desktops {
        return Err("Already at the rightmost workspace".to_string());
    }

    switch_to_adjacent_workspace(true)
}

/// Move mouse to the next window in mission control
///
/// This function:
/// 1. Gets the current mouse position
/// 2. Gets all visible windows on the current workspace
/// 3. Determines which window the mouse is currently in (or closest to)
/// 4. Moves the mouse to the center of the next window in the array
/// 5. Cycles back to the beginning when reaching the end
pub fn move_mouse_to_next_window() -> Result<(), String> {
    debug!("Starting move_mouse_to_next_window");
    
    // Get current mouse position
    let current_mouse_pos = get_cursor_position()?;
    debug!("Current mouse position: {:?}", current_mouse_pos);
    
    // Get all visible windows on current workspace
    let windows = get_current_workspace_windows()?;
    debug!("Found {} windows on current workspace", windows.len());
    
    if windows.is_empty() {
        return Err("No windows found on current workspace".to_string());
    }
    
    if windows.len() == 1 {
        // Only one window, move mouse to its center
        let window = &windows[0];
        let center_x = window.bounds.0 + window.bounds.2 / 2.0;
        let center_y = window.bounds.1 + window.bounds.3 / 2.0;
        debug!("Only one window, moving to center: ({}, {})", center_x, center_y);
        return move_mouse_to_position(center_x, center_y);
    }
    
    // Print all windows found
    for window in &windows {
        debug!("Window: {:?}", window);
    }
    
    // Find which window the mouse is currently in or closest to
    let current_window_index = find_current_window_index(&windows, current_mouse_pos);
    debug!("Current window index: {:?}", current_window_index);
    
    // Calculate next window index (cycle back to 0 if at end)
    let mut next_index = match current_window_index {
        Some(index) => (index + 1) % windows.len(),
        None => 0, // If mouse isn't in any window, start with first window
    };
    
    debug!("Next window index: {}", next_index);
    if windows[next_index].app_name == "Dock" {
        debug!("Next window is Dock, skipping to next");
        // Skip Dock window if it's next
        next_index = (next_index + 1) % windows.len();
        debug!("Adjusted next index to: {}", next_index);
    }
    
    // Move mouse to center of next window
    let next_window = &windows[next_index];
    let center_x = next_window.bounds.0 + next_window.bounds.2 / 2.0;
    let center_y = next_window.bounds.1 + next_window.bounds.3 / 2.0;
    
    debug!("Moving mouse to window({:?}): {:?} at ({}, {})", 
           next_index, next_window, center_x, center_y);
    
    post_mouse_move_event(CGPoint::new(center_x, center_y))
}

/// Find which window contains the mouse cursor or is closest to it
///
/// Returns the index of the window that contains the mouse cursor.
/// If the mouse is not inside any window, returns the index of the closest window.
/// If no windows are provided, returns None.
fn find_current_window_index(windows: &[crate::window_manager::WindowInfo], mouse_pos: CGPoint) -> Option<usize> {
    if windows.is_empty() {
        return None;
    }
    
    // First, check if mouse is inside any window
    for (index, window) in windows.iter().enumerate() {
        if is_point_in_window(mouse_pos, &window.bounds) {
            debug!("Mouse is inside window {} ('{}')", index, window.title);
            return Some(index);
        }
    }
    
    // If mouse is not inside any window, find the closest one
    let mut closest_index = 0;
    let mut closest_distance = f64::MAX;
    
    for (index, window) in windows.iter().enumerate() {
        let distance = distance_to_window(mouse_pos, &window.bounds);
        debug!("Distance to window {} ('{}'): {}", index, window.title, distance);
        
        if distance < closest_distance {
            closest_distance = distance;
            closest_index = index;
        }
    }
    
    debug!("Mouse is closest to window {} ('{}')", closest_index, windows[closest_index].title);
    Some(closest_index)
}

/// Check if a point is inside a window's bounds
fn is_point_in_window(point: CGPoint, bounds: &(f64, f64, f64, f64)) -> bool {
    let (x, y, width, height) = *bounds;
    point.x >= x && point.x <= x + width && point.y >= y && point.y <= y + height
}

/// Calculate the distance from a point to the nearest edge of a window
fn distance_to_window(point: CGPoint, bounds: &(f64, f64, f64, f64)) -> f64 {
    let (x, y, width, height) = *bounds;
    
    // Calculate distance to window center as a simple metric
    let center_x = x + width / 2.0;
    let center_y = y + height / 2.0;
    
    let dx = point.x - center_x;
    let dy = point.y - center_y;
    
    (dx * dx + dy * dy).sqrt()
}

// Legacy function aliases for backward compatibility
pub fn switch_left() -> Result<(), String> {
    switch_workspace_left()
}

pub fn switch_right() -> Result<(), String> {
    switch_workspace_right()
}
