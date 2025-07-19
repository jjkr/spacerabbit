use core_graphics::event::{CGEvent, CGEventTapLocation};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::{CGPoint, CGRect};
use core_foundation::array::{CFArrayRef, CFArrayGetCount, CFArrayGetValueAtIndex};
use core_foundation::base::{CFTypeRef, CFRelease};
use core_foundation::number::{CFNumberRef, CFNumberGetValue, kCFNumberSInt64Type};
use core_foundation::dictionary::CFDictionaryRef;
use core_foundation::string::{CFStringRef, kCFStringEncodingUTF8};
use foreign_types_shared::ForeignType;
use cocoa::appkit::NSEvent;
use cocoa::base::nil;
use std::thread;
use std::time::Duration;

// =============================================================================
// CORE GRAPHICS PRIVATE API BINDINGS
// =============================================================================

type CGSConnectionID = i32;
type CGEventRef = *mut std::ffi::c_void;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CFDictionaryGetValue(theDict: CFDictionaryRef, key: *const std::ffi::c_void) -> *const std::ffi::c_void;
    fn CFStringCreateWithCString(
        alloc: *const std::ffi::c_void,
        cStr: *const std::ffi::c_char,
        encoding: u32
    ) -> CFStringRef;

    fn CGSMainConnectionID() -> CGSConnectionID;
    fn CGSCopyManagedDisplaySpaces(cid: CGSConnectionID) -> CFArrayRef;
    fn CGEventSetIntegerValueField(event: CGEventRef, field: u32, value: i64);
    fn CGEventSetDoubleValueField(event: CGEventRef, field: u32, value: f64);

    fn CGGetDisplaysWithPoint(point: CGPoint, max_displays: u32, displays: *mut u32, display_count: *mut u32) -> i32;
    fn CGMainDisplayID() -> u32;
    fn CGSGetActiveSpace(cid: CGSConnectionID, display_id: u32) -> u64;
    fn CGDisplayBounds(display: u32) -> CGRect;
}

// =============================================================================
// CONSTANTS AND TYPES
// =============================================================================

// Gesture timing and movement
const SWIPE_MOVEMENT_DELTA: f64 = 3.0;
const GESTURE_PHASE_DELAY_MICROS: u64 = 200;
const POSITION_SCALE_FACTOR: f64 = 400.0;

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
        let mouse_location = NSEvent::mouseLocation(nil);
        let cursor_point = CGPoint::new(mouse_location.x as f64, mouse_location.y as f64);
        Ok(cursor_point)
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

        // Find the entry for our display (usually there's just one for the main display)
        for i in 0..count {
            let entry_ref = CFArrayGetValueAtIndex(managed_spaces, i);
            if entry_ref.is_null() {
                continue;
            }

            // Use raw CFDictionary access
            let entry_dict = entry_ref as CFDictionaryRef;

            // Create "Spaces" key string
            let spaces_key_cstr = std::ffi::CString::new("Spaces").unwrap();
            let spaces_key_cfstr = CFStringCreateWithCString(
                std::ptr::null(),
                spaces_key_cstr.as_ptr(),
                kCFStringEncodingUTF8
            );

            // Try to get the spaces array from the dictionary
            let spaces_array_ref = CFDictionaryGetValue(entry_dict, spaces_key_cfstr as *const std::ffi::c_void);
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
                            kCFStringEncodingUTF8
                        );

                        let space_id_value_ref = CFDictionaryGetValue(space_dict, space_id_key_cfstr as *const std::ffi::c_void);
                        CFRelease(space_id_key_cfstr as CFTypeRef);

                        if !space_id_value_ref.is_null() {
                            let mut space_id_value: i64 = 0;
                            let success = CFNumberGetValue(
                                space_id_value_ref as CFNumberRef,
                                kCFNumberSInt64Type,
                                &mut space_id_value as *mut i64 as *mut std::ffi::c_void
                            );

                            if success && space_id_value > 0 && space_id_value as u64 == current_space_id {
                                let desktop_num = j + 1; // 1-based desktop numbers
                                CFRelease(managed_spaces as CFTypeRef);
                                return Ok(desktop_num as u32);
                            }
                        }
                    }
                }
            }
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

        // Search through display entries to find our display
        for i in 0..display_count {
            let entry_ref = CFArrayGetValueAtIndex(managed_spaces, i);
            if entry_ref.is_null() {
                continue;
            }

            let entry_dict = entry_ref as CFDictionaryRef;

            // Create "Spaces" key for dictionary lookup
            let spaces_key_cstr = std::ffi::CString::new("Spaces").unwrap();
            let spaces_key_cfstr = CFStringCreateWithCString(
                std::ptr::null(),
                spaces_key_cstr.as_ptr(),
                kCFStringEncodingUTF8
            );

            // Get spaces array from dictionary
            let spaces_array_ref = CFDictionaryGetValue(entry_dict, spaces_key_cfstr as *const std::ffi::c_void);
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
    let display_id = find_display_at_point(cursor_pos)?;
    let current_desktop = get_desktop_for_display(display_id)?;
    let total_desktops = count_desktops_for_display(display_id)?;
    Ok((current_desktop, total_desktops))
}


