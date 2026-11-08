//! Which URLs the main window may load.
//!
//! - The app origin (and the dev server in `tauri dev`) loads normally.
//! - `api.txio.xyz` and `txio.xyz` are rewritten onto the app origin. This is how OAuth
//!   finishes inside the app: the provider redirects to the API callback, which goes
//!   through the shell's proxy and cookie jar, then on to the app page.
//! - Sign-in pages of the OAuth providers the API supports load in the window.
//! - Every other http(s) link opens in the system browser.

use url::Url;

/// Hosts that serve an OAuth sign-in page the window must be allowed to show.
const OAUTH_HOSTS: &[&str] = &[
    "accounts.google.com",
    "accounts.youtube.com",
    "github.com",
    "x.com",
    "twitter.com",
    "api.x.com",
    "api.twitter.com",
];

pub fn is_api_host(u: &Url) -> bool {
    let api = Url::parse(crate::proxy::API_ORIGIN).ok();
    u.scheme() == "https"
        && api
            .as_ref()
            .map(|a| {
                a.host_str() == u.host_str()
                    && a.port_or_known_default() == u.port_or_known_default()
            })
            .unwrap_or(false)
}

pub fn is_site_host(u: &Url) -> bool {
    u.scheme() == "https" && matches!(u.host_str(), Some("txio.xyz") | Some("www.txio.xyz"))
}

fn is_oauth_host(u: &Url) -> bool {
    u.scheme() == "https"
        && u.host_str()
            .map(|h| {
                OAUTH_HOSTS
                    .iter()
                    .any(|o| h == *o || h.ends_with(&format!(".{o}")))
            })
            .unwrap_or(false)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Allow,
    /// Load this path (with query and fragment) on the app origin instead.
    Rewrite(String),
    External,
    Block,
}

/// Custom schemes such as `txio://` have opaque origins in the `url` crate, so compare
/// scheme, host and port directly.
fn same_origin(a: &Url, b: &Url) -> bool {
    a.scheme() == b.scheme()
        && a.host_str() == b.host_str()
        && a.port_or_known_default() == b.port_or_known_default()
}

pub fn decide(u: &Url, app_origin: &Url) -> Decision {
    if same_origin(u, app_origin) {
        return Decision::Allow;
    }
    match u.scheme() {
        "about" | "data" | "blob" => return Decision::Allow,
        "http" | "https" => {}
        "mailto" => return Decision::External,
        _ => return Decision::Block,
    }
    if is_api_host(u) || is_site_host(u) {
        let mut s = u.path().to_string();
        if let Some(q) = u.query() {
            s.push('?');
            s.push_str(q);
        }
        if let Some(f) = u.fragment() {
            s.push('#');
            s.push_str(f);
        }
        return Decision::Rewrite(s);
    }
    if is_oauth_host(u) {
        return Decision::Allow;
    }
    Decision::External
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> Url {
        Url::parse("txio://localhost/").unwrap()
    }

    #[test]
    fn app_pages_load() {
        assert_eq!(
            decide(&Url::parse("txio://localhost/workspace").unwrap(), &app()),
            Decision::Allow
        );
    }

    #[test]
    fn api_and_site_stay_in_the_app() {
        assert_eq!(
            decide(
                &Url::parse("https://api.txio.xyz/api/v1/auth/google/callback?code=1&state=2")
                    .unwrap(),
                &app()
            ),
            Decision::Rewrite("/api/v1/auth/google/callback?code=1&state=2".into())
        );
        assert_eq!(
            decide(&Url::parse("https://txio.xyz/workspace").unwrap(), &app()),
            Decision::Rewrite("/workspace".into())
        );
    }

    #[test]
    fn oauth_loads_and_others_open_outside() {
        assert_eq!(
            decide(
                &Url::parse("https://accounts.google.com/o/oauth2/v2/auth").unwrap(),
                &app()
            ),
            Decision::Allow
        );
        assert_eq!(
            decide(
                &Url::parse("https://github.com/login/oauth/authorize").unwrap(),
                &app()
            ),
            Decision::Allow
        );
        assert_eq!(
            decide(&Url::parse("https://etherscan.io/tx/0x1").unwrap(), &app()),
            Decision::External
        );
        assert_eq!(
            decide(&Url::parse("file:///etc/passwd").unwrap(), &app()),
            Decision::Block
        );
        assert_eq!(
            decide(
                &Url::parse("https://evil.github.com.attacker.io/").unwrap(),
                &app()
            ),
            Decision::External
        );
    }
}
