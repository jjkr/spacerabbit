# SpaceRabbit Settings Implementation Plan

## Overview

This document outlines the complete implementation plan for adding a modern settings interface to the SpaceRabbit workspace switcher app. The implementation will transform the current basic HTML frontend into a React + Vite + TypeScript application with a native macOS window containing a modern dark web UI.

## Current State Analysis

### Existing Architecture
- **Backend**: Rust with Tauri 2.3, global hotkeys (Alt+H/Alt+L), tray icon, workspace switching
- **Frontend**: Basic HTML/JS in `src/` directory
- **Configuration**: Hardcoded hotkeys and settings in Rust code
- **Window**: Hidden main window, tray-only interface

### Current Limitations
- No user-configurable settings
- Hardcoded hotkey combinations
- No persistent configuration storage
- Basic UI with no settings interface

## Target Architecture

### Frontend Stack
- **React 18** with TypeScript for component-based UI
- **Vite** for modern build tooling and hot reload
- **Tailwind CSS** for styling with dark theme
- **Auto-save** settings on every change
- **Native OS window** with modern web UI inside

### Backend Enhancements
- **tauri-plugin-store** for persistent settings storage
- **Dynamic hotkey registration** based on user settings
- **Settings validation** and error handling
- **Tauri commands** for frontend-backend communication

### Window Behavior
- **Single window** that serves as main settings interface
- **Stays open** until manually closed by user
- **Closes to menu bar** (app continues running in background)
- **Native macOS window** with standard controls
- **Resizable** with sensible default size (600x500)

## Implementation Phases

### Phase 1: Project Setup & Migration
**Goal**: Transform basic HTML setup to React + Vite + TypeScript

#### 1.1 Frontend Migration
- [ ] Initialize Vite + React + TypeScript setup
- [ ] Install required dependencies
- [ ] Configure Tailwind CSS for dark theme
- [ ] Update `tauri.conf.json` for Vite build process
- [ ] Create basic React app structure

#### 1.2 Dependencies
**Frontend**:
```json
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "@tauri-apps/api": "^2.3.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "@vitejs/plugin-react": "^4.0.0",
    "typescript": "^5.0.0",
    "vite": "^4.4.0",
    "tailwindcss": "^3.3.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0"
  }
}
```

**Backend**:
```toml
[dependencies]
tauri-plugin-store = "2.3"
```

#### 1.3 Configuration Updates
- [ ] Update `tauri.conf.json` window configuration
- [ ] Configure Vite build process
- [ ] Set up TypeScript configuration
- [ ] Configure Tailwind CSS

### Phase 2: Settings Schema & Backend
**Goal**: Create settings data structures and persistence layer

#### 2.1 Settings Schema
```typescript
interface AppSettings {
  hotkeys: {
    leftWorkspace: string;    // "Alt+H"
    rightWorkspace: string;   // "Alt+L"
  };
  ui: {
    showDesktopNumber: boolean;
    startMinimized: boolean;
  };
  behavior: {
    wrapAroundWorkspaces: boolean;
    startWithSystem: boolean;
  };
}
```

#### 2.2 Rust Settings Module
- [ ] Create `src-tauri/src/settings.rs`
- [ ] Implement settings struct with serde
- [ ] Add Tauri commands for settings CRUD operations
- [ ] Integrate tauri-plugin-store for persistence
- [ ] Add settings validation logic

#### 2.3 Tauri Commands
- [ ] `get_settings()` - Load current settings
- [ ] `save_settings(settings)` - Save and apply settings
- [ ] `show_settings_window()` - Show/focus main window
- [ ] `validate_hotkey(hotkey)` - Validate hotkey combinations

### Phase 3: React Settings UI
**Goal**: Create modern, responsive settings interface

#### 3.1 Component Architecture
```
src/
├── main.tsx                 # React entry point
├── App.tsx                  # Main settings component
├── components/
│   ├── SettingsLayout.tsx   # Main layout with dark theme
│   ├── HotkeyInput.tsx      # Custom hotkey input component
│   ├── ToggleSwitch.tsx     # Custom toggle component
│   ├── SettingsSection.tsx  # Reusable section wrapper
│   └── StatusIndicator.tsx  # Connection/save status
├── hooks/
│   ├── useSettings.ts       # Settings state management
│   ├── useTauri.ts         # Tauri API wrapper
│   └── useHotkeys.ts       # Hotkey validation logic
├── types/
│   └── settings.ts         # TypeScript interfaces
└── styles/
    └── globals.css         # Dark theme + custom styles
```

