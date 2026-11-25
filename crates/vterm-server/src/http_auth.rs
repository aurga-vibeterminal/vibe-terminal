use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use rand::RngCore;

pub(crate) const SETTINGS_SESSION_COOKIE: &str = "vterm_settings_session";
const TOKEN_LENGTH: usize = 48;

struct Grant {
    owner: String,
    expires_at: Option<Instant>,
    single_use: bool,
}

#[derive(Default)]
struct AccessState {
    grants: HashMap<String, Grant>,
    sessions: HashMap<String, (String, Instant)>,
}

pub(crate) struct HttpAccess {
    cookie_name: String,
    session_ttl: Duration,
    state: Mutex<AccessState>,
}

impl HttpAccess {
    pub(crate) fn new(cookie_name: String, session_ttl: Duration) -> Self {
        Self {
            cookie_name,
            session_ttl,
            state: Mutex::new(AccessState::default()),
        }
    }

    pub(crate) fn new_static(session_ttl: Duration) -> (Self, String) {
        let service_id = random_token(16);
        let access = Self::new(format!("vterm_static_{service_id}"), session_ttl);
        let share_token = access.issue_grant("static-share", None, false);
        (access, share_token)
    }

    pub(crate) fn cookie_name(&self) -> &str {
        &self.cookie_name
    }

    pub(crate) fn issue_grant(
        &self,
        owner: &str,
        ttl: Option<Duration>,
        single_use: bool,
    ) -> String {
        let token = random_token(TOKEN_LENGTH);
        let expires_at = ttl.map(|ttl| Instant::now() + ttl);
        self.state.lock().unwrap().grants.insert(
            token.clone(),
            Grant {
                owner: owner.to_string(),
                expires_at,
                single_use,
            },
        );
        token
    }

    pub(crate) fn exchange_grant(&self, token: &str) -> Option<String> {
        let now = Instant::now();
        let mut state = self.state.lock().unwrap();
        state
            .grants
            .retain(|_, grant| grant.expires_at.is_none_or(|expires| expires > now));
        state.sessions.retain(|_, (_, expires)| *expires > now);
        let grant = state.grants.get(token)?;
        let owner = grant.owner.clone();
        if grant.single_use {
            state.grants.remove(token);
        }
        let session = random_token(TOKEN_LENGTH);
        state
            .sessions
            .insert(session.clone(), (owner, now + self.session_ttl));
        Some(session)
    }

    pub(crate) fn validate_session(&self, session: &str) -> bool {
        let now = Instant::now();
        let mut state = self.state.lock().unwrap();
        let valid = state
            .sessions
            .get(session)
            .is_some_and(|(_, expires)| *expires > now);
        if !valid {
            state.sessions.remove(session);
        }
        valid
    }

    pub(crate) fn has_valid_cookie(&self, cookie_header: Option<&str>) -> bool {
        cookie_header
            .and_then(|header| cookie_value(header, &self.cookie_name))
            .is_some_and(|session| self.validate_session(&session))
    }

    pub(crate) fn session_cookie(&self, session: &str, secure: bool) -> String {
        let secure = if secure { "; Secure" } else { "" };
        format!(
            "{}={session}{secure}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}",
            self.cookie_name,
            self.session_ttl.as_secs()
        )
    }

    pub(crate) fn revoke_owner(&self, owner: &str) {
        let mut state = self.state.lock().unwrap();
        state.grants.retain(|_, grant| grant.owner != owner);
        state
            .sessions
            .retain(|_, (session_owner, _)| session_owner != owner);
    }
}

pub(crate) fn settings_access() -> &'static HttpAccess {
    static ACCESS: OnceLock<HttpAccess> = OnceLock::new();
    ACCESS.get_or_init(|| {
        HttpAccess::new(
            SETTINGS_SESSION_COOKIE.to_string(),
            Duration::from_secs(12 * 60 * 60),
        )
    })
}

pub(crate) fn issue_settings_ticket(owner: &str) -> String {
    settings_access().issue_grant(owner, Some(Duration::from_secs(60)), true)
}

