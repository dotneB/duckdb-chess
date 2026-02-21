use std::sync::LazyLock;

use super::strict::parse_stage;
use super::{
    Mode, ParsedTimeControl, Period, TimeControlError, checked_compose_base_increment,
    checked_hours_minutes_to_seconds, checked_minutes_to_seconds, inferred_parsed, parse_u32,
};

static TRAILING_QUALIFIER_SUFFIX_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^(.+?)\s+(\p{L}+)$").expect("valid trailing qualifier suffix regex")
});

const PER_MOVE_RE: &str = r"(?:per\s*move|/move|/mv|/m)";
const FROM_MOVE_RE: &str = r"(?:from\s*move\s*\w+)";
const INCREMENT_WORDING_RE: &str = r"(?:increment|inc|incr|added|additional)";

const MINUTE_UNIT_RE: &str = r"(?:minutes?|mins?|mns?|min(?:\.|utes?)?|m\.?|m)";
const SECOND_UNIT_RE: &str = r"(?:seconds?|secs?|sec\.?|s\.?|sek|ss|secss)";

static G_PREFIX_NUMERIC_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(&format!(
        r"^(\d+)\s*(?:\+\s*(\d+)\s*(?:inc|{}\s*(?:(?:added\s+)?{}))?)?$",
        SECOND_UNIT_RE, PER_MOVE_RE,
    ))
    .expect("valid g-prefix numeric regex")
});

static APOSTROPHE_PER_MOVE_SHORTHAND_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?i)^\s*(\d+)\s*'\s*\+\s*(\d+)\s*''\s*/\s*(?:m|mv|move)\b.*$")
        .expect("valid apostrophe per-move shorthand regex")
});

static COMPACT_MINUTE_SECOND_INCREMENT_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(&format!(
        r"(?i)^\s*(?:[\p{{L}}\s]+:\s*)?(\d+)\s*{}?\s*\+\s*(\d+)\s*{}\b(?:\s*{})?(?:\s*(?:{}|{}))?\s*$",
        MINUTE_UNIT_RE, SECOND_UNIT_RE, INCREMENT_WORDING_RE, FROM_MOVE_RE, PER_MOVE_RE,
    ))
    .expect("valid compact minute-second increment regex")
});

static CLOCK_STYLE_INCREMENT_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(&format!(
        r"(?i)^\s*(\d+)\s*:\s*([0-5]?\d)(?:\.\d+)?\s*\+\s*(\d+)\s*{}\b(?:\s*{})?(?:\s*(?:{}|{}))?\s*$",
        SECOND_UNIT_RE, INCREMENT_WORDING_RE, FROM_MOVE_RE, PER_MOVE_RE,
    ))
    .expect("valid clock-style increment regex")
});

static COMPACT_FIDE_APOSTROPHE_WITH_MOVE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"(?i)^\s*(\d+)\s*'\s*/\s*(\d+)\s*(?:m|moves?)?\s*(?:\+|&)\s*(\d+)\s*'\s*/\s*(?:g|end)\s*(?:\+|&)\s*(\d+)\s*(?:''?)?\s*/\s*(?:m|mv|move)\b.*$",
    )
    .expect("valid compact FIDE apostrophe with-move regex")
});

static COMPACT_FIDE_APOSTROPHE_G_COMPACT_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"(?i)^\s*(\d+)\s*'\s*/\s*(\d+)\s*(?:m|moves?)?\s*(?:\+|&)\s*(\d+)\s*'\s*/\s*(?:g|end)\s*(?:\+|&)\s*(\d+)\s*''\s*$",
    )
    .expect("valid compact FIDE apostrophe g-compact regex")
});

static COMPACT_FIDE_APOSTROPHE_BONUS_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"(?i)^\s*(\d+)\s*'\s*/\s*(\d+)\s*(?:m|moves?)\s*(?:\+|&)\s*(\d+)\s*'\s*(?:\+|&)\s*(\d+)\s*''\s*bonus(?:\s*increment)?\s*$",
    )
    .expect("valid compact FIDE apostrophe bonus regex")
});

static FIDE_ADDITIONAL_WORDING_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(&format!(
        r"(?i)^\s*(\d+)\s*{}\s*\+\s*(\d+)\s*{}\s*additional\s*\+\s*(\d+)\s*{}\s*after\s*move\s*40\s*$",
        MINUTE_UNIT_RE,
        SECOND_UNIT_RE,
        MINUTE_UNIT_RE,
    ))
    .expect("valid FIDE additional wording regex")
});

static COMPACT_FIDE_TRIPLE_PLUS_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(&format!(
        r"(?i)^\s*(\d+)\s*\+\s*(\d+)\s*\+\s*(\d+)\s*(?:{}\s*{}|after\s*40\s*moves?)\s*$",
        SECOND_UNIT_RE, PER_MOVE_RE,
    ))
    .expect("valid compact FIDE triple-plus regex")
});

static TWO_STAGE_SLASH_SHORTHAND_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^\s*(\d+)\s*\+\s*(\d+)\s*/\s*(\d+)\s*\+\s*(\d+)\s*$")
        .expect("valid two-stage slash shorthand regex")
});

static FREETEXT_FIDE_STAGE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(&format!(
        r"(\d+)\s*{}\s*(?:for\s*(\d+)\s*(?:moves?|mv|mvs?)|/\s*(\d+))",
        MINUTE_UNIT_RE,
    ))
    .expect("valid free-text FIDE stage regex")
});

