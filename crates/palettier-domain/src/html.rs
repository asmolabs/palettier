//! HTML/CSS escaping and sanitization helpers for printable studio sheets.

use crate::color::Rgb;

/// Escapes HTML metacharacters so untrusted text cannot break out of text nodes or attributes.
pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Normalizes a hex color into `#RRGGBB`, falling back to `#808080` to prevent CSS injection.
pub fn sanitize_hex_css(hex: &str) -> String {
    Rgb::of_hex(hex)
        .map(|rgb| rgb.to_hex())
        .unwrap_or_else(|_| "#808080".to_string())
}

/// Validates an inline `data:image/...;base64,...` URI before placing it in `<img src="...">`.
pub fn sanitize_image_data_uri(uri: &str) -> Option<&str> {
    let trimmed = uri.trim();
    const ALLOWED_PREFIXES: [&str; 5] = [
        "data:image/png;base64,",
        "data:image/jpeg;base64,",
        "data:image/jpg;base64,",
        "data:image/webp;base64,",
        "data:image/gif;base64,",
    ];
    let rest = ALLOWED_PREFIXES
        .iter()
        .find_map(|prefix| trimmed.strip_prefix(prefix))?;
    if rest.is_empty() {
        return None;
    }
    let valid_b64 = rest
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'=' | b'\n' | b'\r'));
    if valid_b64 { Some(trimmed) } else { None }
}

