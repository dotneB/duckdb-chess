#[derive(Debug, Clone, Default)]
pub struct ErrorAccumulator(Option<String>);

impl ErrorAccumulator {
    pub fn push(&mut self, msg: &str) {
        match &mut self.0 {
            Some(existing) => {
                existing.push_str("; ");
                existing.push_str(msg);
            }
            None => {
                self.0 = Some(msg.to_string());
            }
        }
    }

    pub fn take(&mut self) -> Option<String> {
        self.0.take()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorAccumulator;

    #[test]
    fn test_push_single_message() {
        let mut accumulator = ErrorAccumulator::default();
        accumulator.push("first error");

        assert_eq!(accumulator.take().as_deref(), Some("first error"));
    }

    #[test]
    fn test_push_multiple_messages_uses_separator() {
        let mut accumulator = ErrorAccumulator::default();
        accumulator.push("first");
        accumulator.push("second");

        assert_eq!(accumulator.take().as_deref(), Some("first; second"));
    }

    #[test]
    fn test_take_consumes_accumulator() {
        let mut accumulator = ErrorAccumulator::default();
        accumulator.push("error");

        assert_eq!(accumulator.take().as_deref(), Some("error"));
        assert!(accumulator.is_empty());
        assert!(accumulator.take().is_none());
    }

    #[test]
    fn test_default_is_empty() {
        let accumulator = ErrorAccumulator::default();
        assert!(accumulator.is_empty());
    }
}
