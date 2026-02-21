use super::{Mode, ParsedTimeControl, Period, TimeControlError, parse_u32};

#[allow(clippy::ptr_arg)]
pub(super) fn try_strict_parse(
    input: &str,
    warnings: &mut Vec<String>,
) -> Option<Result<ParsedTimeControl, TimeControlError>> {
    if input == "?" {
        return Some(Ok(ParsedTimeControl {
            raw: input.to_string(),
            normalized: Some("?".to_string()),
            periods: Vec::new(),
            mode: Mode::Unknown,
            warnings: warnings.clone(),
            inferred: false,
            overflow: false,
        }));
    }

    if input == "-" {
        return Some(Ok(ParsedTimeControl {
            raw: input.to_string(),
            normalized: Some("-".to_string()),
            periods: Vec::new(),
            mode: Mode::Unlimited,
            warnings: warnings.clone(),
            inferred: false,
            overflow: false,
        }));
    }

    if let Some(secs_str) = input.strip_prefix('*')
        && let Some(secs) = parse_u32(secs_str)
    {
        return Some(Ok(ParsedTimeControl {
            raw: input.to_string(),
            normalized: Some(format!("*{}", secs)),
            periods: vec![Period {
                moves: None,
                base_seconds: secs,
                increment_seconds: None,
            }],
            mode: Mode::Sandclock,
            warnings: warnings.clone(),
            inferred: false,
            overflow: false,
        }));
    }

    let stage_parts: Vec<&str> = input.split(':').collect();
    if stage_parts.len() > 1 {
        let mut periods = Vec::new();
        let mut all_valid = true;

        for stage in &stage_parts {
            if let Some(period) = parse_stage(stage) {
                periods.push(period);
            } else {
                all_valid = false;
                break;
            }
        }

        if all_valid && !periods.is_empty() {
            let normalized = periods
                .iter()
                .map(format_period)
                .collect::<Vec<_>>()
                .join(":");

            return Some(Ok(ParsedTimeControl {
                raw: input.to_string(),
                normalized: Some(normalized),
                periods,
                mode: Mode::Normal,
                warnings: warnings.clone(),
                inferred: false,
                overflow: false,
            }));
        }
    }

    if let Some(period) = parse_stage(input) {
        if looks_like_minute_shorthand(&period) {
            return None;
        }
        let normalized = format_period(&period);
        return Some(Ok(ParsedTimeControl {
            raw: input.to_string(),
            normalized: Some(normalized.clone()),
            periods: vec![period],
            mode: Mode::Normal,
            warnings: warnings.clone(),
            inferred: false,
            overflow: false,
        }));
    }

    None
}

fn looks_like_minute_shorthand(period: &Period) -> bool {
    if period.moves.is_some() {
        return false;
    }

    if let Some(inc) = period.increment_seconds {
        (period.base_seconds < 60 && inc <= 60)
            || ((period.base_seconds == 75 || period.base_seconds == 90) && inc == 30)
    } else {
        period.base_seconds < 60
    }
}

pub(super) fn parse_stage(s: &str) -> Option<Period> {
    let (base_part, inc_part) = match s.contains('+') {
        true => {
            let parts: Vec<&str> = s.split('+').collect();
            if parts.len() != 2 {
                return None;
            }
            (parts[0], Some(parts[1]))
        }
        false => (s, None),
    };

    let (moves, base_str) = if base_part.contains('/') {
        let parts: Vec<&str> = base_part.split('/').collect();
        if parts.len() != 2 {
            return None;
        }
        (Some(parse_u32(parts[0])?), parts[1])
    } else {
        (None, base_part)
    };

    let base_seconds = parse_u32(base_str)?;
    let increment_seconds = match inc_part {
        Some(inc_str) => Some(parse_u32(inc_str)?),
        None => None,
    };

    Some(Period {
        moves,
        base_seconds,
        increment_seconds,
    })
}

pub(super) fn format_period(p: &Period) -> String {
    let base = match p.moves {
        Some(m) => format!("{}/{}", m, p.base_seconds),
        None => p.base_seconds.to_string(),
    };

    match p.increment_seconds {
        Some(inc) => format!("{}+{}", base, inc),
        None => base,
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Mode, parse_timecontrol};

    #[test]
    fn test_strict_parse_question_mark() {
        let result = parse_timecontrol("?").unwrap();
        assert_eq!(result.normalized, Some("?".to_string()));
        assert_eq!(result.mode, Mode::Unknown);
    }

    #[test]
    fn test_strict_parse_dash() {
        let result = parse_timecontrol("-").unwrap();
        assert_eq!(result.normalized, Some("-".to_string()));
        assert_eq!(result.mode, Mode::Unlimited);
    }

    #[test]
    fn test_strict_parse_sandclock() {
        let result = parse_timecontrol("*60").unwrap();
        assert_eq!(result.normalized, Some("*60".to_string()));
        assert_eq!(result.mode, Mode::Sandclock);
    }

    #[test]
    fn test_strict_parse_simple() {
        let result = parse_timecontrol("180+2").unwrap();
        assert_eq!(result.normalized, Some("180+2".to_string()));
        assert!(!result.inferred);
    }

    #[test]
    fn test_strict_parse_by_moves() {
        let result = parse_timecontrol("40/5400+30").unwrap();
        assert_eq!(result.normalized, Some("40/5400+30".to_string()));
        assert!(!result.inferred);
    }

    #[test]
    fn test_strict_parse_multi_stage() {
        let result = parse_timecontrol("40/5400+30:1800+30").unwrap();
        assert_eq!(result.normalized, Some("40/5400+30:1800+30".to_string()));
        assert!(!result.inferred);
    }

    #[test]
    fn test_dont_reinterpret_large_values() {
        let result = parse_timecontrol("900+10").unwrap();
        assert_eq!(result.normalized, Some("900+10".to_string()));
        assert!(!result.inferred);
    }

    #[test]
    fn test_invalid_stage_with_non_numeric_moves_fails() {
        let result = parse_timecontrol("x/600+5").unwrap();
        assert_eq!(result.normalized, None);
    }
}
