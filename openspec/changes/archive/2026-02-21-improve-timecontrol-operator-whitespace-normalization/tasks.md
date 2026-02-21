## 1. Parser and normalization updates

- [x] 1.1 Add failing unit tests for optional whitespace around `+`, `/`, and `:` operators in valid `TimeControl` values.
- [x] 1.2 Implement operator-boundary whitespace preprocessing in the normalization path while preserving existing strict token parsing.
- [x] 1.3 Ensure canonical normalized output strips operator-adjacent whitespace and matches existing compact forms.

## 2. Regression and SQL behavior coverage

- [x] 2.1 Add or update SQLLogicTests that verify whitespace-padded operator forms normalize correctly through SQL functions.
- [x] 2.2 Add regression tests confirming malformed suffix-bearing inputs remain NULL and are not newly accepted.
- [x] 2.3 Run `just dev` and fix any failures to confirm formatting, linting, unit tests, and SQLLogicTests pass.