pub(crate) fn cookie_value(header: &str, name: &str) -> Option<String> {
    header.split(';').find_map(|part| {
        let (key, value) = part.trim().split_once('=')?;
        (key == name).then(|| value.to_string())
    })
}

pub(crate) fn query_param(raw_path: &str, name: &str) -> Option<String> {
    let query = raw_path.split_once('?')?.1;
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        (percent_decode(key) == name).then(|| percent_decode(value))
    })
}

pub(crate) fn without_query_param(raw_path: &str, name: &str) -> String {
    let (path, query) = raw_path.split_once('?').unwrap_or((raw_path, ""));
    let query = query
        .split('&')
        .filter(|pair| {
            let key = pair.split_once('=').map_or(*pair, |(key, _)| key);
            percent_decode(key) != name
        })
        .collect::<Vec<_>>()
        .join("&");
    if query.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{query}")
    }
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) = (hex(bytes[index + 1]), hex(bytes[index + 2])) {
                decoded.push((high << 4) | low);
                index += 3;
                continue;
            }
        }
        decoded.push(if bytes[index] == b'+' {
            b' '
        } else {
            bytes[index]
        });
        index += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn random_token(length: usize) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut bytes = vec![0u8; length];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes
        .iter()
        .map(|byte| CHARS[usize::from(*byte) % CHARS.len()] as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_time_ticket_cannot_be_replayed() {
        let access = HttpAccess::new("test".to_string(), Duration::from_secs(60));
        let ticket = access.issue_grant("client", Some(Duration::from_secs(60)), true);
        let session = access.exchange_grant(&ticket).unwrap();
        assert!(access.validate_session(&session));
        assert!(access.exchange_grant(&ticket).is_none());
    }

    #[test]
    fn share_token_exchanges_independent_sessions() {
        let (access, share_token) = HttpAccess::new_static(Duration::from_secs(60));
        let first = access.exchange_grant(&share_token).unwrap();
        let second = access.exchange_grant(&share_token).unwrap();
        assert_ne!(first, second);
        assert!(access.validate_session(&first));
        assert!(access.validate_session(&second));
    }

    #[test]
    fn sessions_are_isolated_between_static_services() {
        let (first, first_token) = HttpAccess::new_static(Duration::from_secs(60));
        let (second, _) = HttpAccess::new_static(Duration::from_secs(60));
        let session = first.exchange_grant(&first_token).unwrap();
        assert!(first.validate_session(&session));
        assert!(!second.validate_session(&session));
        assert_ne!(first.cookie_name(), second.cookie_name());
    }

    #[test]
    fn expired_grants_and_sessions_are_rejected() {
        let access = HttpAccess::new("test".to_string(), Duration::ZERO);
        let ticket = access.issue_grant("client", Some(Duration::ZERO), true);
        assert!(access.exchange_grant(&ticket).is_none());
        let ticket = access.issue_grant("client", None, true);
        let session = access.exchange_grant(&ticket).unwrap();
        assert!(!access.validate_session(&session));
    }

    #[test]
    fn owner_revocation_clears_grants_and_sessions() {
        let access = HttpAccess::new("test".to_string(), Duration::from_secs(60));
        let ticket = access.issue_grant("client", None, true);
        let session = access.exchange_grant(&ticket).unwrap();
        let pending = access.issue_grant("client", None, true);
        access.revoke_owner("client");
        assert!(!access.validate_session(&session));
        assert!(access.exchange_grant(&pending).is_none());
    }

    #[test]
    fn cookies_and_redirects_are_scoped_and_preserve_other_params() {
        let header = "other=abc; test=session-id; theme=dark";
        assert_eq!(cookie_value(header, "test").as_deref(), Some("session-id"));
        assert_eq!(
            query_param("/?a=1&ticket=secret&b=2", "ticket").as_deref(),
            Some("secret")
        );
        assert_eq!(
            without_query_param("/?a=1&ticket=secret&b=2", "ticket"),
            "/?a=1&b=2"
        );
        assert_eq!(without_query_param("/?ticket=secret", "ticket"), "/");
    }
}