static FREETEXT_FIDE_REST_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(&format!(
        r"(?:\+|,|then\s+)\s*(\d+)\s*{}\s*(?:for\s*(?:the\s*)?rest|rest)?",
        MINUTE_UNIT_RE,
    ))
    .expect("valid free-text FIDE rest regex")
});

static FREETEXT_MINUTE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(&format!(r"(\d+)\s*{}\b", MINUTE_UNIT_RE,))
        .expect("valid free-text minute regex")
});

static FREETEXT_SECOND_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(&format!(r"(\d+)\s*{}\b", SECOND_UNIT_RE,))
        .expect("valid free-text second regex")
});

static FREETEXT_INC_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(&format!(
        r"(\d+)\s*{}\s*(?:(?:added\s+)?{})",
        SECOND_UNIT_RE, PER_MOVE_RE,
    ))
    .expect("valid free-text increment regex")
});

pub(super) fn strip_trailing_qualifier_suffix(input: &str) -> Option<String> {
    let caps = TRAILING_QUALIFIER_SUFFIX_RE.captures(input)?;
    let core = caps.get(1)?.as_str().trim();
    let suffix = caps.get(2)?.as_str().trim();

    if core.is_empty() || suffix.is_empty() {
        return None;
    }

    if suffix.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }

    if suffix.chars().any(|c| {
        matches!(
            c,
            '+' | '/' | ':' | '*' | '-' | '?' | '\'' | '"' | '&' | '|'
        )
    }) {
        return None;
    }

    Some(core.to_string())
}

pub(super) struct PreprocessResult {
    pub normalized: String,
    pub warnings: Vec<String>,
    pub has_ambiguous_quote_residue: bool,
}

fn strip_outer_quote_wrappers(input: &str) -> (String, bool) {
    let mut s = input.trim().to_string();
    let mut stripped = false;

    loop {
        let mut chars = s.chars();
        let first = chars.next();
        let last = s.chars().next_back();

        let should_strip = matches!(first, Some('\'' | '"')) && matches!(last, Some('\'' | '"'));
        if !should_strip || s.len() < 2 {
            break;
        }

        s = s[1..s.len() - 1].trim().to_string();
        stripped = true;
    }

    (s, stripped)
}

fn has_ambiguous_quote_residue(input: &str) -> bool {
    if input.contains('"') {
        return true;
    }

    let bytes = input.as_bytes();
    let mut idx = 0;

    while idx < bytes.len() {
        if bytes[idx] != b'\'' {
            idx += 1;
            continue;
        }

        let run_start = idx;
        while idx < bytes.len() && bytes[idx] == b'\'' {
            idx += 1;
        }

        if run_start == 0 {
            return true;
        }

        let prev = bytes[run_start - 1] as char;
        if !prev.is_ascii_digit() {
            return true;
        }
    }

    false
}

pub(super) fn preprocess(input: &str) -> PreprocessResult {
    let mut warnings = Vec::new();
    let mut s = input.to_string();

    let original = s.clone();

    s = s.trim().to_string();
    if s != original {
        warnings.push("trimmed".to_string());
    }

    let (stripped, did_strip_quotes) = strip_outer_quote_wrappers(&s);
    s = stripped;
    if did_strip_quotes {
        warnings.push("stripped_quotes".to_string());
    }

    let has_ambiguous_quote_residue = has_ambiguous_quote_residue(&s);
    if has_ambiguous_quote_residue {
        warnings.push("ambiguous_quote_residue".to_string());
    }

    let original = s.clone();
    s = s
        .replace(" + ", "+")
        .replace("+ ", "+")
        .replace(" +", "+")
        .replace(" - ", "-")
        .replace("- ", "-")
        .replace(" -", "-")
        .replace(" / ", "/")
        .replace("/ ", "/")
        .replace(" /", "/")
        .replace(" : ", ":")
        .replace(": ", ":")
        .replace(" :", ":");
    if s != original {
        warnings.push("normalized_operator_whitespace".to_string());
    }

    let original = s.clone();
    s = s.replace(['|', '_'], "+");
    if s != original {
        warnings.push("mapped_separator".to_string());

        let original2 = s.clone();
        s = s.replace(" + ", "+").replace("+ ", "+").replace(" +", "+");
        if s != original2 {
            warnings.push("normalized_operator_whitespace".to_string());
        }
    }

    if s.ends_with('\'') && !s.ends_with("''") {
        let candidate = s.trim_end_matches('\'').to_string();
        let candidate_is_strict = match candidate.as_str() {
            "?" | "-" => true,
            _ => {
                candidate.strip_prefix('*').and_then(parse_u32).is_some()
                    || (candidate.contains(':')
                        && candidate
                            .split(':')
                            .all(|stage| parse_stage(stage).is_some()))
                    || parse_stage(&candidate).is_some()
            }
        };

        if candidate_is_strict {
            s = candidate;
            warnings.push("stripped_trailing_apostrophe".to_string());
        }
    }

    PreprocessResult {
        normalized: s,
        warnings,
        has_ambiguous_quote_residue,
    }
}