#### 3.2 Key Components
- [ ] **SettingsLayout**: Main container with dark theme
- [ ] **HotkeyInput**: Custom input for hotkey combinations
- [ ] **ToggleSwitch**: Styled toggle switches
- [ ] **SettingsSection**: Collapsible sections with icons
- [ ] **StatusIndicator**: Real-time save/connection status

#### 3.3 Settings Sections
1. **🔥 Hotkeys**
   - Left workspace hotkey input
   - Right workspace hotkey input
   - Real-time conflict detection
   - Visual feedback for valid/invalid combinations

2. **🎨 Interface**
   - Show desktop number in tray toggle
   - Start minimized toggle
   - Future: Theme selection, icon style

3. **⚙️ Behavior**
   - Wrap around workspaces toggle
   - Start with system toggle
   - Future: Animation preferences, timing settings

### Phase 4: Auto-Save & State Management
**Goal**: Implement seamless auto-save functionality

#### 4.1 Settings Hook
- [ ] Create `useSettings` hook for state management
- [ ] Implement auto-save on settings changes
- [ ] Add debouncing for rapid changes
- [ ] Handle loading and error states
- [ ] Provide optimistic updates

#### 4.2 Auto-Save Logic
```typescript
// Auto-save whenever settings change
useEffect(() => {
  if (!isLoading && hasChanges) {
    const timeoutId = setTimeout(() => {
      invoke('save_settings', { settings })
        .then(() => setHasChanges(false))
        .catch(handleError);
    }, 500); // 500ms debounce
    
    return () => clearTimeout(timeoutId);
  }
}, [settings, isLoading]);
```

#### 4.3 Error Handling
- [ ] Network/connection error handling
- [ ] Settings validation errors
- [ ] User-friendly error messages
- [ ] Retry mechanisms for failed saves

### Phase 5: Integration & Dynamic Hotkeys
**Goal**: Wire settings to existing functionality

#### 5.1 Dynamic Hotkey Registration
- [ ] Modify `main.rs` to use settings-based hotkeys
- [ ] Implement hotkey re-registration on settings change
- [ ] Add hotkey conflict detection
- [ ] Handle hotkey registration failures gracefully

#### 5.2 Tray Menu Integration
- [ ] Add "Settings" menu item to tray
- [ ] Implement window show/hide logic
- [ ] Update tray behavior based on UI settings
- [ ] Handle window close events properly

#### 5.3 Settings Application
- [ ] Apply hotkey changes immediately
- [ ] Update tray display based on UI settings
- [ ] Implement workspace wrapping behavior
- [ ] Handle startup behavior settings

### Phase 6: UI Polish & UX
**Goal**: Create professional, polished user experience

