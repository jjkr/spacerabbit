# CGEvent Analysis: macOS Workspace Switching and Mission Control

This document analyzes the Core Graphics Events (CGEvents) generated during macOS workspace switching and Mission Control gestures, based on captured event logs from a three-finger swipe gesture sniffer.

## Overview

macOS uses Core Graphics Events to handle trackpad gestures for workspace switching and Mission Control. These events contain specific field patterns that can be used to programmatically detect and respond to these gestures.

## Event Structure

CGEvents contain numbered fields that store different types of data:
- **Integer Fields**: Discrete values for event types, flags, and identifiers
- **Double Fields**: Floating-point values for coordinates, deltas, and timing
- **Key Fields**: Field numbers that identify specific gesture types and states

## Workspace Switching Events (Three-Finger Left Swipe)

### Event Pattern
Workspace switching generates alternating sequences of **Event Type 29** and **Event Type 30**:

```
Event Type 29 → Event Type 30 → Event Type 29 → Event Type 30 → ...
```

### Critical Fields

| Field | Type | Purpose | Values |
|-------|------|---------|---------|
| **Field 55** | Integer | Event type identifier | 29 (gesture update), 30 (gesture data) |
| **Field 110** | Integer | **Gesture type identifier** | **23 = Workspace switching** |
| **Field 132** | Integer | Gesture phase | 1 (begin), 2 (progress), 4 (end) |
| **Field 134** | Integer | Gesture phase (duplicate) | 1 (begin), 2 (progress), 4 (end) |
| **Field 124** | Double | **X-axis movement delta** | **Negative = left swipe** |
| **Field 125** | Double | Y-axis movement delta | Usually small values |
| **Field 126** | Double | Z-axis/pressure delta | Small values |
| **Field 138** | Integer | Touch count | 3 (three fingers) |

### Direction Detection
- **Left swipe** (next workspace): Field 124 has **negative values** (-0.011673 to -0.268814)
- **Right swipe** (previous workspace): Field 124 would have **positive values**

### Gesture Phases
1. **Begin (Field 132 = 1)**: Initial touch detection
2. **Progress (Field 132 = 2)**: Continuous movement tracking
3. **End (Field 132 = 4)**: Gesture completion and workspace switch

### Sample Event Sequence

```
Event Type: 30, Field 110: 23, Field 132: 1, Field 124: -0.011673  // Begin
Event Type: 30, Field 110: 23, Field 132: 2, Field 124: -0.020355  // Progress
Event Type: 30, Field 110: 23, Field 132: 2, Field 124: -0.023010  // Progress
...
Event Type: 30, Field 110: 23, Field 132: 4, Field 124: -0.268814  // End
```

### Additional Context Fields

| Field | Purpose | Typical Values |
|-------|---------|----------------|
| Field 40 | Source PID | 3449 (WindowServer or similar) |
| Field 53 | Touch count | 3 |
| Field 135 | Timestamp/sequence | Incrementing values |
| Field 169 | High-precision timestamp | Large incrementing values |

## Mission Control Events (Three-Finger Right Swipe)

### Key Differences from Workspace Switching

| Aspect | Workspace Switch | Mission Control |
|--------|------------------|-----------------|
| **Gesture ID (Field 110)** | 23 | **32** |
| **Direction (Field 124)** | Negative (left) | **Positive (right)** |
| **Touch Detection** | 4 fingers detected | **8 fingers detected** |
| **End State** | Direct workspace change | **UI overlay activation** |
| **Event Types** | 29→30 alternating | **29→30 + UI events (14,1,2)** |

### Mission Control Specific Fields

| Field | Purpose | Mission Control Values |
|-------|---------|----------------------|
| **Field 110** | Gesture type | **32** (Mission Control) |
| **Field 144** | Mission Control mode | **5** (consistent) |
| **Field 143** | Mission Control state | **1** (active) |
| **Field 115/117** | Touch count | **8** (higher detection) |

### Mission Control Event Sequence

```
Event Type: 29, Field 110: 32, Field 144: 5, Field 124: 0.012192   // Begin right swipe
Event Type: 30, Field 110: 32, Field 144: 5, Field 124: 0.024200   // Progress
...
Event Type: 30, Field 110: 32, Field 144: 5, Field 124: 0.255478   // End
Event Type: 14, Field 83: 7, Field 99: 7                           // UI transition
Event Type: 1, Field 89: 45, Field 90: 3                           // UI state
Event Type: 2, Field 89: 45, Field 90: 3                           // UI finalization
```