pub(super) fn try_inference(
    input: &str,
    warnings: &mut Vec<String>,
) -> Option<Result<ParsedTimeControl, TimeControlError>> {
    if let Some(result) = try_g_prefix_shorthand(input, warnings) {
        return Some(result);
    }

    if let Some(result) = try_apostrophe_per_move_shorthand(input, warnings) {
        return Some(result);
    }

    if let Some(result) = try_compact_fide_apostrophe_shorthand(input, warnings) {
        return Some(result);
    }

    if let Some(result) = try_fide_additional_wording(input, warnings) {
        return Some(result);
    }

    if let Some(result) = try_compact_fide_triple_plus(input, warnings) {
        return Some(result);
    }

    if let Some(result) = try_two_stage_slash_shorthand(input, warnings) {
        return Some(result);
    }

    if let Some(result) = try_compact_minute_second_increment(input, warnings) {
        return Some(result);
    }

    if let Some(result) = try_clock_style_increment(input, warnings) {
        return Some(result);
    }

    if let Some(result) = try_apostrophe_notation(input, warnings) {
        return Some(result);
    }

    if input.contains('+') {
        let parts: Vec<&str> = input.split('+').collect();
        if parts.len() == 2
            && let (Some(base), Some(inc)) = (parse_u32(parts[0]), parse_u32(parts[1]))
        {
            if base < 60 && inc <= 60 {
                let base_seconds = checked_minutes_to_seconds(base);
                let overflow = base_seconds.is_none();
                let base_seconds = base_seconds.unwrap_or(0);
                warnings.push("interpreted_small_base_as_minutes".to_string());
                return Some(Ok(inferred_parsed(
                    input,
                    warnings,
                    vec![Period {
                        moves: None,
                        base_seconds,
                        increment_seconds: Some(inc),
                    }],
                    overflow,
                )));
            }

            if (base == 75 || base == 90) && inc == 30 {
                let base_seconds = checked_minutes_to_seconds(base);
                let overflow = base_seconds.is_none();
                let base_seconds = base_seconds.unwrap_or(0);
                warnings.push("interpreted_classical_75_90_as_minutes".to_string());
                return Some(Ok(inferred_parsed(
                    input,
                    warnings,
                    vec![Period {
                        moves: None,
                        base_seconds,
                        increment_seconds: Some(inc),
                    }],
                    overflow,
                )));
            }
        }
    }

    if !input.contains('+')
        && !input.contains('/')
        && !input.contains(':')
        && let Some(n) = parse_u32(input)
        && n < 60
    {
        let base_seconds = checked_minutes_to_seconds(n);
        let overflow = base_seconds.is_none();
        let base_seconds = base_seconds.unwrap_or(0);
        warnings.push("interpreted_small_bare_number_as_minutes".to_string());
        return Some(Ok(inferred_parsed(
            input,
            warnings,
            vec![Period {
                moves: None,
                base_seconds,
                increment_seconds: None,
            }],
            overflow,
        )));
    }

    None
}

#[allow(clippy::ptr_arg)]
fn try_g_prefix_shorthand(
    input: &str,
    warnings: &mut Vec<String>,
) -> Option<Result<ParsedTimeControl, TimeControlError>> {
    let lower = input.to_lowercase();

    let rest = if let Some(rest) = lower.strip_prefix("game") {
        rest
    } else if let Some(rest) = lower.strip_prefix('g') {
        rest
    } else {
        return None;
    };

    let mut rest = rest.trim_start();
    if let Some(stripped) = rest.strip_prefix('/') {
        rest = stripped;
    } else if let Some(stripped) = rest.strip_prefix(':') {
        rest = stripped;
    }

    let mut candidate = rest.trim().replace(";+", "+").replace(';', "+");
    candidate = candidate
        .replace(" + ", "+")
        .replace("+ ", "+")
        .replace(" +", "+")
        .replace("\t+", "+")
        .replace("+\t", "+");
    while candidate.contains("++") {
        candidate = candidate.replace("++", "+");
    }

    if let Some(caps) = G_PREFIX_NUMERIC_RE.captures(candidate.trim()) {
        let base_mins = caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok())?;
        let increment = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok());

        let base_seconds = checked_minutes_to_seconds(base_mins);
        let overflow = base_seconds.is_none();
        let base_seconds = base_seconds.unwrap_or(0);

        warnings.push("interpreted_g_prefix_as_minutes".to_string());

        return Some(Ok(inferred_parsed(
            input,
            warnings,
            vec![Period {
                moves: None,
                base_seconds,
                increment_seconds: increment,
            }],
            overflow,
        )));
    }

    None
}

#[allow(clippy::ptr_arg)]
fn try_apostrophe_per_move_shorthand(
    input: &str,
    warnings: &mut Vec<String>,
) -> Option<Result<ParsedTimeControl, TimeControlError>> {
    let caps = APOSTROPHE_PER_MOVE_SHORTHAND_RE.captures(input)?;
    let base_mins = caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok())?;
    let inc_secs = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok())?;

    let base_seconds = checked_minutes_to_seconds(base_mins);
    let overflow = base_seconds.is_none();
    let base_seconds = base_seconds.unwrap_or(0);

    warnings.push("interpreted_apostrophe_per_move_suffix".to_string());
    Some(Ok(inferred_parsed(
        input,
        warnings,
        vec![Period {
            moves: None,
            base_seconds,
            increment_seconds: Some(inc_secs),
        }],
        overflow,
    )))
}

