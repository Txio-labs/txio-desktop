//! `txio-desktop://` links: how a browser sign-in comes back to the app.
//!
//! The website (`/desktop-auth`) ends by opening
//! `txio-desktop://auth?code=…&state=…`. The OS hands that link to this app (to the
//! running instance through the single-instance plugin, or at launch). The shell only
//! moves it onto `/signin#desktop_code=…&state=…`; the page checks `state` and redeems
//! the code with its PKCE verifier through the API proxy, so the session cookie lands
//! in the shell's jar like any other login.

use tauri::AppHandle;
use url::Url;

pub const SCHEME: &str = "txio-desktop";

/// The app path to open for a link, or `None` for links this app does not handle.
pub fn target_path(link: &Url) -> Option<String> {
    if link.scheme() != SCHEME || link.host_str() != Some("auth") {
        return None;
    }
    let mut code = None;
    let mut state = None;
    for (k, v) in link.query_pairs() {
        match k.as_ref() {
            "code" => code = Some(v.into_owned()),
            "state" => state = Some(v.into_owned()),
            _ => {}
        }
    }
    let (code, state) = (code?, state?);
    // Codes and states are base64url; anything else is not ours and never reaches the page.
    if !is_token(&code, 43, 43) || !is_token(&state, 16, 128) {
        return None;
    }
    Some(format!("/signin#desktop_code={code}&state={state}"))
}

fn is_token(s: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&s.len())
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

pub fn handle(app: &AppHandle, links: &[Url]) {
    for link in links {
        if let Some(path) = target_path(link) {
            crate::go(app, &path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code() -> String {
        "c".repeat(43)
    }

    #[test]
    fn auth_link_becomes_signin_fragment() {
        let link = Url::parse(&format!(
            "txio-desktop://auth?code={}&state=abcdefghijklmnop",
            code()
        ))
        .unwrap();
        assert_eq!(
            target_path(&link).as_deref(),
            Some(format!("/signin#desktop_code={}&state=abcdefghijklmnop", code()).as_str())
        );
    }

    #[test]
    fn other_links_are_ignored() {
        for l in [
            "txio-desktop://other?code=x&state=y".to_string(),
            "https://auth?code=x&state=y".to_string(),
            format!("txio-desktop://auth?code={}", code()),
            format!("txio-desktop://auth?code={}&state=short", code()),
            format!(
                "txio-desktop://auth?code={}%26x%3D1&state=abcdefghijklmnop",
                code()
            ),
        ] {
            assert_eq!(target_path(&Url::parse(&l).unwrap()), None, "{l}");
        }
    }
}
