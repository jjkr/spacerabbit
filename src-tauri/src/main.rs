use quickspace::workspace_switcher;
use quickspace::window_manager;

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState, hotkey::{HotKey, Modifiers, Code}};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, AppHandle,
};

// Shared state for tracking current desktop and display
#[derive(Debug, Clone)]
struct AppState {
    current_desktop: Arc<Mutex<u32>>,
    last_cursor_display: Arc<Mutex<u32>>,
    mission_control_active: Arc<Mutex<bool>>,
}

fn main() {
    println!("QuickSpace starting...");

    // Initialize shared state
    let app_state = AppState {
        current_desktop: Arc::new(Mutex::new(1)),
        last_cursor_display: Arc::new(Mutex::new(0)),
        mission_control_active: Arc::new(Mutex::new(false)),
    };

    // Get initial desktop number
    let initial_desktop = workspace_switcher::get_current_desktop().unwrap_or(1);

    // Update shared state
    if let Ok(mut desktop) = app_state.current_desktop.lock() {
        *desktop = initial_desktop;
    }

    // Initialize global hotkey manager
    let manager = GlobalHotKeyManager::new().expect("Failed to create hotkey manager");

    // Create hotkeys: Alt+H (left), Alt+L (right), Alt+E (mission control), and Alt+Tab (window cycling)
    let hotkey_left = HotKey::new(Some(Modifiers::ALT), Code::KeyH);   // Alt+H = left
    let hotkey_right = HotKey::new(Some(Modifiers::ALT), Code::KeyL);  // Alt+L = right
    let hotkey_mission_control = HotKey::new(Some(Modifiers::ALT), Code::KeyE); // Alt+E = mission control
    let hotkey_window_cycle = HotKey::new(Some(Modifiers::ALT), Code::Tab); // Alt+Tab = window cycling

    // Register all hotkeys
    manager.register(hotkey_left).expect("Failed to register Alt+H hotkey");
    manager.register(hotkey_right).expect("Failed to register Alt+L hotkey");
    manager.register(hotkey_mission_control).expect("Failed to register Alt+E hotkey");
    manager.register(hotkey_window_cycle).expect("Failed to register Alt+Tab hotkey");
    println!("Registered Alt+H (left), Alt+L (right), Alt+E (mission control), and Alt+Tab (window cycling) global hotkeys");

    let mut app = tauri::Builder::default()
        .manage(app_state.clone())
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            let app_handle = app.handle();
            let state = app_state.clone();

            // Create tray menu
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&hide, &quit])?;

            // Create tray icon with initial desktop number
            let initial_title = format!("{}", initial_desktop);
            let _tray = TrayIconBuilder::with_id("main")
                .menu(&menu)
                .title(&initial_title)
                .on_tray_icon_event(move |_tray, event| {
                    match event {
                        TrayIconEvent::Click { .. } => {
                            println!("Tray icon clicked");
                        }
                        TrayIconEvent::DoubleClick { .. } => {
                            println!("Tray icon double-clicked");
                        }
                        _ => {}
                    }
                })
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "quit" => {
                            println!("Quit menu item clicked - exiting");
                            std::process::exit(0);
                        }
                        "hide" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            println!("Setting initial tray text to: {}", initial_desktop);

            // Hide the main window on startup
            if let Some(window) = app.get_webview_window("main") {
                window.hide().expect("Failed to hide window");
            }

            // Spawn hotkey listener thread
            let app_handle_hotkey = app_handle.clone();
            let state_hotkey = state.clone();
            thread::spawn(move || {
                hotkey_listener_thread(app_handle_hotkey, state_hotkey, manager, hotkey_left, hotkey_right, hotkey_mission_control, hotkey_window_cycle);
            });

            // Spawn cursor monitoring thread
            let app_handle_cursor = app_handle.clone();
            let state_cursor = state.clone();
            thread::spawn(move || {
                cursor_monitor_thread(app_handle_cursor, state_cursor);
            });

            println!("QuickSpace is now running in the background.");
            println!("   Alt+H: Switch to left workspace");
            println!("   Alt+L: Switch to right workspace");
            println!("   Alt+E: Toggle Mission Control");
            println!("   Alt+Tab: Cycle through windows on current workspace");
            println!("   Tray shows current desktop number for cursor's display");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    // Configure app to not show in dock on macOS
    #[cfg(target_os = "macos")]
    {
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }

    app.run(|_app_handle, event| {
        // Prevent the app from exiting when the last window is closed
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            api.prevent_exit();
        }
    });
}

