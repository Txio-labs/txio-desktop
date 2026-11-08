//! Self-update from GitHub Releases. The release workflow uploads signed update
//! packages and `latest.json`; the public key in `tauri.conf.json` verifies them.

use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;

/// Look for an update. On launch (`user_asked == false`) stay quiet unless one exists.
pub fn check(app: AppHandle, user_asked: bool) {
    tauri::async_runtime::spawn(async move {
        let updater = match app.updater() {
            Ok(u) => u,
            Err(e) => return report(&app, user_asked, &format!("Updates are unavailable: {e}")),
        };
        let update = match updater.check().await {
            Ok(Some(update)) => update,
            Ok(None) => {
                if user_asked {
                    app.dialog()
                        .message(format!(
                            "Txio {} is the latest version.",
                            app.package_info().version
                        ))
                        .title("No update available")
                        .kind(MessageDialogKind::Info)
                        .show(|_| {});
                }
                return;
            }
            Err(e) => {
                return report(
                    &app,
                    user_asked,
                    &format!("Could not check for updates: {e}"),
                )
            }
        };

        let notes = update.body.clone().unwrap_or_default();
        let summary = notes
            .lines()
            .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .unwrap_or("");
        let (tx, rx) = tokio::sync::oneshot::channel();
        app.dialog()
            .message(format!(
                "Txio {} is available (you have {}).\n\n{}\n\nInstall it now? Txio restarts when the update is in place.",
                update.version, update.current_version, summary
            ))
            .title("Update available")
            .kind(MessageDialogKind::Info)
            .buttons(MessageDialogButtons::OkCancelCustom("Install and restart".into(), "Later".into()))
            .show(move |ok| {
                let _ = tx.send(ok);
            });
        if !rx.await.unwrap_or(false) {
            return;
        }
        match update.download_and_install(|_, _| {}, || {}).await {
            Ok(()) => app.restart(),
            Err(e) => report(
                &app,
                true,
                &format!("The update could not be installed: {e}"),
            ),
        }
    });
}

fn report(app: &AppHandle, show: bool, msg: &str) {
    eprintln!("{msg}");
    if show {
        app.dialog()
            .message(msg)
            .title("Txio update")
            .kind(MessageDialogKind::Warning)
            .show(|_| {});
    }
}