#[allow(clippy::ptr_arg)]
fn try_compact_minute_second_increment(
    input: &str,
    warnings: &mut Vec<String>,
) -> Option<Result<ParsedTimeControl, TimeControlError>> {
    let caps = COMPACT_MINUTE_SECOND_INCREMENT_RE.captures(input)?;
    let base_mins = caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok())?;
    let inc_secs = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok())?;

    let base_seconds = checked_minutes_to_seconds(base_mins);
    let overflow = base_seconds.is_none();
    let base_seconds = base_seconds.unwrap_or(0);

    warnings.push("interpreted_compact_minute_second_increment".to_string());
    Some(Ok(inferred_parsed(
        input,
        warnings,
        vec![Period {
            moves: None,
            base_seconds,
            increment_seconds: Some(inc_secs),
        }],
        overflow,
    )))
}

#[allow(clippy::ptr_arg)]
fn try_clock_style_increment(
    input: &str,
    warnings: &mut Vec<String>,
) -> Option<Result<ParsedTimeControl, TimeControlError>> {
    let caps = CLOCK_STYLE_INCREMENT_RE.captures(input)?;

    let hours = caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok())?;
    let minutes = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok())?;
    let inc_secs = caps.get(3).and_then(|m| m.as_str().parse::<u32>().ok())?;

    let base_seconds = checked_hours_minutes_to_seconds(hours, minutes);
    let overflow = base_seconds.is_none();
    let base_seconds = base_seconds.unwrap_or(0);

    warnings.push("interpreted_clock_style_base".to_string());
    Some(Ok(inferred_parsed(
        input,
        warnings,
        vec![Period {
            moves: None,
            base_seconds,
            increment_seconds: Some(inc_secs),
        }],
        overflow,
    )))
}

fn parse_compact_fide_caps(caps: &regex::Captures<'_>) -> Option<(u32, u32, u32, u32)> {
    let base_mins = caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok())?;
    let moves = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok())?;
    let rest_mins = caps.get(3).and_then(|m| m.as_str().parse::<u32>().ok())?;
    let inc_secs = caps.get(4).and_then(|m| m.as_str().parse::<u32>().ok())?;
    Some((base_mins, moves, rest_mins, inc_secs))
}

#[allow(clippy::ptr_arg)]
fn try_compact_fide_apostrophe_shorthand(
    input: &str,
    warnings: &mut Vec<String>,
) -> Option<Result<ParsedTimeControl, TimeControlError>> {
    let (caps, warning_code) = if let Some(c) = COMPACT_FIDE_APOSTROPHE_WITH_MOVE_RE.captures(input)
    {
        (c, "interpreted_compact_fide_apostrophe")
    } else if let Some(c) = COMPACT_FIDE_APOSTROPHE_G_COMPACT_RE.captures(input) {
        (c, "interpreted_compact_fide_apostrophe")
    } else if let Some(c) = COMPACT_FIDE_APOSTROPHE_BONUS_RE.captures(input) {
        (c, "interpreted_compact_fide_bonus_wording")
    } else {
        return None;
    };

    let (base_mins, moves, rest_mins, inc_secs) = parse_compact_fide_caps(&caps)?;
    warnings.push(warning_code.to_string());

    let base_seconds = checked_minutes_to_seconds(base_mins);
    let rest_seconds = checked_minutes_to_seconds(rest_mins);
    let overflow = base_seconds.is_none() || rest_seconds.is_none();
    let base_seconds = base_seconds.unwrap_or(0);
    let rest_seconds = rest_seconds.unwrap_or(0);

    Some(Ok(inferred_parsed(
        input,
        warnings,
        vec![
            Period {
                moves: Some(moves),
                base_seconds,
                increment_seconds: Some(inc_secs),
            },
            Period {
                moves: None,
                base_seconds: rest_seconds,
                increment_seconds: Some(inc_secs),
            },
        ],
        overflow,
    )))
}

#[allow(clippy::ptr_arg)]
fn try_fide_additional_wording(
    input: &str,
    warnings: &mut Vec<String>,
) -> Option<Result<ParsedTimeControl, TimeControlError>> {
    let caps = FIDE_ADDITIONAL_WORDING_RE.captures(input)?;

    let base_mins = caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok())?;
    let inc_secs = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok())?;
    let rest_mins = caps.get(3).and_then(|m| m.as_str().parse::<u32>().ok())?;

    let base_seconds = checked_minutes_to_seconds(base_mins);
    let rest_seconds = checked_minutes_to_seconds(rest_mins);
    let overflow = base_seconds.is_none() || rest_seconds.is_none();
    let base_seconds = base_seconds.unwrap_or(0);
    let rest_seconds = rest_seconds.unwrap_or(0);

    warnings.push("interpreted_fide_additional_wording".to_string());
    Some(Ok(inferred_parsed(
        input,
        warnings,
        vec![
            Period {
                moves: Some(40),
                base_seconds,
                increment_seconds: Some(inc_secs),
            },
            Period {
                moves: None,
                base_seconds: rest_seconds,
                increment_seconds: Some(inc_secs),
            },
        ],
        overflow,
    )))
}

