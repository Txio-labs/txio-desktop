mod nav;
mod proxy;
mod updates;

use std::sync::Arc;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;
use url::Url;

/// The origin the bundled app is served from. In `tauri dev` it is the Next.js dev server.
fn app_origin() -> Url {
    if cfg!(debug_assertions) {
        return Url::parse("http://localhost:3000/").unwrap();
    }
    let s = if cfg!(windows) {
        format!("http://{}.localhost/", proxy::SCHEME)
    } else {
        format!("{}://localhost/", proxy::SCHEME)
    };
    Url::parse(&s).unwrap()
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn go(app: &AppHandle, path: &str) {
    show_main(app);
    if let (Some(w), Ok(url)) = (app.get_webview_window("main"), app_origin().join(path)) {
        let _ = w.navigate(url);
    }
}

fn open_external(app: &AppHandle, url: &str) {
    let _ = app.opener().open_url(url, None::<&str>);
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let txio = Submenu::with_items(
        app,
        "Txio",
        true,
        &[
            &PredefinedMenuItem::about(app, Some("About Txio"), None)?,
            &MenuItem::with_id(app, "updates", "Check for Updates…", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, None)?,
            &PredefinedMenuItem::hide_others(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", "Quit Txio", true, Some("CmdOrCtrl+Q"))?,
        ],
    )?;
    let edit = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;
    let go_menu = Submenu::with_items(
        app,
        "Go",
        true,
        &[
            &MenuItem::with_id(
                app,
                "nav:/workspace",
                "Workspace",
                true,
                Some("CmdOrCtrl+1"),
            )?,
            &MenuItem::with_id(app, "nav:/docs", "Documentation", true, Some("CmdOrCtrl+2"))?,
            &MenuItem::with_id(app, "nav:/", "Home", true, Some("CmdOrCtrl+0"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "reload", "Reload", true, Some("CmdOrCtrl+R"))?,
        ],
    )?;
    let window = Submenu::with_items(
        app,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::maximize(app, None)?,
            &PredefinedMenuItem::fullscreen(app, None)?,
        ],
    )?;
    let help = Submenu::with_items(
        app,
        "Help",
        true,
        &[
            &MenuItem::with_id(app, "web:https://txio.xyz", "txio.xyz", true, None::<&str>)?,
            &MenuItem::with_id(
                app,
                "web:https://github.com/Txio-labs/txio-desktop/issues/new",
                "Report an Issue",
                true,
                None::<&str>,
            )?,
        ],
    )?;
    Menu::with_items(app, &[&txio, &edit, &go_menu, &window, &help])
}

fn on_menu(app: &AppHandle, id: &str) {
    match id {
        "quit" => app.exit(0),
        "updates" => updates::check(app.clone(), true),
        "reload" => {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.eval("window.location.reload()");
            }
        }
        "tray:show" => show_main(app),
        other if other.starts_with("nav:") => go(app, &other[4..]),
        other if other.starts_with("web:") => open_external(app, &other[4..]),
        _ => {}
    }
}

fn create_main_window(app: &AppHandle) -> tauri::Result<()> {
    let origin = app_origin();
    let url = if cfg!(debug_assertions) {
        WebviewUrl::App("/".into())
    } else {
        WebviewUrl::CustomProtocol(origin.clone())
    };

    let nav_app = app.clone();
    let nav_origin = origin.clone();
    let popup_app = app.clone();

    WebviewWindowBuilder::new(app, "main", url)
        .title("Txio")
        .inner_size(1440.0, 900.0)
        .min_inner_size(1024.0, 640.0)
        .center()
        .resizable(true)
        .background_color(tauri::window::Color(10, 10, 10, 255))
        .on_navigation(move |u| match nav::decide(u, &nav_origin) {
            nav::Decision::Allow => true,
            nav::Decision::Rewrite(path) => {
                // Navigating from inside the navigation callback can deadlock on some
                // platforms, so hop off it first.
                let app = nav_app.clone();
                let target = nav_origin.join(&path).ok();
                tauri::async_runtime::spawn(async move {
                    if let (Some(w), Some(t)) = (app.get_webview_window("main"), target) {
                        let _ = w.navigate(t);
                    }
                });
                false
            }
            nav::Decision::External => {
                open_external(&nav_app, u.as_str());
                false
            }
            nav::Decision::Block => false,
        })
        .on_new_window(move |u, _features| {
            // target=_blank and window.open: same rules, never a second webview.
            match nav::decide(&u, &app_origin()) {
                nav::Decision::Rewrite(path) => go(&popup_app, &path),
                nav::Decision::Allow if u.scheme() == "https" => {
                    open_external(&popup_app, u.as_str())
                }
                nav::Decision::External => open_external(&popup_app, u.as_str()),
                _ => {}
            }
            NewWindowResponse::Deny
        })
        .build()?;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            show_main(app)
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .register_asynchronous_uri_scheme_protocol(proxy::SCHEME, |ctx, request, responder| {
            let app = ctx.app_handle().clone();
            if request.uri().path().starts_with("/api/") {
                let proxy = app.state::<Arc<proxy::Proxy>>().inner().clone();
                tauri::async_runtime::spawn(async move {
                    responder.respond(proxy.forward(request).await);
                });
            } else {
                responder.respond(proxy::asset(&app, &request));
            }
        })
        .setup(|app| {
            let handle = app.handle().clone();
            let data_dir = handle.path().app_data_dir().ok();
            app.manage(Arc::new(proxy::Proxy::new(data_dir)));

            // Linux and Windows: no menu bar, like any web app. macOS keeps a menu
            // because its copy/paste shortcuts depend on one.
            if cfg!(target_os = "macos") {
                let menu = build_menu(&handle)?;
                app.set_menu(menu)?;
                app.on_menu_event(|app, event| on_menu(app, event.id().as_ref()));
            }

            create_main_window(&handle)?;

            let tray_menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "tray:show", "Show Txio", true, None::<&str>)?,
                    &MenuItem::with_id(
                        app,
                        "nav:/workspace",
                        "Open Workspace",
                        true,
                        None::<&str>,
                    )?,
                    &MenuItem::with_id(app, "updates", "Check for Updates", true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                    &MenuItem::with_id(app, "quit", "Quit Txio", true, None::<&str>)?,
                ],
            )?;
            let mut tray = TrayIconBuilder::with_id("main")
                .tooltip("Txio")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| on_menu(app, event.id().as_ref()))
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        show_main(tray.app_handle());
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            // A missing tray (no StatusNotifier host on some Linux desktops) must not stop the app.
            let _ = tray.build(app);

            if !cfg!(debug_assertions) {
                updates::check(handle, false);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Txio");
}
