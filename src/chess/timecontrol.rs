use duckdb::vtab::arrow::WritableVector;
use duckdb::{
    Result,
    core::{DataChunkHandle, LogicalTypeHandle, LogicalTypeId},
    vscalar::{ScalarFunctionSignature, VScalar},
};
use std::error::Error;
use std::sync::LazyLock;

use super::scalar::{VarcharNullBehavior, VarcharOutput, invoke_unary_varchar_to_varchar};

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

pub struct ChessTimecontrolNormalizeScalar;

impl VScalar for ChessTimecontrolNormalizeScalar {
    type State = ();

    unsafe fn invoke(
        _state: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> Result<(), Box<dyn Error>> {
        invoke_unary_varchar_to_varchar(input, output, VarcharNullBehavior::Null, |timecontrol| {
            Ok(match normalize_timecontrol(timecontrol) {
                Some(normalized) => VarcharOutput::Value(normalized),
                None => VarcharOutput::Null,
            })
        })
    }

    fn signatures() -> Vec<ScalarFunctionSignature> {
        vec![ScalarFunctionSignature::exact(
            vec![LogicalTypeHandle::from(LogicalTypeId::Varchar)],
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        )]
    }
}

pub struct ChessTimecontrolJsonScalar;

impl VScalar for ChessTimecontrolJsonScalar {
    type State = ();

    unsafe fn invoke(
        _state: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> Result<(), Box<dyn Error>> {
        invoke_unary_varchar_to_varchar(input, output, VarcharNullBehavior::Null, |timecontrol| {
            let json = match parse_timecontrol(timecontrol) {
                Ok(parsed) => timecontrol_to_json(&parsed),
                Err(_) => {
                    let parsed = ParsedTimeControl {
                        raw: timecontrol.to_string(),
                        normalized: None,
                        periods: Vec::new(),
                        mode: Mode::Unknown,
                        warnings: vec!["parse_error".to_string()],
                        inferred: false,
                        overflow: false,
                    };
                    timecontrol_to_json(&parsed)
                }
            };

            Ok(VarcharOutput::Value(json))
        })
    }

    fn signatures() -> Vec<ScalarFunctionSignature> {
        vec![ScalarFunctionSignature::exact(
            vec![LogicalTypeHandle::from(LogicalTypeId::Varchar)],
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        )]
    }
}

pub struct ChessTimecontrolCategoryScalar;

impl VScalar for ChessTimecontrolCategoryScalar {
    type State = ();

