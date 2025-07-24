# macOS Native Styling Guide for SpaceRabbit

## Overview
This guide defines the native macOS styling approach for SpaceRabbit's settings interface, using CSS Modules and native system colors to create an authentic macOS experience.

## Design Principles

### 1. Native System Integration
- Use macOS system colors that adapt to user preferences
- Follow Apple's Human Interface Guidelines
- Respect system-wide dark/light mode preferences
- Use native spacing and typography scales

### 2. Performance First
- CSS Modules for component-scoped styles
- Zero runtime CSS generation
- Minimal bundle size
- Efficient CSS delivery

## Color System

### CSS Variables (macos-variables.css)
```css
:root {
  /* Background Colors */
  --macos-bg-primary: color(display-p3 0.11 0.11 0.12);
  --macos-bg-secondary: color(display-p3 0.16 0.16 0.18);
  --macos-bg-tertiary: color(display-p3 0.19 0.19 0.21);
  --macos-bg-elevated: color(display-p3 0.22 0.22 0.24);
  
  /* Text Colors */
  --macos-text-primary: color(display-p3 0.98 0.98 0.98);
  --macos-text-secondary: color(display-p3 0.78 0.78 0.80);
  --macos-text-tertiary: color(display-p3 0.55 0.55 0.58);
  --macos-text-quaternary: color(display-p3 0.36 0.36 0.38);
  
  /* Accent Colors */
  --macos-accent-blue: color(display-p3 0.0 0.48 1.0);
  --macos-accent-blue-hover: color(display-p3 0.0 0.42 0.88);
  --macos-accent-blue-active: color(display-p3 0.0 0.36 0.76);
  
  /* Status Colors */
  --macos-green: color(display-p3 0.20 0.78 0.35);
  --macos-red: color(display-p3 1.0 0.27 0.23);
  --macos-orange: color(display-p3 1.0 0.58 0.0);
  
  /* Border & Separator */
  --macos-separator: color(display-p3 0.27 0.27 0.30);
  --macos-border: color(display-p3 0.33 0.33 0.36);
  
  /* Shadows */
  --macos-shadow-light: rgba(0, 0, 0, 0.1);
  --macos-shadow-medium: rgba(0, 0, 0, 0.2);
  --macos-shadow-heavy: rgba(0, 0, 0, 0.3);
}

/* Light mode overrides (system preference) */
@media (prefers-color-scheme: light) {
  :root {
    --macos-bg-primary: color(display-p3 1.0 1.0 1.0);
    --macos-bg-secondary: color(display-p3 0.96 0.96 0.97);
    --macos-bg-tertiary: color(display-p3 0.92 0.92 0.93);
    --macos-bg-elevated: color(display-p3 1.0 1.0 1.0);
    
    --macos-text-primary: color(display-p3 0.0 0.0 0.0);
    --macos-text-secondary: color(display-p3 0.24 0.24 0.26);
    --macos-text-tertiary: color(display-p3 0.44 0.44 0.46);
    --macos-text-quaternary: color(display-p3 0.64 0.64 0.66);
    
    --macos-separator: color(display-p3 0.78 0.78 0.78);
    --macos-border: color(display-p3 0.70 0.70 0.70);
  }
}
```

## Typography

### Font Stack
```css
:root {
  --macos-font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'SF Pro Text', system-ui, sans-serif;
  --macos-font-mono: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
}

/* Typography Scale */
:root {
  --macos-text-xs: 0.75rem;    /* 12px */
  --macos-text-sm: 0.875rem;   /* 14px */
  --macos-text-base: 1rem;     /* 16px */
  --macos-text-lg: 1.125rem;   /* 18px */
  --macos-text-xl: 1.25rem;    /* 20px */
  --macos-text-2xl: 1.5rem;    /* 24px */
}

/* Font Weights */
:root {
  --macos-font-regular: 400;
  --macos-font-medium: 500;
  --macos-font-semibold: 600;
  --macos-font-bold: 700;
}
```

## Spacing System

### Spacing Scale
```css
:root {
  --macos-space-1: 0.25rem;   /* 4px */
  --macos-space-2: 0.5rem;    /* 8px */
  --macos-space-3: 0.75rem;   /* 12px */
  --macos-space-4: 1rem;      /* 16px */
  --macos-space-5: 1.25rem;   /* 20px */
  --macos-space-6: 1.5rem;    /* 24px */
  --macos-space-8: 2rem;      /* 32px */
  --macos-space-10: 2.5rem;   /* 40px */
  --macos-space-12: 3rem;     /* 48px */
  --macos-space-16: 4rem;     /* 64px */
}
```

## Component Patterns

### 1. Settings Window Layout
```css
/* SettingsLayout.module.css */
.container {
  background: var(--macos-bg-primary);
  color: var(--macos-text-primary);
  font-family: var(--macos-font-family);
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.header {
  background: var(--macos-bg-secondary);
  border-bottom: 1px solid var(--macos-separator);
  padding: var(--macos-space-4) var(--macos-space-6);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.title {
  font-size: var(--macos-text-lg);
  font-weight: var(--macos-font-semibold);
  color: var(--macos-text-primary);
}

.content {
  flex: 1;
  padding: var(--macos-space-6);
  overflow-y: auto;
}
```

