/// Parse `pkg:npm/%40angular/core@15.0.0` → (name, version, ecosystem)
pub fn parse_purl(purl: &str) -> (String, String, String) {
    // Format: pkg:<ecosystem>/<name>@<version>[?qualifiers]
    let without_scheme = purl.strip_prefix("pkg:").unwrap_or(purl);
    let (ecosystem, rest) =
        without_scheme.split_once('/').unwrap_or(("unknown", without_scheme));

    // Remove qualifiers (everything after '?' or '#')
    let rest = rest.split('?').next().unwrap_or(rest);
    let rest = rest.split('#').next().unwrap_or(rest);

    let (name_raw, version) = rest
        .rsplit_once('@')
        .map(|(n, v)| (n, v.to_string()))
        .unwrap_or((rest, String::new()));

    // Minimal percent-decoding for common cases (e.g. %40 → @)
    let name = name_raw.replace("%40", "@").replace("%2F", "/");

    (name, version, ecosystem.to_string())
}