// =============================================================================
// SYNTHETIC GESTURE GENERATION
// =============================================================================

/// Create and post synthetic gesture events for workspace switching
/// 
/// Generates CGEvents that mimic a 3-finger horizontal swipe gesture.
/// This bypasses the need for actual touch input by directly posting the 
/// essential gesture event fields that macOS recognizes for workspace switching.
fn create_synthetic_gesture(
    event_source: &CGEventSource,
    gesture_phase: i64,
    swipe_direction_right: bool,
    start_event: bool
) -> Result<(), String> {
    // Create gesture phase event and tracking event
    let phase_event = CGEvent::new(event_source.clone())
        .map_err(|_| "Failed to create phase event")?;
    let tracking_event = CGEvent::new(event_source.clone())
        .map_err(|_| "Failed to create tracking event")?;

    // Calculate movement values
    let delta_sign = if swipe_direction_right { 1.0 } else { -1.0 };

    // Extract magic constant from bit pattern - 0x36a0000000000000 required for gesture recognition
    // let magic_constant = unsafe {
    //     let magic_bits = DoubleBits { int_bits: 0x36a0000000000000 };
    //     magic_bits.double_value
    // };

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

        // Movement data
        // Field 0x7c (124): X-axis movement delta as double
        if gesture_phase == 4 {
            CGEventSetDoubleValueField(phase_event.as_ptr() as CGEventRef, 0x7c, 1.0 * delta_sign);
        } else {
            CGEventSetDoubleValueField(phase_event.as_ptr() as CGEventRef, 0x7c, 0.000001 * delta_sign);
        }
        //CGEventSetDoubleValueField(phase_event.as_ptr() as CGEventRef, 0x7c, 0.0);
        //CGEventSetDoubleValueField(phase_event.as_ptr() as CGEventRef, 0x7c, 1.0 * delta_sign);
        //CGEventSetDoubleValueField(phase_event.as_ptr() as CGEventRef, 0x7c, movement_delta);
        // Field 0x7d (125): Y-axis movement delta?? - 0 for horizontal swipe
        //CGEventSetDoubleValueField(phase_event.as_ptr() as CGEventRef, 0x7d, 0.01);
        // Field 0x7e (126): Z-axis/pressure delta?? - 0 for horizontal swipe
        //CGEventSetDoubleValueField(phase_event.as_ptr() as CGEventRef, 0x7e, -0.01);
        
        //let movement_as_float_bits = {
        //    let float_bits = FloatBits { float_value: movement_delta as f32 };
        //    float_bits.int_bits as i64
        //};
        //// Field 0x87 (135): X-axis movement as float bit pattern
        //CGEventSetIntegerValueField(phase_event.as_ptr() as CGEventRef, 0x87, movement_as_float_bits);

        // Required magic constants
        // Field 0x77 (119): Magic constant - 0x36a0000000000000 required for gesture recognition
        CGEventSetDoubleValueField(phase_event.as_ptr() as CGEventRef, 0x77, 0.0);
        
        // Field 0x8b (139): Magic constant mirror
        CGEventSetDoubleValueField(phase_event.as_ptr() as CGEventRef, 0x8b, 0.0);

        // Gesture state flags
        // Field 0x7b (123): Gesture active flag - 1 indicates gesture is active
        CGEventSetIntegerValueField(phase_event.as_ptr() as CGEventRef, 0x7b, 1);
        
        // Field 0xa5 (165): Gesture state flag - 1 indicates gesture state is active
        CGEventSetIntegerValueField(phase_event.as_ptr() as CGEventRef, 0xa5, 1);
        
        // Field 0x29 (41): Event flags - 0x81cf standard gesture event flags
        //CGEventSetIntegerValueField(phase_event.as_ptr() as CGEventRef, 0x29, 0x81cf);
        
        // Field 0x88 (136): Touch count - 0 for synthetic gesture
        //CGEventSetIntegerValueField(phase_event.as_ptr() as CGEventRef, 0x88, 0);

        // Position data (only during snap phase)
        if gesture_phase == 4 {
            //let cumulative_position = scaled_movement * 4.0;
            // Field 0x81 (129): Final X position for workspace snap
            CGEventSetDoubleValueField(phase_event.as_ptr() as CGEventRef, 0x81, 41000.0 * delta_sign);
            
            // Field 0x82 (130): Final X position copy
            CGEventSetDoubleValueField(phase_event.as_ptr() as CGEventRef, 0x82, 41000.0 * delta_sign);
        }

        // === TRACKING EVENT: Gesture Tracking ===
        
        // Field 0x37 (55): Event type - 0x1d (29) for continuous gesture tracking
        //CGEventSetIntegerValueField(tracking_event.as_ptr() as CGEventRef, 0x37, 0x1d);
        
        // Field 0x29 (41): Event flags - 0x81cf standard gesture event flags
        //CGEventSetIntegerValueField(tracking_event.as_ptr() as CGEventRef, 0x29, 0x81cf);
    }

    // Post events to system
    phase_event.post(CGEventTapLocation::HID);
    //if start_event {
    //    // DEBUG: skip
    //    return Ok(());
    //}
    //tracking_event.post(CGEventTapLocation::HID);

    Ok(())
}

