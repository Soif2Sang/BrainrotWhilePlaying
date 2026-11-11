#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
mod bindings;
use bindings::Binding;

#[derive(serde::Serialize)]
struct Status {
    game_mode: bool,
    help: bool,
    compact: bool,
    error: Option<String>,
    bindings: Vec<Binding>,
    capturing: bool,
    listening: bool,
    #[serde(skip)]
    ready: bool,
    #[serde(skip)]
    resume_game: bool,
    #[serde(skip)]
    hidden: bool,
    #[serde(skip)]
    corner: usize,
    #[serde(skip)]
    size: usize,
    #[serde(skip)]
    return_focus: Option<isize>,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            game_mode: false,
            help: true,
            compact: true,
            error: None,
            bindings: bindings::defaults(),
            capturing: false,
            listening: false,
            ready: false,
            resume_game: false,
            hidden: false,
            corner: 0,
            size: 1,
            return_focus: None,
        }
    }
}
type Shared = Mutex<Status>;
struct BindingGate(Mutex<()>);

fn sync_overlay(app: &tauri::AppHandle) {
    let payload = app
        .state::<Shared>()
        .lock()
        .ok()
        .and_then(|state| serde_json::to_string(&*state).ok());
    if let Some(payload) = payload {
        if let Some(window) = app.get_webview("tiktok") {
            let _ = window.eval(format!(
                "document.documentElement.classList.toggle('padview-compact', ({payload}).compact)"
            ));
        }
        if let Some(overlay) = app.get_webview("settings") {
            let _ = overlay.eval(format!("window.updateSettings?.({payload})"));
        }
    }
}

fn has_focus(app: &tauri::AppHandle) -> bool {
    let Some(window) = app.get_window("tiktok") else {
        return false;
    };
    #[cfg(windows)]
    {
        use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
        let foreground = unsafe { GetForegroundWindow() }.0 as isize;
        window
            .hwnd()
            .is_ok_and(|hwnd| hwnd.0 as isize == foreground)
    }
    #[cfg(not(windows))]
    {
        window.is_focused().unwrap_or(false)
    }
}

fn refresh_bindings(app: &tauri::AppHandle) -> Result<(), String> {
    let gate = app.state::<BindingGate>();
    let _guard = gate.0.lock().map_err(|e| e.to_string())?;
    let (all, desired, listening) = {
        let shared = app.state::<Shared>();
        let state = shared.lock().map_err(|e| e.to_string())?;
        if !state.ready {
            return Ok(());
        }
        let listening = !state.capturing;
        (
            state.bindings.clone(),
            bindings::enabled(&state.bindings, state.capturing),
            listening,
        )
    };
    let wanted: std::collections::HashSet<_> = desired
        .iter()
        .map(|b| b.shortcut.parse::<Shortcut>().map(|s| s.id()))
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    for shortcut in bindings::validate(&all)? {
        if !wanted.contains(&shortcut.id()) && app.global_shortcut().is_registered(shortcut) {
            app.global_shortcut()
                .unregister(shortcut)
                .map_err(|e| e.to_string())?;
        }
    }
    let mut failure = None;
    for binding in &desired {
        let shortcut = binding
            .shortcut
            .parse::<Shortcut>()
            .map_err(|e| e.to_string())?;
        if !app.global_shortcut().is_registered(shortcut) {
            if let Err(error) = app.global_shortcut().register(shortcut) {
                failure = Some(format!(
                    "Raccourci indisponible ({}) : {error}",
                    binding.shortcut
                ));
                break;
            }
        }
    }
    if failure.is_some() {
        // Release action keys if activation fails; keep the focus shortcut available.
        for binding in all.iter().filter(|b| b.action != "mode") {
            let shortcut = binding
                .shortcut
                .parse::<Shortcut>()
                .map_err(|e| e.to_string())?;
            if app.global_shortcut().is_registered(shortcut) {
                let _ = app.global_shortcut().unregister(shortcut);
            }
        }
    }
    {
        let shared = app.state::<Shared>();
        let mut state = shared.lock().map_err(|e| e.to_string())?;
        state.listening = listening && failure.is_none();
        if let Some(error) = &failure {
            state.error = Some(error.clone());
        }
    }
    sync_overlay(app);
    failure.map_or(Ok(()), Err)
}

