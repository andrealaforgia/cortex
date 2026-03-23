use regex::Regex;

pub struct RedactionEngine {
    patterns: Vec<(Regex, &'static str)>,
}

impl RedactionEngine {
    pub fn new() -> Self {
        let patterns = vec![
            (Regex::new(r"(?i)(api[_-]?key|token|secret|password|passwd)\s*[=:]\s*\S+").unwrap(),
             "$1=***REDACTED***"),
            (Regex::new(r"(?i)bearer\s+\S+").unwrap(),
             "Bearer ***REDACTED***"),
            (Regex::new(r"(?i)(aws_access_key_id|aws_secret_access_key)\s*=\s*\S+").unwrap(),
             "$1=***REDACTED***"),
            (Regex::new(r"-----BEGIN\s+\w+\s+PRIVATE\s+KEY-----[\s\S]*?-----END\s+\w+\s+PRIVATE\s+KEY-----").unwrap(),
             "***PRIVATE_KEY_REDACTED***"),
            (Regex::new(r"(?i)(mongodb|postgres|postgresql|mysql|redis)://\S+").unwrap(),
             "$1://***REDACTED***"),
            (Regex::new(r"(?i)sk-[a-zA-Z0-9]{20,}").unwrap(),
             "***API_KEY_REDACTED***"),
        ];
        RedactionEngine { patterns }
    }

    pub fn redact(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (pattern, replacement) in &self.patterns {
            result = pattern.replace_all(&result, *replacement).to_string();
        }
        result
    }
}

impl Default for RedactionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redacts_api_keys() {
        let engine = RedactionEngine::new();
        let input = "API_KEY=sk-abc123xyz456 some text";
        let output = engine.redact(input);
        assert!(!output.contains("sk-abc123xyz456"));
        assert!(output.contains("REDACTED"));
    }

    #[test]
    fn test_redacts_bearer_tokens() {
        let engine = RedactionEngine::new();
        let input = "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.test";
        let output = engine.redact(input);
        assert!(!output.contains("eyJhbGciOiJIUzI1NiJ9"));
    }

    #[test]
    fn test_redacts_connection_strings() {
        let engine = RedactionEngine::new();
        let input = "DATABASE_URL=postgres://user:pass@host:5432/db";
        let output = engine.redact(input);
        assert!(!output.contains("user:pass@host"));
    }

    #[test]
    fn test_leaves_normal_text() {
        let engine = RedactionEngine::new();
        let input = "Hello world, this is normal output";
        let output = engine.redact(input);
        assert_eq!(input, output);
    }
}
