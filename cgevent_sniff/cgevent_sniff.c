#include <ApplicationServices/ApplicationServices.h>
#include <stdio.h>
#include <stdlib.h>
#include <signal.h>
#include <unistd.h>

// Global variables for cleanup
CFMachPortRef eventTap = NULL;
CFRunLoopSourceRef runLoopSource = NULL;

// Function to dump all fields of a CGEvent
void dump_all_event_fields(CGEventRef event) {
    printf("=== CGEvent Field Dump ===\n");
    
    // Check common documented fields first
    printf("Event Type: %d\n", (int)CGEventGetType(event));
    printf("Timestamp: %llu\n", CGEventGetTimestamp(event));
    printf("Flags: 0x%llx\n", (unsigned long long)CGEventGetFlags(event));
    
    // Dump integer fields (0-200 range)
    printf("\n--- Integer Fields ---\n");
    for (int i = 0; i < 200; i++) {
        int64_t value = CGEventGetIntegerValueField(event, i);
        if (value != 0) {
            printf("Field %d (0x%x): %lld (0x%llx)\n", i, i, value, value);
        }
    }
    
    // Dump double fields (these are less common)
    printf("\n--- Double Fields ---\n");
    for (int i = 0; i < 200; i++) {
        double value = CGEventGetDoubleValueField(event, i);
        if (value != 0.0) {
            printf("Double Field %d (0x%x): %f\n", i, i, value);
        }
    }
    
    printf("========================\n\n");
}

// Function to check if this looks like a space switching event
bool is_likely_space_switch_event(CGEventRef event) {
    // Check for common field patterns that might indicate space switching
    int64_t field_41 = CGEventGetIntegerValueField(event, 41);   // Space ID
    int64_t field_55 = CGEventGetIntegerValueField(event, 55);   // Event type
    int64_t field_110 = CGEventGetIntegerValueField(event, 110); // Animation control
    int64_t field_123 = CGEventGetIntegerValueField(event, 123); // Enable flag
    int64_t field_165 = CGEventGetIntegerValueField(event, 165); // Another enable flag
    
    // Look for specific patterns that might indicate space switching
    if (field_55 == 29 || field_55 == 30) {
        return true;
    }
    
    if (field_110 == 23) {
        return true;
    }
    
    if (field_123 == 1 && field_165 == 1) {
        return true;
    }
    
    // Check for specific space ID patterns
    if (field_41 == 0x81cf || field_41 == 33231) {
        return true;
    }
    
    return false;
}

// Event tap callback function
CGEventRef eventTapCallback(CGEventTapProxy proxy, CGEventType type, CGEventRef event, void *userInfo) {
    // Get the source process
    pid_t sourcePID = CGEventGetIntegerValueField(event, kCGEventTargetUnixProcessID);
    
    // Check if this is a space switch event
    bool is_interesting = false;
    
    // Check for events that match space switching patterns
    if (is_likely_space_switch_event(event)) {
        is_interesting = true;
        printf("🎯 POTENTIAL SPACE SWITCH EVENT DETECTED!\n");
        printf("Source PID: %d\n", sourcePID);
    }
    
    // Also log any event with non-zero values in specific fields
    int64_t field_41 = CGEventGetIntegerValueField(event, 41);
    int64_t field_55 = CGEventGetIntegerValueField(event, 55);
    int64_t field_110 = CGEventGetIntegerValueField(event, 110);
    
    if (field_41 != 0 || field_55 != 0 || field_110 != 0) {
        is_interesting = true;
        printf("🔍 Event with interesting fields detected!\n");
        printf("Source PID: %d, Field 41: %lld, Field 55: %lld, Field 110: %lld\n", 
               sourcePID, field_41, field_55, field_110);
    }
    
    // Dump full details for interesting events
    //if (is_interesting) {
        dump_all_event_fields(event);
    //}
    
    // Return the event unmodified (we're just monitoring)
    return event;
}

// Signal handler for clean shutdown
void signal_handler(int sig) {
    printf("\nShutting down event sniffer...\n");
    
    if (eventTap) {
        CGEventTapEnable(eventTap, false);
        CFRelease(eventTap);
    }
    
    if (runLoopSource) {
        CFRunLoopRemoveSource(CFRunLoopGetCurrent(), runLoopSource, kCFRunLoopCommonModes);
        CFRelease(runLoopSource);
    }
    
    exit(0);
}

int main(int argc, char *argv[]) {
    printf("=== CGEvent Sniffer ===\n");
    printf("This program will monitor all CGEvents and highlight potential space switching events.\n");
    printf("Press Ctrl+C to stop.\n\n");
    
    // Check accessibility permissions
    if (!AXIsProcessTrusted()) {
        printf("ERROR: This program needs Accessibility permissions!\n");
        printf("Go to: System Preferences > Security & Privacy > Privacy > Accessibility\n");
        printf("Add Terminal.app (or this program) to the list.\n");
        return 1;
    }
    
    // Set up signal handler for clean shutdown
    signal(SIGINT, signal_handler);
    signal(SIGTERM, signal_handler);
    
    // Create event tap to monitor ALL events
    // We use kCGEventTapOptionListenOnly to avoid modifying events
    eventTap = CGEventTapCreate(
        kCGSessionEventTap,                    // Tap at session level
        kCGHeadInsertEventTap,                 // Insert at head
        kCGEventTapOptionListenOnly,           // Listen only, don't modify
        kCGEventMaskForAllEvents,              // Monitor all event types
        eventTapCallback,                      // Our callback function
        NULL                                   // No user data
    );
    
    if (!eventTap) {
        printf("ERROR: Failed to create event tap!\n");
        printf("Make sure you have accessibility permissions and try again.\n");
        return 1;
    }
    
    // Create run loop source
    runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, eventTap, 0);
    if (!runLoopSource) {
        printf("ERROR: Failed to create run loop source!\n");
        CFRelease(eventTap);
        return 1;
    }

    // Add to run loop
    CFRunLoopAddSource(CFRunLoopGetCurrent(), runLoopSource, kCFRunLoopCommonModes);

    // Enable the event tap
    CGEventTapEnable(eventTap, true);

    printf("✓ Event tap created and enabled\n");
    printf("✓ Monitoring all CGEvents...\n\n");
    printf("Now try switching spaces to see the events!\n\n");

    // Run the event loop
    CFRunLoopRun();

    // Cleanup (though we usually won't reach here due to signal handler)
    signal_handler(0);
    
    return 0;
}
