//! Source code span tracking for error reporting.

use serde::{Deserialize, Serialize};

/// Represents a location in source code (line, column, and byte offset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Location {
    #[must_use]
    pub const fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            offset: 0, // Will be updated by lexer
        }
    }

    #[must_use]
    pub const fn with_offset(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }
}

/// Represents a span of source code with start and end locations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start: Location,
    pub end: Location,
}

impl Span {
    #[must_use]
    pub const fn new(start: Location, end: Location) -> Self {
        Self { start, end }
    }

    /// Creates a span from byte offsets (for later conversion to line/column).
    #[must_use]
    pub const fn from_range(start: usize, end: usize) -> Self {
        Self {
            start: Location::with_offset(0, 0, start),
            end: Location::with_offset(0, 0, end),
        }
    }

    /// Checks if a location is contained within this span.
    #[must_use]
    pub fn contains(&self, location: Location) -> bool {
        if location.line < self.start.line || location.line > self.end.line {
            return false;
        }

        if location.line == self.start.line && location.column < self.start.column {
            return false;
        }

        if location.line == self.end.line && location.column > self.end.column {
            return false;
        }

        true
    }

    /// Merges two spans into a single span covering both.
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        let start = if self.start.line < other.start.line
            || (self.start.line == other.start.line && self.start.column < other.start.column)
        {
            self.start
        } else {
            other.start
        };

        let end = if self.end.line > other.end.line
            || (self.end.line == other.end.line && self.end.column > other.end.column)
        {
            self.end
        } else {
            other.end
        };

        Self { start, end }
    }
}