### 2. Settings Section
```css
/* SettingsSection.module.css */
.section {
  background: var(--macos-bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--macos-separator);
  margin-bottom: var(--macos-space-6);
  overflow: hidden;
}

.header {
  padding: var(--macos-space-4) var(--macos-space-5);
  border-bottom: 1px solid var(--macos-separator);
  background: var(--macos-bg-tertiary);
}

.title {
  font-size: var(--macos-text-base);
  font-weight: var(--macos-font-medium);
  color: var(--macos-text-primary);
  display: flex;
  align-items: center;
  gap: var(--macos-space-2);
}

.icon {
  width: 16px;
  height: 16px;
  color: var(--macos-accent-blue);
}

.content {
  padding: var(--macos-space-5);
}
```

### 3. Toggle Switch (Native Style)
```css
/* ToggleSwitch.module.css */
.container {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--macos-space-3) 0;
}

.label {
  font-size: var(--macos-text-base);
  color: var(--macos-text-primary);
  font-weight: var(--macos-font-regular);
}

.switch {
  position: relative;
  width: 44px;
  height: 24px;
  background: var(--macos-separator);
  border-radius: 12px;
  border: none;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.switch:checked {
  background: var(--macos-accent-blue);
}

.switch::before {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 20px;
  height: 20px;
  background: white;
  border-radius: 50%;
  transition: transform 0.2s ease;
  box-shadow: 0 1px 3px var(--macos-shadow-light);
}

.switch:checked::before {
  transform: translateX(20px);
}

.switch:focus {
  outline: 2px solid var(--macos-accent-blue);
  outline-offset: 2px;
}
```

### 4. Hotkey Input
```css
/* HotkeyInput.module.css */
.container {
  display: flex;
  flex-direction: column;
  gap: var(--macos-space-2);
}

.label {
  font-size: var(--macos-text-sm);
  color: var(--macos-text-secondary);
  font-weight: var(--macos-font-medium);
}

.input {
  background: var(--macos-bg-tertiary);
  border: 1px solid var(--macos-border);
  border-radius: 6px;
  padding: var(--macos-space-3) var(--macos-space-4);
  font-family: var(--macos-font-mono);
  font-size: var(--macos-text-sm);
  color: var(--macos-text-primary);
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}

.input:focus {
  outline: none;
  border-color: var(--macos-accent-blue);
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.2);
}

.input.error {
  border-color: var(--macos-red);
}

.input.error:focus {
  box-shadow: 0 0 0 3px rgba(255, 69, 58, 0.2);
}

.errorMessage {
  font-size: var(--macos-text-xs);
  color: var(--macos-red);
  margin-top: var(--macos-space-1);
}
```

## Animation Guidelines

### Transitions
```css
:root {
  --macos-transition-fast: 0.15s ease;
  --macos-transition-normal: 0.2s ease;
  --macos-transition-slow: 0.3s ease;
}

/* Common transition patterns */
.interactive {
  transition: 
    background-color var(--macos-transition-normal),
    border-color var(--macos-transition-normal),
    box-shadow var(--macos-transition-normal),
    transform var(--macos-transition-fast);
}

.interactive:hover {
  transform: translateY(-1px);
}

.interactive:active {
  transform: translateY(0);
}
```

## Accessibility

### Focus Management
```css
/* Global focus styles */
*:focus {
  outline: 2px solid var(--macos-accent-blue);
  outline-offset: 2px;
}

/* Custom focus for specific components */
.customFocus:focus {
  outline: none;
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.3);
}
```

### High Contrast Support
```css
@media (prefers-contrast: high) {
  :root {
    --macos-border: color(display-p3 0.5 0.5 0.5);
    --macos-separator: color(display-p3 0.4 0.4 0.4);
  }
}
```

## Build Configuration

### Vite Configuration
```typescript
// vite.config.ts
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  css: {
    modules: {
      localsConvention: 'camelCase',
      generateScopedName: '[name]__[local]___[hash:base64:5]'
    },
    postcss: {
      plugins: [
        require('postcss-preset-env')({
          stage: 1,
          features: {
            'nesting-rules': true,
            'custom-properties': true
          }
        })
      ]
    }
  }
})
```

### PostCSS Configuration
```javascript
// postcss.config.js
module.exports = {
  plugins: {
    'postcss-preset-env': {
      stage: 1,
      features: {
        'nesting-rules': true,
        'custom-properties': true,
        'color-function': true
      }
    },
    'postcss-nesting': {}
  }
}
```

## Usage Examples

### Component with CSS Modules
```typescript
// HotkeyInput.tsx
import React from 'react'
import styles from './HotkeyInput.module.css'

interface HotkeyInputProps {
  label: string
  value: string
  onChange: (value: string) => void
  error?: string
}

export const HotkeyInput: React.FC<HotkeyInputProps> = ({
  label,
  value,
  onChange,
  error
}) => {
  return (
    <div className={styles.container}>
      <label className={styles.label}>{label}</label>
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className={`${styles.input} ${error ? styles.error : ''}`}
        placeholder="Press keys..."
      />
      {error && <div className={styles.errorMessage}>{error}</div>}
    </div>
  )
}
```

This styling approach provides:
- **Authentic macOS feel** with native system colors
- **Performance optimization** with CSS Modules
- **Accessibility compliance** with proper focus management
- **Maintainable code** with component-scoped styles
- **Future-proof** design that adapts to system preferences