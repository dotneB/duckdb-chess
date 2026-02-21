use std::error::Error;

use ::duckdb::vtab::arrow::WritableVector;
use ::duckdb::{
    Result,
    core::{DataChunkHandle, LogicalTypeHandle, LogicalTypeId},
    vscalar::{ScalarFunctionSignature, VScalar},
};

use super::duckdb_impl::scalar::{
    VarcharNullBehavior, VarcharOutput, invoke_unary_varchar_to_varchar,
};

mod inference;
mod json;
mod strict;

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
            .map(strict::format_period)
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

pub fn parse_timecontrol(raw: &str) -> Result<ParsedTimeControl, TimeControlError> {
    let input = raw.trim();
    if input.is_empty() {
        return Err(TimeControlError {
            message: "empty input".to_string(),
        });
    }

    let preprocessed = inference::preprocess(input);
    let mut warnings = preprocessed.warnings;

    if preprocessed.has_ambiguous_quote_residue {
        return Ok(ParsedTimeControl {
            raw: raw.to_string(),
            normalized: None,
            periods: Vec::new(),
            mode: Mode::Unknown,
            warnings,
            inferred: false,
            overflow: false,
        });
    }

    if let Some(result) = strict::try_strict_parse(&preprocessed.normalized, &mut warnings) {
        return with_original_raw(raw, result);
    }

    if let Some(result) = inference::try_inference(&preprocessed.normalized, &mut warnings) {
        return with_original_raw(raw, result);
    }

    if let Some(result) =
        inference::try_free_text_templates(&preprocessed.normalized, &mut warnings)
    {
        return with_original_raw(raw, result);
    }

    if let Some(core) = inference::strip_trailing_qualifier_suffix(&preprocessed.normalized) {
        let mut fallback_warnings = warnings.clone();
        fallback_warnings.push("ignored_trailing_qualifier_suffix".to_string());

        if let Some(result) = strict::try_strict_parse(&core, &mut fallback_warnings) {
            return with_original_raw(raw, result);
        }

        if let Some(result) = inference::try_inference(&core, &mut fallback_warnings) {
            return with_original_raw(raw, result);
        }

        if let Some(result) = inference::try_free_text_templates(&core, &mut fallback_warnings) {
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
    json::timecontrol_to_json(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_category_returns_none_on_overflow() {
        let result = parse_timecontrol("G71582789").unwrap();
        assert!(result.overflow);
        assert_eq!(category_from_parsed_timecontrol(&result), None);
    }
}