#[allow(clippy::ptr_arg)]
fn try_compact_fide_triple_plus(
    input: &str,
    warnings: &mut Vec<String>,
) -> Option<Result<ParsedTimeControl, TimeControlError>> {
    let caps = COMPACT_FIDE_TRIPLE_PLUS_RE.captures(input)?;

    let base_mins = caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok())?;
    let rest_mins = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok())?;
    let inc_secs = caps.get(3).and_then(|m| m.as_str().parse::<u32>().ok())?;

    let base_seconds = checked_minutes_to_seconds(base_mins);
    let rest_seconds = checked_minutes_to_seconds(rest_mins);
    let overflow = base_seconds.is_none() || rest_seconds.is_none();
    let base_seconds = base_seconds.unwrap_or(0);
    let rest_seconds = rest_seconds.unwrap_or(0);

    warnings.push("interpreted_compact_fide_triple_plus".to_string());
    Some(Ok(inferred_parsed(
        input,
        warnings,
        vec![
            Period {
                moves: Some(40),
                base_seconds,
                increment_seconds: Some(inc_secs),
            },
            Period {
                moves: None,
                base_seconds: rest_seconds,
                increment_seconds: Some(inc_secs),
            },
        ],
        overflow,
    )))
}

#[allow(clippy::ptr_arg)]
fn try_two_stage_slash_shorthand(
    input: &str,
    warnings: &mut Vec<String>,
) -> Option<Result<ParsedTimeControl, TimeControlError>> {
    let caps = TWO_STAGE_SLASH_SHORTHAND_RE.captures(input)?;

    let first_mins = caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok())?;
    let first_inc = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok())?;
    let second_mins = caps.get(3).and_then(|m| m.as_str().parse::<u32>().ok())?;
    let second_inc = caps.get(4).and_then(|m| m.as_str().parse::<u32>().ok())?;

    let first_seconds = checked_minutes_to_seconds(first_mins);
    let second_seconds = checked_minutes_to_seconds(second_mins);
    let overflow = first_seconds.is_none() || second_seconds.is_none();
    let first_seconds = first_seconds.unwrap_or(0);
    let second_seconds = second_seconds.unwrap_or(0);

    warnings.push("interpreted_two_stage_slash_shorthand".to_string());
    Some(Ok(inferred_parsed(
        input,
        warnings,
        vec![
            Period {
                moves: None,
                base_seconds: first_seconds,
                increment_seconds: Some(first_inc),
            },
            Period {
                moves: None,
                base_seconds: second_seconds,
                increment_seconds: Some(second_inc),
            },
        ],
        overflow,
    )))
}

#[allow(clippy::ptr_arg)]
fn try_apostrophe_notation(
    input: &str,
    warnings: &mut Vec<String>,
) -> Option<Result<ParsedTimeControl, TimeControlError>> {
    if let Some(without_double) = input.strip_suffix("''") {
        if let Some(pos) = without_double.find('\'') {
            let minutes = parse_u32(&without_double[..pos])?;
            let mut seconds_str = &without_double[pos + 1..];
            if let Some(stripped) = seconds_str.strip_prefix('+') {
                seconds_str = stripped;
            }
            let seconds = if seconds_str.is_empty() {
                0
            } else {
                parse_u32(seconds_str)?
            };

            let base_seconds = checked_minutes_to_seconds(minutes);
            let overflow = base_seconds.is_none();
            let base_seconds = base_seconds.unwrap_or(0);

            warnings.push("interpreted_apostrophe_notation".to_string());

            return Some(Ok(inferred_parsed(
                input,
                warnings,
                vec![Period {
                    moves: None,
                    base_seconds,
                    increment_seconds: Some(seconds),
                }],
                overflow,
            )));
        }

        if let Some(seconds) = parse_u32(without_double) {
            warnings.push("interpreted_apostrophe_notation".to_string());
            return Some(Ok(ParsedTimeControl {
                raw: input.to_string(),
                normalized: Some(seconds.to_string()),
                periods: vec![Period {
                    moves: None,
                    base_seconds: seconds,
                    increment_seconds: None,
                }],
                mode: Mode::Normal,
                warnings: warnings.to_owned(),
                inferred: true,
                overflow: false,
            }));
        }
    }

    if let Some(without_single) = input.strip_suffix('\'')
        && !input.ends_with("''")
        && let Some(minutes) = parse_u32(without_single)
    {
        let base_seconds = checked_minutes_to_seconds(minutes);
        let overflow = base_seconds.is_none();
        let base_seconds = base_seconds.unwrap_or(0);

        warnings.push("interpreted_apostrophe_notation".to_string());
        return Some(Ok(inferred_parsed(
            input,
            warnings,
            vec![Period {
                moves: None,
                base_seconds,
                increment_seconds: None,
            }],
            overflow,
        )));
    }

    None
}