    unsafe fn invoke(
        _state: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> Result<(), Box<dyn Error>> {
        invoke_unary_varchar_to_varchar(input, output, VarcharNullBehavior::Null, |timecontrol| {
            Ok(match categorize_timecontrol(timecontrol) {
                Some(category) => VarcharOutput::Value(category.to_string()),
                None => VarcharOutput::Null,
            })
        })
    }

    fn signatures() -> Vec<ScalarFunctionSignature> {
        vec![ScalarFunctionSignature::exact(
            vec![LogicalTypeHandle::from(LogicalTypeId::Varchar)],
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        )]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Period {
    pub moves: Option<u32>,
    pub base_seconds: u32,
    pub increment_seconds: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Unknown,
    Unlimited,
    Sandclock,
    Normal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedTimeControl {
    pub raw: String,
    pub normalized: Option<String>,
    pub periods: Vec<Period>,
    pub mode: Mode,
    pub warnings: Vec<String>,
    pub inferred: bool,
    pub overflow: bool,
}

#[derive(Debug, Clone)]
pub struct TimeControlError {
    pub message: String,
}

impl std::fmt::Display for TimeControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for TimeControlError {}

fn parse_u32(s: &str) -> Option<u32> {
    s.parse().ok()
}

const OVERFLOW_WARNING: &str = "inference_arithmetic_overflow";

fn checked_minutes_to_seconds(minutes: u32) -> Option<u32> {
    minutes.checked_mul(60)
}

fn checked_compose_base_increment(base_seconds: u32, inc_seconds: u32) -> Option<u32> {
    base_seconds.checked_add(inc_seconds)
}

fn checked_hours_minutes_to_seconds(hours: u32, minutes: u32) -> Option<u32> {
    let hours_seconds = hours.checked_mul(3600)?;
    let minutes_seconds = minutes.checked_mul(60)?;
    hours_seconds.checked_add(minutes_seconds)
}

fn with_original_raw(
    raw: &str,
    result: Result<ParsedTimeControl, TimeControlError>,
) -> Result<ParsedTimeControl, TimeControlError> {
    result.map(|mut parsed| {
        parsed.raw = raw.to_string();
        parsed
    })
}

pub fn parse_timecontrol(raw: &str) -> Result<ParsedTimeControl, TimeControlError> {
    let input = raw.trim();
    if input.is_empty() {
        return Err(TimeControlError {
            message: "empty input".to_string(),
        });
    }

    let (preprocessed, mut warnings) = preprocess(input);

    if let Some(result) = try_strict_parse(&preprocessed, &mut warnings) {
        return with_original_raw(raw, result);
    }

    if let Some(result) = try_inference(&preprocessed, &mut warnings) {
        return with_original_raw(raw, result);
    }

    if let Some(result) = try_free_text_templates(&preprocessed, &mut warnings) {
        return with_original_raw(raw, result);
    }

    if let Some(core) = strip_trailing_qualifier_suffix(&preprocessed) {
        let mut fallback_warnings = warnings.clone();
        fallback_warnings.push("ignored_trailing_qualifier_suffix".to_string());

        if let Some(result) = try_strict_parse(&core, &mut fallback_warnings) {
            return with_original_raw(raw, result);
        }

        if let Some(result) = try_inference(&core, &mut fallback_warnings) {
            return with_original_raw(raw, result);
        }

        if let Some(result) = try_free_text_templates(&core, &mut fallback_warnings) {
            return with_original_raw(raw, result);
        }
    }

    Ok(ParsedTimeControl {
        raw: raw.to_string(),
        normalized: None,
        periods: Vec::new(),
        mode: Mode::Unknown,
        warnings,
        inferred: false,
        overflow: false,
    })
}

fn strip_trailing_qualifier_suffix(input: &str) -> Option<String> {
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

fn preprocess(input: &str) -> (String, Vec<String>) {
    let mut warnings = Vec::new();
    let mut s = input.to_string();

    let original = s.clone();

    s = s.trim().to_string();
    if s != original {
        warnings.push("trimmed".to_string());
    }

    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s = s[1..s.len() - 1].to_string();
        if !s.is_empty() {
            warnings.push("stripped_quotes".to_string());
        }
    }

    let original = s.clone();
    s = s
        .replace(" + ", "+")
        .replace(" - ", "-")
        .replace(" / ", "/")
        .replace(" : ", ":");
    if s != original {
        warnings.push("normalized_operator_whitespace".to_string());
    }

    let original = s.clone();
    s = s.replace(['|', '_'], "+");
    if s != original {
        warnings.push("mapped_separator".to_string());

        // Normalize whitespace around + after mapping separator
        let original2 = s.clone();
        s = s.replace(" + ", "+");
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

    (s, warnings)
}

#[allow(clippy::ptr_arg)]
fn try_strict_parse(
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

fn parse_stage(s: &str) -> Option<Period> {
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

fn format_period(p: &Period) -> String {
    let base = match p.moves {
        Some(m) => format!("{}/{}", m, p.base_seconds),
        None => p.base_seconds.to_string(),
    };

    match p.increment_seconds {
        Some(inc) => format!("{}+{}", base, inc),
        None => base,
    }
}

fn try_inference(
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

fn inferred_parsed(
    input: &str,
    warnings: &mut Vec<String>,
    periods: Vec<Period>,
    overflow: bool,
) -> ParsedTimeControl {
    if overflow {
        warnings.push(OVERFLOW_WARNING.to_string());
        ParsedTimeControl {
            raw: input.to_string(),
            normalized: None,
            periods: Vec::new(),
            mode: Mode::Normal,
            warnings: warnings.to_vec(),
            inferred: true,
            overflow: true,
        }
    } else {
        let normalized = periods
            .iter()
            .map(format_period)
            .collect::<Vec<_>>()
            .join(":");

        ParsedTimeControl {
            raw: input.to_string(),
            normalized: Some(normalized),
            periods,
            mode: Mode::Normal,
            warnings: warnings.to_vec(),
            inferred: true,
            overflow: false,
        }
    }
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
    // Parse apostrophe notation:
    // - N' => minutes
    // - N'' => seconds
    // - N'+X'' => minutes plus increment seconds
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

#[allow(clippy::ptr_arg)]
fn try_free_text_templates(
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

pub fn normalize_timecontrol(raw: &str) -> Option<String> {
    match parse_timecontrol(raw) {
        Ok(parsed) => parsed.normalized,
        Err(_) => None,
    }
}

pub fn category_from_parsed_timecontrol(parsed: &ParsedTimeControl) -> Option<&'static str> {
    if parsed.mode != Mode::Normal || parsed.overflow {
        return None;
    }

    let period = parsed.periods.first()?;
    let increment = period.increment_seconds.unwrap_or(0) as u64;
    let estimated_seconds = period.base_seconds as u64 + 40 * increment;

    match estimated_seconds {
        0..=29 => Some("ultra-bullet"),
        30..=179 => Some("bullet"),
        180..=479 => Some("blitz"),
        480..=1499 => Some("rapid"),
        _ => Some("classical"),
    }
}

pub fn categorize_timecontrol(raw: &str) -> Option<&'static str> {
    let parsed = parse_timecontrol(raw).ok()?;
    category_from_parsed_timecontrol(&parsed)
}

pub fn timecontrol_to_json(parsed: &ParsedTimeControl) -> String {
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
    use super::*;

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
    fn test_dont_reinterpret_large_values() {
        let result = parse_timecontrol("900+10").unwrap();
        assert_eq!(result.normalized, Some("900+10".to_string()));
        assert!(!result.inferred);
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
    fn test_normalize_spaces_around_operators() {
        let result = parse_timecontrol("15 + 10").unwrap();
        assert_eq!(result.normalized, Some("900+10".to_string()));
    }

    #[test]
    fn test_apostrophe_notation() {
        let result = parse_timecontrol("10'+5''").unwrap();
        assert_eq!(result.normalized, Some("600+5".to_string()));
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
    fn test_invalid_stage_with_non_numeric_moves_fails() {
        let result = parse_timecontrol("x/600+5").unwrap();
        assert_eq!(result.normalized, None);
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
    fn test_raw_preserves_original_input() {
        let result = parse_timecontrol(" 15 + 10 ").unwrap();
        assert_eq!(result.raw, " 15 + 10 ");
        assert_eq!(result.normalized, Some("900+10".to_string()));
    }

    #[test]
    fn test_normalize_function() {
        assert_eq!(normalize_timecontrol("3+2"), Some("180+2".to_string()));
        assert_eq!(normalize_timecontrol("180+2"), Some("180+2".to_string()));
        assert_eq!(normalize_timecontrol("?"), Some("?".to_string()));
        assert_eq!(normalize_timecontrol("invalid"), None);
    }

    #[test]
    fn test_category_threshold_boundaries() {
        assert_eq!(categorize_timecontrol("29''"), Some("ultra-bullet"));
        assert_eq!(categorize_timecontrol("30''"), Some("bullet"));
        assert_eq!(categorize_timecontrol("179''"), Some("bullet"));
        assert_eq!(categorize_timecontrol("180+0"), Some("blitz"));
        assert_eq!(categorize_timecontrol("479+0"), Some("blitz"));
        assert_eq!(categorize_timecontrol("480+0"), Some("rapid"));
        assert_eq!(categorize_timecontrol("1499+0"), Some("rapid"));
        assert_eq!(categorize_timecontrol("1500+0"), Some("classical"));
    }

    #[test]
    fn test_category_increment_driven_case() {
        assert_eq!(categorize_timecontrol("2+12"), Some("rapid"));
    }

    #[test]
    fn test_category_respects_small_base_minute_inference() {
        assert_eq!(categorize_timecontrol("29+0"), Some("classical"));
        assert_eq!(categorize_timecontrol("29''"), Some("ultra-bullet"));
    }

    #[test]
    fn test_category_returns_none_for_non_normal_modes_and_invalid() {
        assert_eq!(categorize_timecontrol("?"), None);
        assert_eq!(categorize_timecontrol("-"), None);
        assert_eq!(categorize_timecontrol("*60"), None);
        assert_eq!(categorize_timecontrol("klassisch"), None);
    }

    #[test]
    fn test_category_returns_none_when_normal_mode_has_no_periods() {
        let parsed = ParsedTimeControl {
            raw: "n/a".to_string(),
            normalized: Some("n/a".to_string()),
            periods: Vec::new(),
            mode: Mode::Normal,
            warnings: Vec::new(),
            inferred: false,
            overflow: false,
        };

        assert_eq!(category_from_parsed_timecontrol(&parsed), None);
    }

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
    fn test_overflow_boundary_exactly_at_limit() {
        // u32::MAX = 4294967295
        // 4294967295 / 60 = 71582788 with remainder 15
        // So 71582788 minutes = 4294967280 seconds, which is within limit
        // Use G-prefix format to ensure it goes through inference path
        let result = parse_timecontrol("G71582788").unwrap();
        // This should NOT overflow since 71582788 * 60 = 4294967280 < 4294967295
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
        // 71582789 minutes * 60 = 4294967340 > u32::MAX
        // Use G-prefix format to ensure it goes through inference path
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
    fn test_category_returns_none_on_overflow() {
        let result = parse_timecontrol("G71582789").unwrap();
        assert!(result.overflow);
        assert_eq!(category_from_parsed_timecontrol(&result), None);
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

    #[test]
    fn test_non_overflow_inference_unchanged() {
        // Small values should work exactly as before
        let result = parse_timecontrol("3+2").unwrap();
        assert!(!result.overflow);
        assert_eq!(result.normalized, Some("180+2".to_string()));
        assert!(
            !result
                .warnings
                .contains(&"inference_arithmetic_overflow".to_string())
        );
        assert_eq!(categorize_timecontrol("3+2"), Some("blitz"));
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
        // 1193047 hours * 3600 = 4294969200 > u32::MAX
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
