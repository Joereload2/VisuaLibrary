/// Acquisition decision for Manual Factory needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcquisitionDecision {
    Found,
    Generate,
    Skipped,
}

impl AcquisitionDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Found => "found",
            Self::Generate => "generate",
            Self::Skipped => "skipped",
        }
    }
}

/// MVP matching (D-013): approved candidate already filtered by concept/representation.
/// `need` / `asset` values: exact match or either side is `any` / empty.
pub fn field_matches(need: &str, asset: Option<&str>) -> bool {
    let need = need.trim();
    if need.is_empty() || need.eq_ignore_ascii_case("any") {
        return true;
    }
    match asset.map(str::trim).filter(|s| !s.is_empty()) {
        None => true,
        Some(a) if a.eq_ignore_ascii_case("any") => true,
        Some(a) => a.eq_ignore_ascii_case(need),
    }
}

/// Pure policy: if a sufficiently good approved candidate exists → Found, else Generate.
pub fn decide_acquisition(has_sufficient_candidate: bool) -> AcquisitionDecision {
    if has_sufficient_candidate {
        AcquisitionDecision::Found
    } else {
        AcquisitionDecision::Generate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_matches_everything() {
        assert!(field_matches("any", Some("portrait")));
        assert!(field_matches("portrait", Some("any")));
        assert!(field_matches("portrait", None));
    }

    #[test]
    fn exact_orientation() {
        assert!(field_matches("portrait", Some("portrait")));
        assert!(!field_matches("portrait", Some("landscape")));
    }

    #[test]
    fn decide_found_or_generate() {
        assert_eq!(decide_acquisition(true), AcquisitionDecision::Found);
        assert_eq!(decide_acquisition(false), AcquisitionDecision::Generate);
    }
}