/// Hotkey listener thread
fn hotkey_listener_thread(
    app_handle: AppHandle,
    state: AppState,
    _manager: GlobalHotKeyManager,
    hotkey_left: HotKey,
    hotkey_right: HotKey,
    hotkey_mission_control: HotKey,
    hotkey_window_cycle: HotKey,
) {
    loop {
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            // Only trigger on key press, not release
            if event.state == HotKeyState::Pressed {

                // Handle workspace switching based on which hotkey was pressed
                match event.id {
                    id if id == hotkey_left.id() => {
                        if let Err(e) = workspace_switcher::switch_left() {
                            eprintln!("Failed to switch left: {}", e);
                        } else {
                            // Update tray text after workspace change (small delay for system to catch up)
                            thread::sleep(Duration::from_millis(100));
                            update_tray_text_for_current_context(&app_handle, &state);
                        }
                    },
                    id if id == hotkey_right.id() => {
                        if let Err(e) = workspace_switcher::switch_right() {
                            eprintln!("Failed to switch right: {}", e);
                        } else {
                            // Update tray text after workspace change (small delay for system to catch up)
                            thread::sleep(Duration::from_millis(100));
                            update_tray_text_for_current_context(&app_handle, &state);
                        }
                    },
                    id if id == hotkey_mission_control.id() => {
                        // Toggle Mission Control state
                        let is_active = {
                            let mut mc_active = state.mission_control_active.lock().unwrap();
                            let current_state = *mc_active;
                            *mc_active = !current_state;
                            current_state
                        };

                        if is_active {
                            // Mission Control is currently active, so deactivate it
                            if let Err(e) = workspace_switcher::deactivate_mission_control() {
                                eprintln!("Failed to deactivate mission control: {}", e);
                                // Revert state on error
                                if let Ok(mut mc_active) = state.mission_control_active.lock() {
                                    *mc_active = true;
                                }
                            } else {
                                println!("Mission Control deactivated");
                            }
                        } else {
                            // Mission Control is currently inactive, so activate it
                            if let Err(e) = workspace_switcher::activate_mission_control() {
                                eprintln!("Failed to activate mission control: {}", e);
                                // Revert state on error
                                if let Ok(mut mc_active) = state.mission_control_active.lock() {
                                    *mc_active = false;
                                }
                            } else {
                                println!("Mission Control activated");
                            }
                        }
                    },
                    id if id == hotkey_window_cycle.id() => {
                        // Cycle to next window on current workspace
                        if let Err(e) = window_manager::cycle_next_window() {
                            eprintln!("Failed to cycle to next window: {}", e);
                        } else {
                            println!("Cycled to next window");
                        }
                    },
                    _ => {
                        println!("Unknown hotkey event: {:?}", event);
                    }
                }

            }
        }

        // sleep to prevent busy wait
        thread::sleep(Duration::from_millis(10));
    }
}

/// Cursor monitoring thread - updates tray text when cursor moves between displays
fn cursor_monitor_thread(app_handle: AppHandle, state: AppState) {
    let mut last_update = std::time::Instant::now();

    loop {
        // Check cursor position every 100ms
        thread::sleep(Duration::from_millis(100));

        // Don't update too frequently
        if last_update.elapsed() < Duration::from_millis(80) {
            continue;
        }

        // Get current cursor display info
        if let Ok(display_info) = workspace_switcher::get_cursor_display_info() {
            let should_update = {
                let mut last_display = state.last_cursor_display.lock().unwrap();
                let mut current_desktop = state.current_desktop.lock().unwrap();

                let display_changed = *last_display != display_info.display_id;
                let desktop_changed = *current_desktop != display_info.current_desktop;

                // Only log when there are actual changes
                if display_changed || desktop_changed {
                    println!("Desktop switched to: {}", display_info.current_desktop);
                }

                if display_changed || desktop_changed {
                    *last_display = display_info.display_id;
                    *current_desktop = display_info.current_desktop;
                    true
                } else {
                    false
                }
            };

            if should_update {
                update_tray_text(&app_handle, display_info.current_desktop);
                last_update = std::time::Instant::now();
            }
        } else {
            // Fallback: update periodically even if cursor detection fails
            if last_update.elapsed() > Duration::from_secs(5) {
                update_tray_text_for_current_context(&app_handle, &state);
                last_update = std::time::Instant::now();
            }
        }
    }
}

/// Update tray text with the given desktop number
fn update_tray_text(app_handle: &AppHandle, desktop_num: u32) {
    // Show the desktop number as text in the menu bar
    let title = format!("{}", desktop_num);

    if let Some(tray) = app_handle.tray_by_id("main") {
        if let Err(e) = tray.set_title(Some(&title)) {
            eprintln!("Failed to update tray title: {}", e);
        }
    }
}

/// Update tray text based on current cursor context
fn update_tray_text_for_current_context(app_handle: &AppHandle, state: &AppState) {
    match workspace_switcher::get_current_desktop() {
        Ok(desktop_num) => {
            // Update state
            if let Ok(mut current) = state.current_desktop.lock() {
                *current = desktop_num;
            }
            update_tray_text(app_handle, desktop_num);
        }
        Err(e) => {
            eprintln!("Failed to get current context desktop: {}", e);
        }
    }
}
