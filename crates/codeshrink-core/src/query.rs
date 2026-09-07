#[derive(Debug, Clone)]
pub struct QueryTerms {
    pub exact: Vec<String>,
    pub stemmed: Vec<String>,
    pub related: Vec<String>,
}

impl QueryTerms {
    pub fn is_empty(&self) -> bool {
        self.exact.is_empty() && self.stemmed.is_empty() && self.related.is_empty()
    }

    pub fn matches(&self, name: &str) -> f64 {
        let lower = name.to_lowercase();
        let parts = split_identifier(&lower);

        for term in &self.exact {
            if lower == *term {
                return 1.0;
            }
        }

        for term in &self.exact {
            if lower.contains(term) {
                return 0.9;
            }
            for part in &parts {
                if part == term {
                    return 0.85;
                }
            }
        }

        for term in &self.stemmed {
            if lower.contains(term) {
                return 0.7;
            }
            for part in &parts {
                if part.starts_with(term) || term.starts_with(part) {
                    return 0.6;
                }
            }
        }

        for term in &self.related {
            if lower.contains(term) {
                return 0.4;
            }
            for part in &parts {
                if part == term {
                    return 0.35;
                }
            }
        }

        0.0
    }
}

pub fn extract_terms(query: &str) -> QueryTerms {
    let stop_words = [
        "where", "is", "the", "a", "an", "how", "does", "what", "find",
        "show", "me", "in", "this", "that", "do", "to", "of", "for",
        "with", "and", "or", "not", "are", "was", "been", "being",
        "have", "has", "had", "can", "could", "will", "would", "should",
        "my", "your", "code", "file", "function", "class", "method",
    ];

    let words: Vec<String> = query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| w.len() >= 2)
        .filter(|w| !stop_words.contains(w))
        .map(String::from)
        .collect();

    let stemmed: Vec<String> = words.iter().map(|w| naive_stem(w)).collect();

    let related: Vec<String> = words
        .iter()
        .flat_map(|w| get_related_terms(w))
        .collect();

    QueryTerms {
        exact: words,
        stemmed,
        related,
    }
}

fn naive_stem(word: &str) -> String {
    let w = word.to_lowercase();
    for suffix in &["ation", "tion", "ing", "ment", "ness", "ity", "ous", "ive", "ful", "less", "able", "ible", "er", "or", "ed", "es", "ly", "al", "ize", "ise", "fy"] {
        if w.len() > suffix.len() + 2 {
            if let Some(stem) = w.strip_suffix(suffix) {
                return stem.to_string();
            }
        }
    }
    w
}

fn get_related_terms(word: &str) -> Vec<String> {
    let groups: &[&[&str]] = &[
        &["auth", "authenticate", "authorize", "authorization", "authentication", "login", "signin", "session", "token", "jwt", "oauth", "credential"],
        &["route", "router", "routing", "endpoint", "handler", "controller", "middleware", "path", "url"],
        &["database", "db", "query", "sql", "model", "schema", "migration", "orm", "repository", "entity"],
        &["config", "configuration", "settings", "options", "env", "environment", "dotenv"],
        &["test", "testing", "spec", "assert", "expect", "mock", "stub", "fixture"],
        &["error", "exception", "panic", "catch", "throw", "result", "failure"],
        &["log", "logger", "logging", "debug", "trace", "info", "warn"],
        &["cache", "caching", "redis", "memcached", "store", "ttl"],
        &["api", "rest", "graphql", "grpc", "rpc", "request", "response"],
        &["user", "account", "profile", "permission", "role", "rbac", "acl"],
        &["encrypt", "decrypt", "hash", "crypto", "cipher", "secret", "key"],
        &["validate", "validation", "sanitize", "parse", "schema", "zod"],
        &["deploy", "ci", "cd", "pipeline", "docker", "kubernetes", "k8s"],
        &["websocket", "socket", "ws", "realtime", "event", "stream", "pubsub"],
    ];

    let lower = word.to_lowercase();
    for group in groups {
        if group.iter().any(|t| lower.contains(t) || t.contains(&lower)) {
            return group
                .iter()
                .filter(|&&t| t != lower)
                .map(|&t| t.to_string())
                .collect();
        }
    }
    Vec::new()
}

fn split_identifier(name: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();

    for ch in name.chars() {
        if ch == '_' || ch == '-' {
            if current.len() >= 2 {
                parts.push(current.to_lowercase());
            }
            current.clear();
        } else if ch.is_uppercase() && !current.is_empty() {
            if current.len() >= 2 {
                parts.push(current.to_lowercase());
            }
            current.clear();
            current.push(ch);
        } else {
            current.push(ch);
        }
    }
    if current.len() >= 2 {
        parts.push(current.to_lowercase());
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_auth_terms() {
        let terms = extract_terms("where is the authentication?");
        assert!(terms.exact.contains(&"authentication".to_string()));
        assert!(!terms.exact.contains(&"where".to_string()));
        assert!(!terms.exact.contains(&"is".to_string()));
        assert!(terms.related.contains(&"login".to_string()));
        assert!(terms.related.contains(&"token".to_string()));
    }

    #[test]
    fn match_scores() {
        let terms = extract_terms("auth");
        assert!(terms.matches("authenticate") > 0.5);
        assert!(terms.matches("AuthService") > 0.5);
        assert!(terms.matches("loginHandler") > 0.1);
        assert_eq!(terms.matches("completely_unrelated"), 0.0);
    }

    #[test]
    fn split_camel_case() {
        let parts = split_identifier("authenticateUser");
        assert!(parts.contains(&"authenticate".to_string()));
        assert!(parts.contains(&"user".to_string()));
    }

    #[test]
    fn split_snake_case() {
        let parts = split_identifier("auth_service_handler");
        assert!(parts.contains(&"auth".to_string()));
        assert!(parts.contains(&"service".to_string()));
        assert!(parts.contains(&"handler".to_string()));
    }

    #[test]
    fn empty_query_filtered() {
        let terms = extract_terms("where is the");
        assert!(terms.is_empty());
    }
}
