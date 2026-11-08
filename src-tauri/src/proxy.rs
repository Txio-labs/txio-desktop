//! Serves the bundled web app and the Txio API from one origin.
//!
//! The web app signs in with an HttpOnly session cookie set by `api.txio.xyz`. A page
//! loaded from the app bundle (`txio://localhost`) is cross-site to that API, so the
//! webview would drop the cookie (WebKit blocks third-party cookies outright). Instead
//! the web app is built with `NEXT_PUBLIC_API_URL=/api/v1` and every `/api/*` request on
//! the app origin is forwarded here, server to server, with the session kept in a cookie
//! jar owned by the shell. The jar is saved to the app data folder (owner-only on Unix),
//! the same trust boundary as a browser profile, and never reaches page script.

use std::fs;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest_cookie_store::CookieStoreMutex;
use tauri::http::{Request, Response, StatusCode};
use tauri::{AppHandle, Runtime};

/// The scheme the bundled app is served on: `txio://localhost` on macOS and Linux,
/// `http://txio.localhost` on Windows (WebView2 maps custom schemes that way).
pub const SCHEME: &str = "txio";

/// Where `/api/*` requests go. Set `TXIO_API_ORIGIN` at build time to point a build at
/// a staging backend.
pub const API_ORIGIN: &str = match option_env!("TXIO_API_ORIGIN") {
    Some(origin) => origin,
    None => "https://api.txio.xyz",
};

/// Request headers the shell sets itself or that must not leak the webview's state.
const DROP_REQUEST: &[&str] = &[
    "host",
    "origin",
    "referer",
    "cookie",
    "connection",
    "content-length",
    "accept-encoding",
    "transfer-encoding",
];

/// Response headers the webview must not act on: cookies stay in the shell's jar, and
/// the body is already decoded and re-framed.
const DROP_RESPONSE: &[&str] = &[
    "set-cookie",
    "content-encoding",
    "content-length",
    "transfer-encoding",
    "connection",
    "access-control-allow-origin",
    "access-control-allow-credentials",
];

pub struct Proxy {
    client: reqwest::Client,
    jar: Arc<CookieStoreMutex>,
    jar_path: Option<PathBuf>,
}

