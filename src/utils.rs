use actix_web::HttpRequest;

/// Extract client IP address from request
pub fn get_client_ip(req: &HttpRequest) -> Option<String> {
    req.connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string())
}

/// Extract user agent from request
pub fn get_user_agent(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// HTML escape for preventing XSS
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('/', "&#x2F;")
}

/// Validate agent ID format
pub fn validate_agent_id(agent_id: &str) -> bool {
    // Agent ID should be alphanumeric and not too long
    agent_id.len() <= 50
        && agent_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Generate secure random string
pub fn generate_random_string(len: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();

    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_validate_agent_id() {
        assert!(validate_agent_id("001"));
        assert!(validate_agent_id("agent-123"));
        assert!(!validate_agent_id("../../../etc/passwd"));
        assert!(!validate_agent_id("a".repeat(51).as_str()));
    }
}