fn arrange(app: &tauri::AppHandle, reposition: bool) -> Result<(), String> {
    let window = app.get_window("tiktok").ok_or("Fenêtre fermée")?;
    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or("Écran indisponible")?;
    let scale = monitor.scale_factor();
    let area = monitor.work_area();
    let state = app.state::<Shared>();
    let state = state.lock().map_err(|e| e.to_string())?;
    let desired_width = [280.0_f64, 340.0, 400.0][state.size];
    let width = desired_width
        .min((area.size.height as f64 / scale - 32.0) * 9.0 / 16.0)
        .min(area.size.width as f64 / scale - 32.0)
        .max(160.0);
    let height = width * 16.0 / 9.0;
    let right = state.corner == 1 || state.corner == 2;
    let bottom = state.corner >= 2;
    let x = area.position.x as f64 / scale
        + if right {
            area.size.width as f64 / scale - width - 16.0
        } else {
            16.0
        };
    let y = area.position.y as f64 / scale
        + if bottom {
            area.size.height as f64 / scale - height - 16.0
        } else {
            16.0
        };
    drop(state);
    window
        .set_size(tauri::LogicalSize::new(width, height))
        .map_err(|e| e.to_string())?;
    if reposition {
        window
            .set_position(tauri::LogicalPosition::new(x, y))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn set_mode(app: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let window = app.get_window("tiktok").ok_or("Fenêtre fermée")?;
    let shared = app.state::<Shared>();
    let mut state = shared.lock().map_err(|e| e.to_string())?;
    #[cfg(windows)]
    if !enabled {
        use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
        let foreground = unsafe { GetForegroundWindow() }.0 as isize;
        if foreground != 0 && window.hwnd().is_ok_and(|h| h.0 as isize != foreground) {
            state.return_focus = Some(foreground);
        }
    }
    let result = (|| -> tauri::Result<()> {
        window.set_ignore_cursor_events(enabled)?;
        window.set_focusable(!enabled)?;
        if !enabled {
            window.show()?;
            window.set_focus()?;
        }
        Ok(())
    })();
    if let Err(error) = result {
        let _ = window.set_ignore_cursor_events(false);
        let _ = window.set_focusable(true);
        state.game_mode = false;
        return Err(error.to_string());
    }
    state.game_mode = enabled;
    if !enabled {
        state.hidden = false;
    }
    if enabled {
        state.help = false;
    }
    #[cfg(windows)]
    if enabled {
        if let Some(handle) = state.return_focus.take() {
            use windows::Win32::{
                Foundation::HWND,
                UI::WindowsAndMessaging::{IsWindow, SetForegroundWindow},
            };
            let hwnd = HWND(handle as *mut std::ffi::c_void);
            unsafe {
                if IsWindow(Some(hwnd)).as_bool() {
                    let _ = SetForegroundWindow(hwnd);
                }
            }
        }
    }
    drop(state);
    update_panel(app)?;
    refresh_bindings(app)
}

fn update_panel(app: &tauri::AppHandle) -> Result<(), String> {
    let visible = app
        .state::<Shared>()
        .lock()
        .map_err(|e| e.to_string())?
        .help;
    if let Some(overlay) = app.get_webview("settings") {
        layout_overlay(app)?;
        overlay.show().map_err(|e| e.to_string())?;
        if visible {
            overlay.set_focus().map_err(|e| e.to_string())?;
        }
    }
    sync_overlay(app);
    Ok(())
}

fn layout_overlay(app: &tauri::AppHandle) -> Result<(), String> {
    let window = app.get_window("tiktok").ok_or("Fenêtre fermée")?;
    let size = window
        .inner_size()
        .map_err(|e| e.to_string())?
        .to_logical::<f64>(window.scale_factor().map_err(|e| e.to_string())?);
    let open = app
        .state::<Shared>()
        .lock()
        .map_err(|e| e.to_string())?
        .help;
    if let Some(overlay) = app.get_webview("settings") {
        overlay
            .set_size(tauri::LogicalSize::new(
                size.width,
                if open { size.height } else { 36.0 },
            ))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn drag_window(webview: tauri::Webview, app: tauri::AppHandle) -> Result<(), String> {
    authorize(&webview)?;
    {
        let shared = app.state::<Shared>();
        let state = shared.lock().map_err(|e| e.to_string())?;
        if state.game_mode || state.capturing {
            return Err("Passe en interaction pour déplacer la fenêtre.".into());
        }
    }
    if !has_focus(&app) {
        return Err("Active la fenêtre pour la déplacer.".into());
    }
    app.get_window("tiktok")
        .ok_or("Fenêtre fermée")?
        .start_dragging()
        .map_err(|e| e.to_string())
}

fn register_bindings(app: &tauri::AppHandle, bindings: &[Binding]) -> Result<(), String> {
    let shortcuts = bindings
        .iter()
        .map(|b| b.shortcut.parse::<Shortcut>().map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    for (index, shortcut) in shortcuts.iter().enumerate() {
        if let Err(error) = app.global_shortcut().register(*shortcut) {
            for item in &shortcuts[..index] {
                let _ = app.global_shortcut().unregister(*item);
            }
            return Err(format!(
                "Raccourci indisponible ({}) : {error}",
                bindings[index].shortcut
            ));
        }
    }
    Ok(())
}

fn config_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let directory = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    Ok(directory.join("keybinds.json"))
}

fn authorize(webview: &tauri::Webview) -> Result<(), String> {
    let url = webview.url().map_err(|e| e.to_string())?;
    if webview.label() == "settings"
        && (url.scheme() == "tauri" || url.host_str() == Some("tauri.localhost"))
    {
        Ok(())
    } else {
        Err("Accès refusé".into())
    }
}

#[tauri::command]
async fn settings_state(
    webview: tauri::Webview,
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    authorize(&webview)?;
    let shared = app.state::<Shared>();
    let state = shared.lock().map_err(|e| e.to_string())?;
    let mut result = serde_json::to_value(&*state).map_err(|e| e.to_string())?;
    result["defaults"] = serde_json::to_value(bindings::defaults()).map_err(|e| e.to_string())?;
    Ok(result)
}

#[tauri::command]
async fn overlay_action(
    webview: tauri::Webview,
    app: tauri::AppHandle,
    action: String,
) -> Result<(), String> {
    authorize(&webview)?;
    if app
        .state::<Shared>()
        .lock()
        .map_err(|e| e.to_string())?
        .capturing
    {
        return Err("Termine la saisie du raccourci.".into());
    }
    let result = perform(&app, &action);
    if let Ok(mut state) = app.state::<Shared>().lock() {
        state.error = result.as_ref().err().cloned();
    }
    sync_overlay(&app);
    result
}

#[tauri::command]
async fn capture_key(
    webview: tauri::Webview,
    app: tauri::AppHandle,
    active: bool,
) -> Result<(), String> {
    authorize(&webview)?;
    let (game, help, capturing) = {
        let shared = app.state::<Shared>();
        let state = shared.lock().map_err(|e| e.to_string())?;
        (state.game_mode, state.help, state.capturing)
    };
    if active && (game || !help) {
        return Err("Ouvre le panneau en mode interaction.".into());
    }
    if active == capturing {
        return Ok(());
    }
    app.state::<Shared>()
        .lock()
        .map_err(|e| e.to_string())?
        .capturing = active;
    refresh_bindings(&app)
}

#[tauri::command]
async fn save_bindings(
    webview: tauri::Webview,
    app: tauri::AppHandle,
    bindings: Vec<Binding>,
) -> Result<(), String> {
    authorize(&webview)?;
    bindings::validate(&bindings)?;
    let gate = app.state::<BindingGate>();
    let guard = gate.0.lock().map_err(|e| e.to_string())?;
    let old = {
        let shared = app.state::<Shared>();
        let state = shared.lock().map_err(|e| e.to_string())?;
        if state.capturing || state.game_mode || !state.help {
            return Err("Termine la saisie en mode interaction.".into());
        }
        state.bindings.clone()
    };
    let path = config_path(&app)?;
    let temporary = path.with_extension("tmp");
    std::fs::write(
        &temporary,
        serde_json::to_vec_pretty(&bindings).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;
    let result = register_bindings(&app, &bindings)
        .and_then(|_| std::fs::rename(&temporary, &path).map_err(|e| e.to_string()));
    if let Err(error) = result {
        let _ = app.global_shortcut().unregister_all();
        let recovery = register_bindings(&app, &bindings::enabled(&old, false));
        return Err(match recovery {
            Ok(()) => error,
            Err(restore) => {
                format!(
                    "{error}. Restauration impossible : {restore}. Relance BrainrotWhilePlaying."
                )
            }
        });
    }
    app.state::<Shared>()
        .lock()
        .map_err(|e| e.to_string())?
        .bindings = bindings;
    drop(guard);
    refresh_bindings(&app)
}

fn perform(app: &tauri::AppHandle, action: &str) -> Result<(), String> {
    let window = app.get_window("tiktok").ok_or("Fenêtre fermée")?;
    match action {
        "quit" => {
            app.exit(0);
            Ok(())
        }
        "mode" => {
            let game = app
                .state::<Shared>()
                .lock()
                .map_err(|e| e.to_string())?
                .game_mode;
            let enabled = has_focus(app) && !game;
            set_mode(app, enabled)
        }
        "visibility" => {
            let shared = app.state::<Shared>();
            let mut state = shared.lock().map_err(|e| e.to_string())?;
            if state.hidden {
                window.show()
            } else {
                window.hide()
            }
            .map_err(|e| e.to_string())?;
            state.hidden = !state.hidden;
            drop(state);
            refresh_bindings(app)
        }
        "help" => {
            let (open, game, resume) = {
                let shared = app.state::<Shared>();
                let state = shared.lock().map_err(|e| e.to_string())?;
                (state.help, state.game_mode, state.resume_game)
            };
            if !open {
                set_mode(app, false)?;
                let shared = app.state::<Shared>();
                let mut state = shared.lock().map_err(|e| e.to_string())?;
                state.help = true;
                state.resume_game = game;
            } else {
                app.state::<Shared>()
                    .lock()
                    .map_err(|e| e.to_string())?
                    .help = false;
                if resume {
                    set_mode(app, true)?;
                }
            }
            update_panel(app)
        }
        "compact" | "corner" | "size" => {
            let shared = app.state::<Shared>();
            let mut state = shared.lock().map_err(|e| e.to_string())?;
            match action {
                "compact" => state.compact = !state.compact,
                "corner" => state.corner = (state.corner + 1) % 4,
                "size" => state.size = (state.size + 1) % 3,
                _ => unreachable!(),
            }
            drop(state);
            if action == "corner" || action == "size" {
                arrange(app, action == "corner")?;
            }
            Ok(())
        }
        "next" | "previous" | "pause" | "quieter" | "louder" | "mute" => {
            let window = app.get_webview("tiktok").ok_or("Lecteur fermé")?;
            let handle = app.clone();
            window.eval_with_callback(format!("(() => {{ try {{ return ({})(\"{}\") ?? null; }} catch (error) {{ return 'TikTok : ' + error.message; }} }})()", include_str!("player.js"), action), move |result| {
                let error = serde_json::from_str::<Option<String>>(&result).unwrap_or_else(|_| Some("Réponse du lecteur invalide".into()));
                if let Ok(mut state) = handle.state::<Shared>().lock() { state.error = error; }
                sync_overlay(&handle);
            }).map_err(|e| e.to_string())
        }
        _ => Err("Commande inconnue".into()),
    }
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(Status::default()))
        .manage(BindingGate(Mutex::new(())))
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Resized(_)) {
                let app = window.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    let _ = layout_overlay(&app);
                });
            }
        })
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let app = app.clone();
                    let shortcut = *shortcut;
                    tauri::async_runtime::spawn(async move {
                        let action = {
                            let shared = app.state::<Shared>();
                            let Ok(state) = shared.lock() else {
                                return;
                            };
                            if state.capturing {
                                return;
                            }
                            state
                                .bindings
                                .iter()
                                .find(|b| {
                                    b.shortcut
                                        .parse::<Shortcut>()
                                        .is_ok_and(|s| s.id() == shortcut.id())
                                })
                                .map(|b| b.action.clone())
                        };
                        let Some(action) = action else {
                            return;
                        };
                        let result = perform(&app, &action);
                        if let Ok(mut state) = app.state::<Shared>().lock() {
                            state.error = result.err();
                        }
                        sync_overlay(&app);
                    });
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            settings_state,
            overlay_action,
            capture_key,
            save_bindings,
            drag_window
        ])
        .setup(|app| {
            let path = config_path(app.handle()).map_err(std::io::Error::other)?;
            if path.exists() {
                match std::fs::read(&path)
                    .map_err(|e| e.to_string())
                    .and_then(|bytes| {
                        serde_json::from_slice::<Vec<Binding>>(&bytes).map_err(|e| e.to_string())
                    })
                    .and_then(|bindings| {
                        bindings::validate(&bindings)?;
                        Ok(bindings)
                    }) {
                    Ok(bindings) => app.state::<Shared>().lock().unwrap().bindings = bindings,
                    Err(error) => {
                        app.state::<Shared>().lock().unwrap().error =
                            Some(format!("Réglages illisibles, touches par défaut : {error}"))
                    }
                }
            }
            WebviewWindowBuilder::new(
                app,
                "tiktok",
                WebviewUrl::External("https://www.tiktok.com/foryou".parse()?),
            )
            .title("BrainrotWhilePlaying")
            .icon(tauri::include_image!("icons/128x128.png"))?
            .inner_size(340.0, 604.0)
            .min_inner_size(160.0, 280.0)
            .decorations(false)
            .shadow(true)
            .always_on_top(true)
            .initialization_script(include_str!("overlay.js"))
            .on_page_load(|window, _| {
                sync_overlay(window.app_handle());
            })
            .on_navigation(|url| url.scheme() == "https")
            .build()?;
            let parent = app.get_window("tiktok").ok_or("Fenêtre absente")?;
            #[cfg(windows)]
            unsafe {
                use windows::Win32::{
                    Foundation::{HWND, LPARAM, WPARAM},
                    UI::WindowsAndMessaging::{
                        SendMessageW, ICON_BIG, ICON_SMALL, WM_GETICON, WM_SETICON,
                    },
                };
                let hwnd = HWND(parent.hwnd()?.0);
                let icon = SendMessageW(hwnd, WM_GETICON, Some(WPARAM(ICON_SMALL as usize)), None);
                // Windows uses the large window icon for the taskbar; Tauri sets only the small one.
                if icon.0 != 0 {
                    SendMessageW(
                        hwnd,
                        WM_SETICON,
                        Some(WPARAM(ICON_BIG as usize)),
                        Some(LPARAM(icon.0)),
                    );
                }
            }
            app.get_webview("tiktok")
                .ok_or("Lecteur absent")?
                .set_auto_resize(true)?;
            parent.add_child(
                tauri::webview::WebviewBuilder::new(
                    "settings",
                    WebviewUrl::App("index.html".into()),
                )
                .transparent(true)
                .on_navigation(|url| {
                    url.scheme() == "tauri" || url.host_str() == Some("tauri.localhost")
                }),
                tauri::LogicalPosition::new(0.0, 0.0),
                parent.inner_size()?,
            )?;
            arrange(app.handle(), true).map_err(std::io::Error::other)?;
            layout_overlay(app.handle()).map_err(std::io::Error::other)?;
            let bindings = app.state::<Shared>().lock().unwrap().bindings.clone();
            if let Err(error) =
                register_bindings(app.handle(), &bindings::enabled(&bindings, false))
            {
                let defaults = bindings::defaults();
                app.state::<Shared>().lock().unwrap().bindings = defaults.clone();
                app.state::<Shared>().lock().unwrap().error = Some(error);
                register_bindings(app.handle(), &bindings::enabled(&defaults, false))
                    .map_err(std::io::Error::other)?;
            }
            app.state::<Shared>().lock().unwrap().ready = true;
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = refresh_bindings(&handle);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Impossible de démarrer BrainrotWhilePlaying");
}
