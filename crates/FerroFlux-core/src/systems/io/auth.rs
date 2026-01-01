use crate::components::AuthConfig;
use base64::{Engine as _, engine::general_purpose};
use std::env;

pub fn resolve_auth_headers(auth_config: &AuthConfig) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    match auth_config {
        AuthConfig::Basic { user_env, pass_env } => {
            if let (Ok(u), Ok(p)) = (env::var(user_env), env::var(pass_env)) {
                let plain = format!("{}:{}", u, p);
                let encoded = general_purpose::STANDARD.encode(plain);
                headers.push(("Authorization".to_string(), format!("Basic {}", encoded)));
            }
        }
        AuthConfig::ApiKey {
            key_env,
            header,
            query: _,
        } => {
            if let Ok(key_val) = env::var(key_env)
                && let Some(h) = header
            {
                headers.push((h.clone(), key_val));
            }
        }
        AuthConfig::Bearer { token_env }
        | AuthConfig::OAuth2 {
            token_ref: token_env,
        } => {
            if let Ok(token) = env::var(token_env) {
                headers.push(("Authorization".to_string(), format!("Bearer {}", token)));
            }
        }
    }
    headers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::AuthConfig;

    #[test]
    fn test_resolve_auth_headers_bearer() {
        unsafe {
            std::env::set_var("TEST_TOKEN", "secret123");
        }
        let config = AuthConfig::Bearer {
            token_env: "TEST_TOKEN".to_string(),
        };
        let headers = resolve_auth_headers(&config);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "Authorization");
        assert_eq!(headers[0].1, "Bearer secret123");
    }

    #[test]
    fn test_resolve_auth_headers_basic() {
        unsafe {
            std::env::set_var("TEST_USER", "user");
            std::env::set_var("TEST_PASS", "pass");
        }
        let config = AuthConfig::Basic {
            user_env: "TEST_USER".to_string(),
            pass_env: "TEST_PASS".to_string(),
        };
        let headers = resolve_auth_headers(&config);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "Authorization");
        assert_eq!(headers[0].1, "Basic dXNlcjpwYXNz");
    }
}