pub(super) fn try_free_text_templates(
    input: &str,
    warnings: &mut Vec<String>,
) -> Option<Result<ParsedTimeControl, TimeControlError>> {
    let lower = input.to_lowercase();

    let stage_caps = FREETEXT_FIDE_STAGE_RE.captures(&lower);

    if let Some(stage_caps) = stage_caps
        && let Some(base_mins) = stage_caps
            .get(1)
            .and_then(|m| m.as_str().parse::<u32>().ok())
        && let Some(moves) = stage_caps
            .get(2)
            .or_else(|| stage_caps.get(3))
            .and_then(|m| m.as_str().parse::<u32>().ok())
        && let Some(rest_caps) = FREETEXT_FIDE_REST_RE.captures(&lower)
        && let Some(rest_mins) = rest_caps
            .get(1)
            .and_then(|m| m.as_str().parse::<u32>().ok())
        && let Some(inc_caps) = FREETEXT_INC_RE.captures(&lower)
        && let Some(inc_secs) = inc_caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok())
    {
        let first_base = checked_minutes_to_seconds(base_mins);
        let second_base = checked_minutes_to_seconds(rest_mins);
        let overflow = first_base.is_none() || second_base.is_none();
        let first_base = first_base.unwrap_or(0);
        let second_base = second_base.unwrap_or(0);

        warnings.push("matched_free_text_template_fide_classical".to_string());

        return Some(Ok(inferred_parsed(
            input,
            warnings,
            vec![
                Period {
                    moves: Some(moves),
                    base_seconds: first_base,
                    increment_seconds: Some(inc_secs),
                },
                Period {
                    moves: None,
                    base_seconds: second_base,
                    increment_seconds: Some(inc_secs),
                },
            ],
            overflow,
        )));
    }

    if lower.contains("minute")
        || lower.contains("minut")
        || lower.contains("min")
        || lower.contains("sek")
        || lower.contains("sec")
    {
        let mut minutes: Option<u32> = None;
        let mut seconds: Option<u32> = None;
        let mut inc: Option<u32> = None;

        let mut template_text = lower.clone();

        if let Some(inc_cap) = FREETEXT_INC_RE.captures(&lower)
            && let Some(m) = inc_cap.get(1)
        {
            inc = m.as_str().parse().ok();
            template_text = FREETEXT_INC_RE.replace_all(&lower, " ").to_string();
        }

        for cap in FREETEXT_MINUTE_RE.captures_iter(&template_text) {
            if let Some(m) = cap.get(1) {
                minutes = m.as_str().parse().ok();
            }
        }

        for cap in FREETEXT_SECOND_RE.captures_iter(&template_text) {
            if let Some(m) = cap.get(1) {
                seconds = m.as_str().parse().ok();
            }
        }

        if let Some(mins) = minutes {
            let mins_seconds = checked_minutes_to_seconds(mins);
            let base = match (mins_seconds, seconds) {
                (Some(ms), Some(s)) => checked_compose_base_increment(ms, s),
                (Some(ms), None) => Some(ms),
                (None, _) => None,
            };
            let overflow = base.is_none();
            let base = base.unwrap_or(0);

            warnings.push("matched_free_text_template".to_string());

            return Some(Ok(inferred_parsed(
                input,
                warnings,
                vec![Period {
                    moves: None,
                    base_seconds: base,
                    increment_seconds: inc,
                }],
                overflow,
            )));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::super::parse_timecontrol;

    #[test]
    fn test_minute_shorthand_n_plus_i() {
        let result = parse_timecontrol("3+2").unwrap();
        assert_eq!(result.normalized, Some("180+2".to_string()));
        assert!(result.inferred);
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("interpreted_small_base_as_minutes"))
        );
    }

    #[test]
    fn test_minute_shorthand_15_plus_10() {
        let result = parse_timecontrol("15+10").unwrap();
        assert_eq!(result.normalized, Some("900+10".to_string()));
        assert!(result.inferred);
    }

    #[test]
    fn test_minute_shorthand_75_30() {
        let result = parse_timecontrol("75+30").unwrap();
        assert_eq!(result.normalized, Some("4500+30".to_string()));
        assert!(result.inferred);
    }

    #[test]
    fn test_minute_shorthand_90_30() {
        let result = parse_timecontrol("90+30").unwrap();
        assert_eq!(result.normalized, Some("5400+30".to_string()));
        assert!(result.inferred);
    }

    #[test]
    fn test_bare_minute_shorthand() {
        let result = parse_timecontrol("25").unwrap();
        assert_eq!(result.normalized, Some("1500".to_string()));
        assert!(result.inferred);
    }

    #[test]
    fn test_normalize_punctuation_pipe() {
        let result = parse_timecontrol("75 | 30").unwrap();
        assert_eq!(result.normalized, Some("4500+30".to_string()));
    }

    #[test]
    fn test_normalize_quoted() {
        let result = parse_timecontrol("\"180+2\"").unwrap();
        assert_eq!(result.normalized, Some("180+2".to_string()));
    }

    #[test]
    fn test_normalize_mixed_wrapper_quotes() {
        let result = parse_timecontrol("'\"180+2\"'").unwrap();
        assert_eq!(result.normalized, Some("180+2".to_string()));
        assert!(result.warnings.iter().any(|w| w == "stripped_quotes"));
    }

    #[test]
    fn test_normalize_repeated_outer_quote_noise_around_minute_shorthand() {
        let result = parse_timecontrol("''\"15 + 10\"''").unwrap();
        assert_eq!(result.normalized, Some("900+10".to_string()));
        assert!(result.warnings.iter().any(|w| w == "stripped_quotes"));
    }

    #[test]
    fn test_normalize_spaces_around_operators() {
        let result = parse_timecontrol("15 + 10").unwrap();
        assert_eq!(result.normalized, Some("900+10".to_string()));
    }

    #[test]
    fn test_normalize_spaces_around_plus_left_only() {
        let result = parse_timecontrol("15 +10").unwrap();
        assert_eq!(result.normalized, Some("900+10".to_string()));
    }

    #[test]
    fn test_normalize_spaces_around_plus_right_only() {
        let result = parse_timecontrol("15+ 10").unwrap();
        assert_eq!(result.normalized, Some("900+10".to_string()));
    }

    #[test]
    fn test_normalize_spaces_around_slash_and_colon_in_staged() {
        let result = parse_timecontrol("40 / 5400 + 30 : 1800 + 30").unwrap();
        assert_eq!(result.normalized, Some("40/5400+30:1800+30".to_string()));
    }

    #[test]
    fn test_normalize_mixed_operator_whitespace_in_stage_by_moves() {
        let result = parse_timecontrol("90 + 30 / 30 + 30").unwrap();
        assert_eq!(result.normalized, Some("5400+30:1800+30".to_string()));
    }

    #[test]
    fn test_apostrophe_notation() {
        let result = parse_timecontrol("10'+5''").unwrap();
        assert_eq!(result.normalized, Some("600+5".to_string()));
    }

    #[test]
    fn test_apostrophe_notation_with_spaces_is_preserved() {
        let result = parse_timecontrol("3' + 2''").unwrap();
        assert_eq!(result.normalized, Some("180+2".to_string()));
    }

    #[test]
    fn test_apostrophe_minutes_only() {
        let result = parse_timecontrol("10'").unwrap();
        assert_eq!(result.normalized, Some("600".to_string()));
        assert!(result.inferred);
    }

    #[test]
    fn test_apostrophe_seconds_only() {
        let result = parse_timecontrol("5''").unwrap();
        assert_eq!(result.normalized, Some("5".to_string()));
        assert!(result.inferred);
    }

    #[test]
    fn test_strip_trailing_apostrophe_for_spec_like_value() {
        let result = parse_timecontrol("90+30'").unwrap();
        assert_eq!(result.normalized, Some("5400+30".to_string()));
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w == "stripped_trailing_apostrophe")
        );
    }

    #[test]
    fn test_apostrophe_per_move_suffix() {
        let result = parse_timecontrol("3' + 2''/mv from move 1").unwrap();
        assert_eq!(result.normalized, Some("180+2".to_string()));
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w == "interpreted_apostrophe_per_move_suffix")
        );
    }

    #[test]
    fn test_apostrophe_per_move_suffix_larger_base() {
        let result = parse_timecontrol("15' + 10''/mv from move 1").unwrap();
        assert_eq!(result.normalized, Some("900+10".to_string()));
    }

    #[test]
    fn test_compact_minute_second_text_abbreviation() {
        let result = parse_timecontrol("3 mins + 2 seconds increment").unwrap();
        assert_eq!(result.normalized, Some("180+2".to_string()));
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w == "interpreted_compact_minute_second_increment")
        );
    }

    #[test]
    fn test_compact_classical_minute_second_text_abbreviation() {
        let result = parse_timecontrol("90 mins + 30 Secs").unwrap();
        assert_eq!(result.normalized, Some("5400+30".to_string()));
    }

    #[test]
    fn test_compact_minute_second_text_with_prefix_label() {
        let result = parse_timecontrol("Standard: 90mins + 30sec increment").unwrap();
        assert_eq!(result.normalized, Some("5400+30".to_string()));
    }

    #[test]
    fn test_compact_minute_second_without_minute_unit() {
        let result = parse_timecontrol("90+30 sec per move").unwrap();
        assert_eq!(result.normalized, Some("5400+30".to_string()));
    }

    #[test]
    fn test_compact_classical_with_trailing_qualifier_suffix() {
        let result = parse_timecontrol("90 + 30 OFICIAL").unwrap();
        assert_eq!(result.normalized, Some("5400+30".to_string()));
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w == "ignored_trailing_qualifier_suffix")
        );
    }

    #[test]
    fn test_suffix_with_digits_is_not_stripped() {
        let result = parse_timecontrol("90 + 30 round2").unwrap();
        assert_eq!(result.normalized, None);
    }

    #[test]
    fn test_malformed_suffix_with_whitespace_operators_still_fails() {
        // Regression tests: ensure inputs with digit-bearing suffixes still fail
        // (non-digit suffixes are stripped as trailing qualifiers - existing behavior)
        let result = parse_timecontrol("90 + 30 round2").unwrap();
        assert_eq!(result.normalized, None);

        // Verify that valid controls with digit-bearing suffixes still fail
        let result = parse_timecontrol("15 + 10 test123").unwrap();
        assert_eq!(result.normalized, None);

        let result = parse_timecontrol("40 / 5400 + 30 : 1800 + 30 abc456").unwrap();
        assert_eq!(result.normalized, None);

        // These should succeed because non-digit suffixes are stripped
        let result = parse_timecontrol("15 + 10 valid").unwrap();
        assert_eq!(result.normalized, Some("900+10".to_string()));
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w == "ignored_trailing_qualifier_suffix")
        );
    }

    #[test]
    fn test_clock_style_base_with_increment() {
        let result = parse_timecontrol("1:30.00 + 30 seconds increment from move 1").unwrap();
        assert_eq!(result.normalized, Some("5400+30".to_string()));
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w == "interpreted_clock_style_base")
        );
    }

    #[test]
    fn test_compact_fide_two_stage_with_game_token() {
        let result = parse_timecontrol("90'/40+30'/G+30''").unwrap();
        assert_eq!(result.normalized, Some("40/5400+30:1800+30".to_string()));
    }

    #[test]
    fn test_compact_fide_two_stage_with_end_token_and_ampersand() {
        let result = parse_timecontrol("90'/40m + 30'/end & 30/m").unwrap();
        assert_eq!(result.normalized, Some("40/5400+30:1800+30".to_string()));
    }

    #[test]
    fn test_compact_fide_two_stage_bonus_increment_wording() {
        let result = parse_timecontrol("90'/40 moves + 30' + 30'' bonus increment").unwrap();
        assert_eq!(result.normalized, Some("40/5400+30:1800+30".to_string()));
    }

    #[test]
    fn test_compact_fide_two_stage_additional_wording() {
        let result = parse_timecontrol("90mins+30second additional +30mins after move 40").unwrap();
        assert_eq!(result.normalized, Some("40/5400+30:1800+30".to_string()));
    }

    #[test]
    fn test_compact_fide_triple_plus() {
        let result = parse_timecontrol("90 + 30 + 30s per move").unwrap();
        assert_eq!(result.normalized, Some("40/5400+30:1800+30".to_string()));
    }

    #[test]
    fn test_two_stage_slash_shorthand_missing_move_qualifier() {
        let result = parse_timecontrol("90+30/30+30").unwrap();
        assert_eq!(result.normalized, Some("5400+30:1800+30".to_string()));
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w == "interpreted_two_stage_slash_shorthand")
        );
    }

    #[test]
    fn test_failure_unparseable() {
        let result = parse_timecontrol("klassisch").unwrap();
        assert_eq!(result.normalized, None);
    }

    #[test]
    fn test_unbalanced_quote_fragment_fails_safely() {
        let result = parse_timecontrol("\"90 + \"30").unwrap();
        assert_eq!(result.normalized, None);
        assert_eq!(result.mode, super::super::Mode::Unknown);
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w == "ambiguous_quote_residue")
        );
    }

    #[test]
    fn test_free_text_increment_not_counted_as_base_seconds() {
        let result = parse_timecontrol("game in 3 minutes + 2 seconds per move").unwrap();
        assert_eq!(result.normalized, Some("180+2".to_string()));
    }

    #[test]
    fn test_g_prefix_compact_n_plus_i() {
        let result = parse_timecontrol("g60+30").unwrap();
        assert_eq!(result.normalized, Some("3600+30".to_string()));
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w == "interpreted_g_prefix_as_minutes")
        );
    }

    #[test]
    fn test_g_prefix_slash_semicolon_inc_suffix() {
        let result = parse_timecontrol("G/90; +30inc").unwrap();
        assert_eq!(result.normalized, Some("5400+30".to_string()));
    }

    #[test]
    fn test_g_prefix_seconds_per_move() {
        let result = parse_timecontrol("G90 + 30 seconds/move").unwrap();
        assert_eq!(result.normalized, Some("5400+30".to_string()));
    }

    #[test]
    fn test_game_prefix_seconds_per_move() {
        let result = parse_timecontrol("Game/15 + 10 seconds per move").unwrap();
        assert_eq!(result.normalized, Some("900+10".to_string()));
    }

    #[test]
    fn test_g_prefix_added_per_move_text() {
        let result = parse_timecontrol("G: 15 min + 10 seconds added per move").unwrap();
        assert_eq!(result.normalized, Some("900+10".to_string()));
    }

    #[test]
    fn test_free_text_fide_classical_verbose_template() {
        let result = parse_timecontrol(
            "90 minutes for 40 moves + 30 minutes for the rest + 30 seconds per move from move one",
        )
        .unwrap();
        assert_eq!(result.normalized, Some("40/5400+30:1800+30".to_string()));
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w == "matched_free_text_template_fide_classical")
        );
    }

    #[test]
    fn test_free_text_fide_classical_compact_template() {
        let result = parse_timecontrol("90min./40 + 30min. + 30s./move").unwrap();
        assert_eq!(result.normalized, Some("40/5400+30:1800+30".to_string()));
    }

    #[test]
    fn test_free_text_increment_only_is_not_base_time() {
        let result = parse_timecontrol("30 sec per move").unwrap();
        assert_eq!(result.normalized, None);
    }

    #[test]
    fn test_overflow_boundary_exactly_at_limit() {
        let result = parse_timecontrol("G71582788").unwrap();
        assert!(!result.overflow);
        assert!(
            !result
                .warnings
                .contains(&"inference_arithmetic_overflow".to_string())
        );
        assert_eq!(result.normalized, Some("4294967280".to_string()));
    }

    #[test]
    fn test_overflow_just_over_limit() {
        let result = parse_timecontrol("G71582789").unwrap();
        assert!(result.overflow);
        assert!(
            result
                .warnings
                .contains(&"inference_arithmetic_overflow".to_string())
        );
        assert_eq!(result.normalized, None);
        assert!(result.periods.is_empty());
    }

    #[test]
    fn test_g_prefix_overflow() {
        let result = parse_timecontrol("G71582789").unwrap();
        assert!(result.overflow);
        assert!(
            result
                .warnings
                .contains(&"inference_arithmetic_overflow".to_string())
        );
        assert_eq!(result.normalized, None);
    }

    #[test]
    fn test_compact_minute_second_overflow() {
        let result = parse_timecontrol("71582789 mins + 30 seconds").unwrap();
        assert!(result.overflow);
        assert!(
            result
                .warnings
                .contains(&"inference_arithmetic_overflow".to_string())
        );
        assert_eq!(result.normalized, None);
    }

    #[test]
    fn test_clock_style_overflow() {
        let result = parse_timecontrol("1193047:00 + 30 sec").unwrap();
        assert!(result.overflow);
        assert!(
            result
                .warnings
                .contains(&"inference_arithmetic_overflow".to_string())
        );
        assert_eq!(result.normalized, None);
    }
}
