use duckdb::vtab::arrow::WritableVector;
use duckdb::{
    Result,
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vscalar::{ScalarFunctionSignature, VScalar},
};
use libduckdb_sys::duckdb_string_t;
use std::error::Error;
use std::ffi::CString;

use crate::chess::duckdb_string::decode_duckdb_string;

pub struct ChessTimecontrolNormalizeScalar;

impl VScalar for ChessTimecontrolNormalizeScalar {
    type State = ();

    unsafe fn invoke(
        _state: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> Result<(), Box<dyn Error>> {
        let len = input.len();
        let input_vec = input.flat_vector(0);
        let mut output_vec = output.flat_vector();

        let input_slice = input_vec.as_slice::<duckdb_string_t>();

        for (i, s) in input_slice.iter().take(len).enumerate() {
            if input_vec.row_is_null(i as u64) {
                output_vec.set_null(i);
                continue;
            }

            let val = unsafe { decode_duckdb_string(s) };
            match normalize_timecontrol(val.as_ref()) {
                Some(normalized) => {
                    output_vec.insert(i, CString::new(normalized)?);
                }
                None => {
                    output_vec.set_null(i);
                }
            }
        }

        Ok(())
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
        let len = input.len();
        let input_vec = input.flat_vector(0);
        let mut output_vec = output.flat_vector();

        let input_slice = input_vec.as_slice::<duckdb_string_t>();

        for (i, s) in input_slice.iter().take(len).enumerate() {
            if input_vec.row_is_null(i as u64) {
                output_vec.set_null(i);
                continue;
            }

            let val = unsafe { decode_duckdb_string(s) };
            match parse_timecontrol(val.as_ref()) {
                Ok(parsed) => {
                    let json = timecontrol_to_json(&parsed);
                    output_vec.insert(i, CString::new(json)?);
                }
                Err(_) => {
                    let parsed = ParsedTimeControl {
                        raw: val.into_owned(),
                        normalized: None,
                        periods: Vec::new(),
                        mode: Mode::Unknown,
                        warnings: vec!["parse_error".to_string()],
                        inferred: false,
                    };
                    let json = timecontrol_to_json(&parsed);
                    output_vec.insert(i, CString::new(json)?);
                }
            }
        }

        Ok(())
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
        let len = input.len();
        let input_vec = input.flat_vector(0);
        let mut output_vec = output.flat_vector();

        let input_slice = input_vec.as_slice::<duckdb_string_t>();

        for (i, s) in input_slice.iter().take(len).enumerate() {
            if input_vec.row_is_null(i as u64) {
                output_vec.set_null(i);
                continue;
            }

            let val = unsafe { decode_duckdb_string(s) };
            match categorize_timecontrol(val.as_ref()) {
                Some(category) => {
                    output_vec.insert(i, CString::new(category)?);
                }
                None => {
                    output_vec.set_null(i);
                }
            }
        }

        Ok(())
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

    Ok(ParsedTimeControl {
        raw: raw.to_string(),
        normalized: None,
        periods: Vec::new(),
        mode: Mode::Unknown,
        warnings,
        inferred: false,
    })
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

    if let Some(result) = try_apostrophe_notation(input, warnings) {
        return Some(result);
    }

    if input.contains('+') {
        let parts: Vec<&str> = input.split('+').collect();
        if parts.len() == 2
            && let (Some(base), Some(inc)) = (parse_u32(parts[0]), parse_u32(parts[1]))
        {
            if base < 60 && inc <= 60 {
                let normalized = format!("{}+{}", base * 60, inc);
                warnings.push("interpreted_small_base_as_minutes".to_string());
                return Some(Ok(ParsedTimeControl {
                    raw: input.to_string(),
                    normalized: Some(normalized),
                    periods: vec![Period {
                        moves: None,
                        base_seconds: base * 60,
                        increment_seconds: Some(inc),
                    }],
                    mode: Mode::Normal,
                    warnings: warnings.to_owned(),
                    inferred: true,
                }));
            }

            if (base == 75 || base == 90) && inc == 30 {
                let normalized = format!("{}+{}", base * 60, inc);
                warnings.push("interpreted_classical_75_90_as_minutes".to_string());
                return Some(Ok(ParsedTimeControl {
                    raw: input.to_string(),
                    normalized: Some(normalized),
                    periods: vec![Period {
                        moves: None,
                        base_seconds: base * 60,
                        increment_seconds: Some(inc),
                    }],
                    mode: Mode::Normal,
                    warnings: warnings.to_owned(),
                    inferred: true,
                }));
            }
        }
    }

    if !input.contains('+')
        && !input.contains('/')
        && !input.contains(':')
        && let Some(n) = parse_u32(input)
        && n < 60
    {
        let normalized = (n * 60).to_string();
        warnings.push("interpreted_small_bare_number_as_minutes".to_string());
        return Some(Ok(ParsedTimeControl {
            raw: input.to_string(),
            normalized: Some(normalized),
            periods: vec![Period {
                moves: None,
                base_seconds: n * 60,
                increment_seconds: None,
            }],
            mode: Mode::Normal,
            warnings: warnings.to_owned(),
            inferred: true,
        }));
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

    let g_numeric_re = regex::Regex::new(
        r"^(\d+)\s*(?:\+\s*(\d+)\s*(?:inc|(?:seconds?|secs?|sec\.?|s\.?|sek)\s*(?:(?:added\s+)?per\s*move|/move|/mv|/m)?)?)?$",
    )
    .ok()?;

    if let Some(caps) = g_numeric_re.captures(candidate.trim()) {
        let base_mins = caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok())?;
        let increment = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok());

        let base_seconds = base_mins * 60;
        let normalized = match increment {
            Some(inc) => format!("{}+{}", base_seconds, inc),
            None => base_seconds.to_string(),
        };

        warnings.push("interpreted_g_prefix_as_minutes".to_string());

        return Some(Ok(ParsedTimeControl {
            raw: input.to_string(),
            normalized: Some(normalized),
            periods: vec![Period {
                moves: None,
                base_seconds,
                increment_seconds: increment,
            }],
            mode: Mode::Normal,
            warnings: warnings.to_owned(),
            inferred: true,
        }));
    }

    None
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

            let normalized = format!("{}+{}", minutes * 60, seconds);
            warnings.push("interpreted_apostrophe_notation".to_string());

            return Some(Ok(ParsedTimeControl {
                raw: input.to_string(),
                normalized: Some(normalized),
                periods: vec![Period {
                    moves: None,
                    base_seconds: minutes * 60,
                    increment_seconds: Some(seconds),
                }],
                mode: Mode::Normal,
                warnings: warnings.to_owned(),
                inferred: true,
            }));
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
            }));
        }
    }

    if let Some(without_single) = input.strip_suffix('\'')
        && !input.ends_with("''")
        && let Some(minutes) = parse_u32(without_single)
    {
        warnings.push("interpreted_apostrophe_notation".to_string());
        return Some(Ok(ParsedTimeControl {
            raw: input.to_string(),
            normalized: Some((minutes * 60).to_string()),
            periods: vec![Period {
                moves: None,
                base_seconds: minutes * 60,
                increment_seconds: None,
            }],
            mode: Mode::Normal,
            warnings: warnings.to_owned(),
            inferred: true,
        }));
    }

    None
}

