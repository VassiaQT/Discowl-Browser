use url::Url;
use urlencoding;

pub fn normalize_url(input: String) -> String {
    let input = input.trim();
    
    if input.is_empty() {
        return "https://duckduckgo.com".to_string();
    }
    
    if is_probably_url(input) {
        if input.starts_with("http://") || input.starts_with("https://") {
            input.to_string()
        } else {
            format!("https://{}", input)
        }
    } else {
        let query = urlencoding::encode(input);
        format!("https://duckduckgo.com/?q={}", query)
    }
}

fn is_probably_url(input: &str) -> bool {
    if input.contains(' ') {
        return false;
    }
    
    if input.starts_with("http://") || input.starts_with("https://") || input.starts_with("file://") {
        return true;
    }
    
    if input.contains('.') && !input.contains(' ') {
        let parts: Vec<&str> = input.split('.').collect();
        if parts.len() >= 2 {
            let tld = parts.last().unwrap();
            if tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic()) {
                return true;
            }
        }
    }
    
    if input == "localhost" || input.starts_with("localhost:") || input.starts_with("127.0.0.1") {
        return true;
    }
    
    false
}

pub fn is_valid_url(url: &str) -> bool {
    Url::parse(url).is_ok()
}

pub fn get_domain(url: &str) -> Option<String> {
    Url::parse(url).ok()?.domain().map(|s| s.to_string())
}

pub fn get_favicon_url(url: &str) -> Option<String> {
    get_domain(url).map(|domain| format!("https://www.google.com/s2/favicons?domain={}&sz=32", domain))
}