impl Proxy {
    pub fn new(data_dir: Option<PathBuf>) -> Self {
        let jar_path = data_dir.map(|d| d.join("session.json"));
        let store = jar_path
            .as_ref()
            .and_then(|p| fs::File::open(p).ok())
            .and_then(|f| cookie_store::serde::json::load(BufReader::new(f)).ok())
            .unwrap_or_default();
        let jar = Arc::new(CookieStoreMutex::new(store));
        let client = reqwest::Client::builder()
            .cookie_provider(jar.clone())
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("Txio-Desktop/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(std::time::Duration::from_secs(15))
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("HTTP client");
        Self {
            client,
            jar,
            jar_path,
        }
    }

    fn save_jar(&self) {
        let Some(path) = &self.jar_path else { return };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let mut buf = Vec::new();
        {
            let store = self.jar.lock().unwrap();
            if cookie_store::serde::json::save(&store, &mut buf).is_err() {
                return;
            }
        }
        let tmp = path.with_extension("json.tmp");
        if fs::write(&tmp, &buf).is_ok() {
            restrict(&tmp);
            let _ = fs::rename(&tmp, path);
        }
    }

    /// Forward one `/api/*` request to the API and hand back its response.
    pub async fn forward(&self, req: Request<Vec<u8>>) -> Response<Vec<u8>> {
        let path_and_query = req
            .uri()
            .path_and_query()
            .map(|p| p.as_str())
            .unwrap_or("/");
        let url = format!("{API_ORIGIN}{path_and_query}");
        let method = match reqwest::Method::from_bytes(req.method().as_str().as_bytes()) {
            Ok(m) => m,
            Err(_) => return plain(StatusCode::METHOD_NOT_ALLOWED, "Unsupported method"),
        };

        let mut headers = HeaderMap::new();
        for (name, value) in req.headers() {
            if DROP_REQUEST.contains(&name.as_str()) {
                continue;
            }
            if let (Ok(n), Ok(v)) = (
                HeaderName::from_bytes(name.as_str().as_bytes()),
                HeaderValue::from_bytes(value.as_bytes()),
            ) {
                headers.append(n, v);
            }
        }

        let body = req.into_body();
        let mut builder = self.client.request(method, &url).headers(headers);
        if !body.is_empty() {
            builder = builder.body(body);
        }

        let upstream = match builder.send().await {
            Ok(r) => r,
            Err(e) => {
                let msg = if e.is_timeout() {
                    "The Txio API took too long to answer."
                } else {
                    "Could not reach the Txio API. Check your connection."
                };
                return json_error(StatusCode::BAD_GATEWAY, msg);
            }
        };

        let status = upstream.status();
        let set_cookie = upstream.headers().contains_key("set-cookie");
        let mut out = Response::builder().status(status.as_u16());
        for (name, value) in upstream.headers() {
            if DROP_RESPONSE.contains(&name.as_str()) {
                continue;
            }
            // Redirects back to the API or the website stay inside the app origin.
            if name == "location" {
                if let Ok(loc) = value.to_str() {
                    out = out.header("location", rewrite_location(loc));
                    continue;
                }
            }
            out = out.header(name.as_str(), value.as_bytes());
        }
        let bytes = match upstream.bytes().await {
            Ok(b) => b.to_vec(),
            Err(_) => {
                return json_error(
                    StatusCode::BAD_GATEWAY,
                    "The Txio API response was cut off.",
                )
            }
        };
        if set_cookie {
            self.save_jar();
        }
        out.body(bytes)
            .unwrap_or_else(|_| plain(StatusCode::INTERNAL_SERVER_ERROR, "Bad response"))
    }
}

/// `https://api.txio.xyz/x` and `https://txio.xyz/x` become `/x` on the app origin.
fn rewrite_location(loc: &str) -> String {
    match url::Url::parse(loc) {
        Ok(u) if crate::nav::is_api_host(&u) || crate::nav::is_site_host(&u) => {
            let mut s = u.path().to_string();
            if let Some(q) = u.query() {
                s.push('?');
                s.push_str(q);
            }
            if let Some(f) = u.fragment() {
                s.push('#');
                s.push_str(f);
            }
            s
        }
        _ => loc.to_string(),
    }
}

#[cfg(unix)]
fn restrict(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}
#[cfg(not(unix))]
fn restrict(_path: &std::path::Path) {}

fn plain(status: StatusCode, msg: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header("content-type", "text/plain; charset=utf-8")
        .body(msg.as_bytes().to_vec())
        .unwrap()
}

fn json_error(status: StatusCode, msg: &str) -> Response<Vec<u8>> {
    let body = serde_json::json!({ "error": msg, "message": msg }).to_string();
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(body.into_bytes())
        .unwrap()
}

/// Serve a file from the embedded web app. Next.js' static export writes `/docs` as
/// `docs.html`, so try the exact path, then `.html`, then `/index.html`, then the 404 page.
pub fn asset<R: Runtime>(app: &AppHandle<R>, req: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let resolver = app.asset_resolver();
    let raw = req.uri().path();
    let path = percent_decode(raw.trim_start_matches('/'));
    let path = path.trim_end_matches('/');
    if path.split('/').any(|seg| seg == "..") {
        return plain(StatusCode::BAD_REQUEST, "Bad path");
    }
    let candidates: Vec<String> = if path.is_empty() {
        vec!["index.html".into()]
    } else {
        vec![
            path.to_string(),
            format!("{path}.html"),
            format!("{path}/index.html"),
        ]
    };
    let (found, status) = candidates
        .iter()
        .find_map(|c| resolver.get(c.clone()).map(|a| (a, StatusCode::OK)))
        .or_else(|| {
            resolver
                .get("404.html".into())
                .map(|a| (a, StatusCode::NOT_FOUND))
        })
        .map(|(a, s)| (Some(a), s))
        .unwrap_or((None, StatusCode::NOT_FOUND));
    let Some(asset) = found else {
        return plain(StatusCode::NOT_FOUND, "Not found");
    };
    let mut res = Response::builder()
        .status(status)
        .header("content-type", asset.mime_type())
        .header("x-content-type-options", "nosniff")
        .header("referrer-policy", "strict-origin-when-cross-origin");
    if let Some(csp) = asset.csp_header() {
        res = res.header("content-security-policy", csp);
    }
    res.body(asset.bytes().to_vec()).unwrap()
}

fn percent_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_api_and_site_redirects_into_the_app() {
        assert_eq!(
            rewrite_location("https://api.txio.xyz/api/v1/auth/me?x=1"),
            "/api/v1/auth/me?x=1"
        );
        assert_eq!(
            rewrite_location("https://txio.xyz/workspace#a"),
            "/workspace#a"
        );
        assert_eq!(rewrite_location("https://www.txio.xyz/"), "/");
        assert_eq!(
            rewrite_location("https://accounts.google.com/o/oauth2/auth?a=b"),
            "https://accounts.google.com/o/oauth2/auth?a=b"
        );
        assert_eq!(rewrite_location("/relative"), "/relative");
    }

    #[test]
    fn decodes_paths() {
        assert_eq!(percent_decode("a%20b"), "a b");
        assert_eq!(percent_decode("_next/static/x.js"), "_next/static/x.js");
    }
}