#[allow(clippy::ptr_arg)]
fn try_free_text_templates(
    input: &str,
    warnings: &mut Vec<String>,
) -> Option<Result<ParsedTimeControl, TimeControlError>> {
    let lower = input.to_lowercase();

    let fide_stage_for_re = regex::Regex::new(
        r"(\d+)\s*(?:minutes?|min(?:utes?)?|min\.)\s*for\s*(\d+)\s*(?:moves?|mv|mvs?)",
    )
    .ok()?;
    let fide_stage_slash_re =
        regex::Regex::new(r"(\d+)\s*(?:minutes?|min(?:utes?)?|min\.)\s*/\s*(\d+)").ok()?;
    let fide_rest_re = regex::Regex::new(
        r"(?:\+|,|then\s+)\s*(\d+)\s*(?:minutes?|min(?:utes?)?|min\.)\s*(?:for\s*(?:the\s*)?rest|rest)?",
    )
    .ok()?;
    let fide_inc_re = regex::Regex::new(
        r"(\d+)\s*(?:seconds?|secs?|sec\.?|s\.?|sek)\s*(?:(?:added\s+)?per\s*move|/move|/mv|/m)",
    )
    .ok()?;

    let stage_caps = fide_stage_for_re
        .captures(&lower)
        .or_else(|| fide_stage_slash_re.captures(&lower));

    if let Some(stage_caps) = stage_caps
        && let (Some(base_mins), Some(moves)) = (
            stage_caps
                .get(1)
                .and_then(|m| m.as_str().parse::<u32>().ok()),
            stage_caps
                .get(2)
                .and_then(|m| m.as_str().parse::<u32>().ok()),
        )
        && let Some(rest_caps) = fide_rest_re.captures(&lower)
        && let Some(rest_mins) = rest_caps
            .get(1)
            .and_then(|m| m.as_str().parse::<u32>().ok())
        && let Some(inc_caps) = fide_inc_re.captures(&lower)
        && let Some(inc_secs) = inc_caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok())
    {
        let first_base = base_mins * 60;
        let second_base = rest_mins * 60;
        let normalized = format!(
            "{}/{}+{}:{}+{}",
            moves, first_base, inc_secs, second_base, inc_secs
        );

        warnings.push("matched_free_text_template_fide_classical".to_string());

        return Some(Ok(ParsedTimeControl {
            raw: input.to_string(),
            normalized: Some(normalized),
            periods: vec![
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
            mode: Mode::Normal,
            warnings: warnings.to_owned(),
            inferred: true,
        }));
    }

    if lower.contains("minute")
        || lower.contains("minut")
        || lower.contains("min")
        || lower.contains("sek")
        || lower.contains("sec")
    {
        let minute_re = regex::Regex::new(r"(\d+)\s*(?:minutes?|min(?:utes?)?|min\.)\b").ok()?;
        let second_re = regex::Regex::new(r"(\d+)\s*(?:seconds?|secs?|sek)\b").ok()?;
        let inc_re = regex::Regex::new(
            r"(\d+)\s*(?:seconds?|secs?|sec\.?|s\.?|sek)\s*(?:(?:added\s+)?per\s*move|/move|/mv|/m)",
        )
        .ok()?;

        let mut minutes: Option<u32> = None;
        let mut seconds: Option<u32> = None;
        let mut inc: Option<u32> = None;

        let mut template_text = lower.clone();

        if let Some(inc_cap) = inc_re.captures(&lower)
            && let Some(m) = inc_cap.get(1)
        {
            inc = m.as_str().parse().ok();
            template_text = inc_re.replace_all(&lower, " ").to_string();
        }

        for cap in minute_re.captures_iter(&template_text) {
            if let Some(m) = cap.get(1) {
                minutes = m.as_str().parse().ok();
            }
        }

        for cap in second_re.captures_iter(&template_text) {
            if let Some(m) = cap.get(1) {
                seconds = m.as_str().parse().ok();
            }
        }

        if let Some(mins) = minutes {
            let base = mins * 60 + seconds.unwrap_or(0);
            let normalized = match inc {
                Some(i) => format!("{}+{}", base, i),
                None => base.to_string(),
            };

            warnings.push("matched_free_text_template".to_string());

            return Some(Ok(ParsedTimeControl {
                raw: input.to_string(),
                normalized: Some(normalized),
                periods: vec![Period {
                    moves: None,
                    base_seconds: base,
                    increment_seconds: inc,
                }],
                mode: Mode::Normal,
                warnings: warnings.to_owned(),
                inferred: true,
            }));
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
    if parsed.mode != Mode::Normal {
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
        r#"{{"raw":{},"normalized":{},"mode":"{}","periods":[{}],"warnings":{},"inferred":{}}}"#,
        raw_json,
        normalized_json,
        mode_str,
        periods_json.join(","),
        warnings_json,
        if parsed.inferred { "true" } else { "false" }
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
}
