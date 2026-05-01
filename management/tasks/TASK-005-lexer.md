# TASK-005: PDF Lexer / Tokenizer

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [x] Done

## Owner
@jules

## Objective
Convert a stream of raw bytes into discrete `PdfToken`s (e.g., Number, Name, String, DictionaryStart). The Lexer does *not* build the AST; it only categorizes syntax.

## Acceptance Criteria
- [ ] Define `PdfToken` enum.
- [ ] Implement a `Lexer` struct that consumes `&[u8]`.
- [ ] Handle delimiters (`<<`, `>>`, `[`, `]`, `(`, `)`, `/`).
- [ ] Handle whitespace and comments (`%`).
- [ ] Handle Literal Strings `(...)` and Hex Strings `<...>`.