// =============================================================================
// PUBLIC WORKSPACE SWITCHING API
// =============================================================================

/// Switch to adjacent macOS workspace using synthetic gesture simulation
/// 
/// Simulates a 3-finger horizontal swipe by posting synthetic CGEvents.
/// Uses a two-phase approach that mimics real gesture behavior:
/// 1. Begin gesture (phase 1) - initiates workspace transition animation
/// 2. End gesture (phase 4) - completes transition and snaps to target workspace
pub fn switch_to_adjacent_workspace(move_right: bool) -> Result<(), String> {
    let event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| "Failed to create CGEventSource")?;

    // Phase 1: Begin gesture
    create_synthetic_gesture(&event_source, 1, move_right, false)?;

    // Brief delay between phases (mimics natural gesture timing)
    //thread::sleep(Duration::from_micros(GESTURE_PHASE_DELAY_MICROS));

    // Phase 2: Gesture update
    create_synthetic_gesture(&event_source, 2, move_right, true)?;

    // Brief delay between phases (mimics natural gesture timing)
    //thread::sleep(Duration::from_micros(GESTURE_PHASE_DELAY_MICROS));

    // Phase 2: End gesture
    create_synthetic_gesture(&event_source, 4, move_right, true)?;

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

// Legacy function aliases for backward compatibility
pub fn switch_left() -> Result<(), String> {
    switch_workspace_left()
}

pub fn switch_right() -> Result<(), String> {
    switch_workspace_right()
}

pub fn switch_to_adjacent_space(to_right: bool) -> Result<(), String> {
    switch_to_adjacent_workspace(to_right)
}

pub fn get_current_context_desktop() -> Result<u32, String> {
    get_current_desktop()
}