#### 6.1 Dark Theme Design
- **Color Palette**:
  - Background: `bg-gray-900` (#111827)
  - Cards: `bg-gray-800` (#1f2937)
  - Text: `text-white` / `text-gray-300`
  - Accents: `text-blue-400` / `bg-blue-600`
  - Borders: `border-gray-700`

#### 6.2 Visual Design Elements
- [ ] Smooth animations for toggles and state changes
- [ ] Hover effects and interactive feedback
- [ ] Loading states and progress indicators
- [ ] Success/error visual feedback
- [ ] Consistent spacing and typography

#### 6.3 Accessibility & UX
- [ ] Keyboard navigation support
- [ ] Screen reader compatibility
- [ ] Focus management
- [ ] Intuitive tab order
- [ ] Clear visual hierarchy

### Phase 7: Testing & Validation
**Goal**: Ensure reliability and user experience

#### 7.1 Functionality Testing
- [ ] Settings persistence across app restarts
- [ ] Hotkey registration and conflict detection
- [ ] Auto-save functionality
- [ ] Window behavior (show/hide/close)
- [ ] Tray menu integration

#### 7.2 Edge Cases
- [ ] Invalid hotkey combinations
- [ ] Settings file corruption
- [ ] Network/storage errors
- [ ] Rapid settings changes
- [ ] App startup with corrupted settings

#### 7.3 Performance
- [ ] Settings load time optimization
- [ ] Auto-save debouncing effectiveness
- [ ] Memory usage monitoring
- [ ] UI responsiveness under load

## File Structure Changes

### New Files to Create
```
src/
├── main.tsx                 # React entry point
├── App.tsx                  # Main settings component
├── index.html               # Vite HTML template
├── components/              # React components
├── hooks/                   # Custom React hooks
├── types/                   # TypeScript type definitions
└── styles/                  # CSS and styling

src-tauri/src/
├── settings.rs              # Settings module
└── (modifications to existing files)

docs/
└── settings_implementation_plan.md  # This file
```

### Files to Modify
```
src-tauri/
├── Cargo.toml              # Add tauri-plugin-store dependency
├── tauri.conf.json         # Update build config and window settings
├── src/main.rs             # Integrate settings, update tray menu
├── src/lib.rs              # Export settings module
└── src/workspace_switcher.rs  # Use dynamic hotkeys from settings
```

### Files to Remove/Replace
```
src/
├── main.js                 # Replace with main.tsx
└── (existing basic HTML/JS files)
```

## Success Criteria

### Functional Requirements
- [ ] Settings persist across app restarts
- [ ] Hotkeys can be customized and work immediately
- [ ] Settings auto-save without user intervention
- [ ] Window opens from tray menu and closes to tray
- [ ] All existing workspace switching functionality preserved

### User Experience Requirements
- [ ] Modern, professional dark theme interface
- [ ] Intuitive settings organization and labeling
- [ ] Immediate visual feedback for all interactions
- [ ] No data loss during settings changes
- [ ] Smooth, responsive UI with proper loading states

### Technical Requirements
- [ ] Type-safe TypeScript implementation
- [ ] Proper error handling and user feedback
- [ ] Efficient auto-save with debouncing
- [ ] Clean, maintainable code architecture
- [ ] Cross-platform compatibility (focus on macOS)

## Future Enhancements

### Potential Additional Features
- [ ] **Advanced Hotkeys**: Support for more complex key combinations
- [ ] **Multiple Displays**: Per-display settings and behavior
- [ ] **Workspace Names**: Custom names for workspaces
- [ ] **Animations**: Customizable transition animations
- [ ] **Themes**: Light/dark theme toggle
- [ ] **Import/Export**: Settings backup and sharing
- [ ] **Profiles**: Multiple settings profiles
- [ ] **Advanced Gestures**: Custom gesture configurations

### Architecture Improvements
- [ ] **Settings Validation**: More robust validation system
- [ ] **Plugin System**: Extensible settings architecture
- [ ] **Performance**: Settings caching and optimization
- [ ] **Logging**: Comprehensive logging for debugging
- [ ] **Analytics**: Usage analytics for feature improvement

## Implementation Timeline

### Week 1: Foundation
- Phase 1: Project setup and migration
- Phase 2: Settings schema and backend

### Week 2: Core Features
- Phase 3: React settings UI
- Phase 4: Auto-save and state management

### Week 3: Integration
- Phase 5: Integration with existing functionality
- Phase 6: UI polish and UX improvements

### Week 4: Testing & Refinement
- Phase 7: Testing and validation
- Bug fixes and performance optimization
- Documentation and final polish

## Notes

### Development Considerations
- Maintain backward compatibility during migration
- Test thoroughly on macOS (primary target platform)
- Keep settings schema extensible for future features
- Prioritize user experience and data safety
- Follow React and TypeScript best practices

### Risk Mitigation
- Implement settings validation to prevent corruption
- Add fallback to default settings if loading fails
- Ensure hotkey conflicts are properly detected and resolved
- Test auto-save thoroughly to prevent data loss
- Maintain existing functionality during migration

This plan provides a comprehensive roadmap for implementing a modern, professional settings interface for SpaceRabbit while maintaining all existing functionality and following current best practices for React, TypeScript, and Tauri development.