## Implementation Guidelines

### For Workspace Switching Detection

```c
// Pseudo-code for workspace switching detection
if (event.field_55 == 30 &&           // Gesture data event
    event.field_110 == 23 &&          // Workspace switching gesture
    event.field_132 == 4) {           // Gesture completion
    
    if (event.field_124 < 0) {
        // Left swipe - next workspace
        switch_to_next_workspace();
    } else if (event.field_124 > 0) {
        // Right swipe - previous workspace  
        switch_to_previous_workspace();
    }
}
```

### Required Accessibility Permissions

To monitor these events, your application needs:
- **Accessibility permissions** in System Preferences → Security & Privacy → Privacy → Accessibility
- **CGEventTap** with appropriate event mask
- **Event types**: `kCGEventOtherMouseDown`, `kCGEventOtherMouseUp`, or custom event types 29/30

### Event Tap Setup

```c
CGEventMask eventMask = (1 << 29) | (1 << 30);  // Custom gesture events
CFMachPortRef eventTap = CGEventTapCreate(
    kCGSessionEventTap,
    kCGHeadInsertEventTap,
    kCGEventTapOptionDefault,
    eventMask,
    eventCallback,
    NULL
);
```

## Field Reference

### Common Fields Across All Events

| Field | Type | Description |
|-------|------|-------------|
| 39 | Integer | Event source identifier |
| 40 | Integer | Source process PID |
| 45 | Integer | Event flags |
| 50 | Integer | Additional flags |
| 53 | Integer | Touch/finger count |
| 55 | Integer | **Event type** (29/30 for gestures) |
| 58 | Integer | Event timestamp |
| 85 | Integer | Source identifier |
| 87 | Integer | Extended source info |
| 101 | Integer | Display/screen identifier |
| 107 | Integer | Additional display info |
| 169 | Integer | High-precision timestamp |

### Gesture-Specific Fields

| Field | Type | Description |
|-------|------|-------------|
| 110 | Integer | **Gesture type** (23=workspace, 32=mission control) |
| 115/117 | Integer | Touch count (detailed) |
| 119/120 | Double | Touch coordinates |
| 123 | Integer | Gesture timestamp |
| 124 | Double | **X-axis delta** (direction) |
| 125 | Double | Y-axis delta |
| 126 | Double | Z-axis/pressure delta |
| 132 | Integer | **Gesture phase** (1=begin, 2=progress, 4=end) |
| 134 | Integer | Gesture phase (duplicate) |
| 135 | Integer | Sequence/timestamp |
| 138 | Integer | Touch count |
| 139/140 | Double | Additional coordinates |
| 143 | Integer | Mission Control state flag |
| 144 | Integer | Mission Control mode |
| 164 | Integer | Extended touch info |
| 165 | Integer | Additional gesture data |

## Event Source Analysis

### Source PIDs Observed
- **PID 3449**: Primary workspace switching events (likely WindowServer)
- **PID 1777**: Mission Control and system UI events

### Event Timing
- Events fire at ~8ms intervals during active gestures
- High-precision timestamps in Field 169 provide microsecond accuracy
- Field 58 contains standard CGEvent timestamps

## Debugging and Monitoring

### Event Sniffer Implementation
The analysis was performed using a custom CGEvent sniffer that:
1. Creates an event tap for all event types
2. Filters for events with specific field patterns
3. Dumps all integer and double fields for analysis
4. Identifies potential workspace switching patterns

### Key Detection Patterns
- **Workspace switching**: Field 110=23 with negative Field 124
- **Mission Control**: Field 110=32 with positive Field 124
- **Gesture completion**: Field 132=4 indicates action trigger point
- **? fields**: Events with Fields 41=0, 55=29/30, 110≠0

## Conclusion

macOS workspace switching and Mission Control use distinct CGEvent patterns that can be reliably detected through Core Graphics event monitoring. The key differentiators are:

1. **Field 110** for gesture type identification
2. **Field 124** for direction detection  
3. **Field 132** for gesture phase tracking
4. **Event Type 30** for actionable gesture data

This information enables programmatic workspace switching without relying on private APIs, using only the public Core Graphics Events framework with appropriate accessibility permissions.
