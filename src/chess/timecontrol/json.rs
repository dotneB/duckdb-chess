use super::{Mode, ParsedTimeControl};

pub(super) fn timecontrol_to_json(parsed: &ParsedTimeControl) -> String {
    let mode_str = match parsed.mode {
        Mode::Unknown => "unknown",
        Mode::Unlimited => "unlimited",
        Mode::Sandclock => "sandclock",
        Mode::Normal => "normal",
    };

    let periods_json: Vec<String> = parsed
        .periods
        .iter()
        .map(|p| {
            let moves_str = p
                .moves
                .map(|m| format!(r#""moves":{},"#, m))
                .unwrap_or_default();
            let inc_str = p
                .increment_seconds
                .map(|i| format!(r#","increment":{}"#, i))
                .unwrap_or_default();
            format!(r#"{{{}"base":{}{}}}"#, moves_str, p.base_seconds, inc_str)
        })
        .collect();

    let raw_json =
        serde_json::to_string(&parsed.raw).unwrap_or_else(|_| format!("\"{}\"", parsed.raw));
    let normalized_json = parsed
        .normalized
        .as_ref()
        .map(|s| serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s)))
        .unwrap_or_else(|| "null".to_string());

    let warnings_json =
        serde_json::to_string(&parsed.warnings).unwrap_or_else(|_| "[]".to_string());

    format!(
        r#"{{"raw":{},"normalized":{},"mode":"{}","periods":[{}],"warnings":{},"inferred":{},"overflow":{}}}"#,
        raw_json,
        normalized_json,
        mode_str,
        periods_json.join(","),
        warnings_json,
        if parsed.inferred { "true" } else { "false" },
        if parsed.overflow { "true" } else { "false" }
    )
}

#[cfg(test)]
mod tests {
    use super::super::{parse_timecontrol, timecontrol_to_json};

    #[test]
    fn test_json_output() {
        let result = parse_timecontrol("3+2").unwrap();
        let json = timecontrol_to_json(&result);
        assert!(json.contains(r#""raw":"3+2""#));
        assert!(json.contains(r#""normalized":"180+2""#));
        assert!(json.contains(r#""mode":"normal""#));
        assert!(json.contains(r#""inferred":true"#));
    }

    #[test]
    fn test_json_always_includes_normalized_key() {
        let result = parse_timecontrol("klassisch").unwrap();
        let json = timecontrol_to_json(&result);
        assert!(json.contains(r#""normalized":null"#));
    }

    #[test]
    fn test_json_includes_overflow_warning() {
        let result = parse_timecontrol("G71582789").unwrap();
        let json = timecontrol_to_json(&result);
        assert!(json.contains("inference_arithmetic_overflow"));
        assert!(json.contains(r#""normalized":null"#));
        assert!(json.contains(r#""periods":[]"#));
        assert!(json.contains(r#""overflow":true"#));
    }
}
