use crate::protocols::ProtocolMatch;

pub fn detect_from_name(name: Option<&str>) -> Option<ProtocolMatch> {
    let name = name?;
    let lower = name.to_ascii_lowercase();
    let looks_like_parkside = lower.contains("parkside")
        || lower.contains("paps")
        || lower.contains("smart battery")
        || lower.contains("performance");

    looks_like_parkside.then(|| ProtocolMatch {
        protocol: "parkside-smart-battery".to_string(),
        confidence: "candidate".to_string(),
        evidence: vec![format!("Geraetename enthaelt PARKSIDE-Hinweis: {name}")],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_parkside_names_as_candidates() {
        let detected = detect_from_name(Some("PARKSIDE Smart Battery")).expect("match");
        assert_eq!(detected.confidence, "candidate");
    }
}
