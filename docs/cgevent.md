# Core Graphics/Quartz Events: Format and Field Analysis

## Overview

Core Graphics events (also known as Quartz events) are low-level hardware events in macOS that represent user input from devices like mice, keyboards, and trackpads. These events flow through the system from hardware drivers through the window server to applications.

## Event Structure

### Basic Event Properties
Every CGEvent has these fundamental properties:
- **Event Type** (`CGEventType`): Identifies the kind of event (mouse down, key press, etc.)
- **Timestamp** (`CGEventTimestamp`): When the event occurred
- **Location** (`CGPoint`): Mouse cursor position in global coordinates
- **Flags** (`CGEventFlags`): Modifier key states and other event flags
- **Source Information**: Process ID and other metadata about the event source

### Event Fields System

CGEvents use a field-based system where data is stored in numbered fields accessible via:
- `CGEventGetIntegerValueField(event, field_number)` - for 64-bit integer values
- `CGEventGetDoubleValueField(event, field_number)` - for floating-point values
- `CGEventSetIntegerValueField(event, field_number, value)` - to set integer values
- `CGEventSetDoubleValueField(event, field_number, value)` - to set double values

## Documented Field Constants

Apple provides official constants for common fields in `CGEventField` enum:

### Mouse Event Fields
- **Field 0** (`kCGMouseEventNumber`): Mouse button event number
- **Field 1** (`kCGMouseEventClickState`): Click state (1=single, 2=double, 3=triple)
- **Field 2** (`kCGMouseEventPressure`): Mouse button pressure (0.0-1.0)
- **Field 3** (`kCGMouseEventButtonNumber`): Which mouse button
- **Field 4** (`kCGMouseEventDeltaX`): Horizontal mouse movement delta
- **Field 5** (`kCGMouseEventDeltaY`): Vertical mouse movement delta
- **Field 6** (`kCGMouseEventInstantMouser`): Inkwell subsystem flag
- **Field 7** (`kCGMouseEventSubtype`): Mouse event subtype

### Keyboard Event Fields
- **Field 8** (`kCGKeyboardEventAutorepeat`): Auto-repeat flag
- **Field 9** (`kCGKeyboardEventKeycode`): Virtual key code
- **Field 10** (`kCGKeyboardEventKeyboardType`): Keyboard type identifier

### Scroll Wheel Event Fields
- **Field 11** (`kCGScrollWheelEventDeltaAxis1`): Vertical scroll delta
- **Field 12** (`kCGScrollWheelEventDeltaAxis2`): Horizontal scroll delta
- **Field 13** (`kCGScrollWheelEventDeltaAxis3`): Third axis (unused)
- **Field 14-16**: Fixed-point scroll deltas
- **Field 17-19**: Point-based scroll deltas
- **Field 20** (`kCGScrollWheelEventInstantMouser`): Inkwell flag

### Tablet Event Fields
- **Fields 21-39**: Various tablet pen properties (position, pressure, tilt, rotation, etc.)

### Process/Source Fields
- **Field 40** (`kCGEventTargetProcessSerialNumber`): Target process serial number
- **Field 41** (`kCGEventTargetUnixProcessID`): Target process ID
- **Field 42** (`kCGEventSourceUnixProcessID`): Source process ID
- **Field 43** (`kCGEventSourceUserData`): User-supplied data (64-bit)
- **Field 44** (`kCGEventSourceUserID`): Source effective UID
- **Field 45** (`kCGEventSourceGroupID`): Source effective GID
- **Field 46** (`kCGEventSourceStateID`): Event source state ID

## Undocumented Fields (Reverse Engineered)

From analyzing the code samples, several undocumented fields have been discovered:

### Gesture/Workspace Switching Fields
Based on the Rust workspace switcher code:
- **Field 0x29 (41)**: Magic space identifier (often `0x81cf` = 33231)
- **Field 0x37 (55)**: Gesture event type (29, 30 for workspace switching)
- **Field 0x6e (110)**: Animation control (23 for workspace animations)
- **Field 0x77 (119)**: Magic constant for gestures
- **Field 0x7b (123)**: Gesture enable flag
- **Field 0x7c (124)**: Swipe delta (double)
- **Field 0x81 (129)**: Position field 1 (double)
- **Field 0x82 (130)**: Position field 2 (double)
- **Field 0x84 (132)**: Gesture value
- **Field 0x86 (134)**: Gesture value copy
- **Field 0x87 (135)**: Delta as 32-bit float pattern
- **Field 0x88 (136)**: Additional gesture control
- **Field 0x8b (139)**: Another magic constant
- **Field 0xa5 (165)**: Secondary enable flag

### BetterTouchTool Detection Patterns
The cgevent_sniff.c code identifies BetterTouchTool events by looking for:
- Field 55 values of 29 or 30
- Field 110 value of 23
- Field 123 and 165 both set to 1
- Field 41 containing `0x81cf` (33231)

## Event Types

Common `CGEventType` values:
- `kCGEventNull` (0): Null event
- `kCGEventLeftMouseDown` (1): Left mouse button pressed
- `kCGEventLeftMouseUp` (2): Left mouse button released
- `kCGEventRightMouseDown` (3): Right mouse button pressed
- `kCGEventRightMouseUp` (4): Right mouse button released
- `kCGEventMouseMoved` (5): Mouse moved
- `kCGEventLeftMouseDragged` (6): Left mouse dragged
- `kCGEventRightMouseDragged` (7): Right mouse dragged
- `kCGEventKeyDown` (10): Key pressed
- `kCGEventKeyUp` (11): Key released
- `kCGEventFlagsChanged` (12): Modifier keys changed
- `kCGEventScrollWheel` (22): Scroll wheel moved
- `kCGEventTabletPointer` (23): Tablet pointer event
- `kCGEventTabletProximity` (24): Tablet proximity event
- `kCGEventOtherMouseDown` (25): Other mouse button pressed
- `kCGEventOtherMouseUp` (26): Other mouse button released
- `kCGEventOtherMouseDragged` (27): Other mouse button dragged

## Magic Constants and Patterns

### Workspace Switching Magic Values
- **0x81cf (33231)**: Space identifier magic number
- **0x36a0000000000000**: Magic constant for gesture events (as 64-bit double)
- **29, 30**: Event type values for workspace switching
- **23**: Animation control value
- **1**: Enable flags for gesture processing

### Data Encoding
- Some fields store 32-bit float values as 64-bit integers (bit pattern preservation)
- Fixed-point 16.16 format used for precise scroll values
- Double values often contain magic constants for gesture recognition

## Event Flow and Processing

1. **Hardware Input**: Device driver creates low-level event
2. **I/O Kit**: Processes and queues event
3. **Window Server**: Creates CGEvent, adds annotations
4. **Event Taps**: Can intercept and modify events at various points
5. **Application Delivery**: Event reaches target application
6. **Framework Processing**: Carbon Event Manager or Cocoa processes event

## Security and Permissions

- Event taps require accessibility permissions
- Some event types (key events) need special authorization
- Process isolation prevents unauthorized event injection
- Root privileges required for system-level event taps

This analysis reveals that while Apple documents the basic event structure and common fields, many advanced features like gesture recognition and workspace switching rely on undocumented field patterns that have been reverse-engineered by developers.