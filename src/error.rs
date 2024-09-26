use crate::ast;

/// An error that occurred during parsing or compiling a regular expression.
#[derive(Clone, PartialEq, Debug)]
pub enum Error {
    /// A syntax error.
    Syntax(String),
}

impl Error {
    pub(crate) fn from_ast_parse_error(err: ast::Error) -> Error {
        Error::Syntax(err.to_string())
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Error::Syntax(ref x) => x.fmt(f),
        }
    }
}

/// A helper type for formatting nice error messages.
///
/// This type is responsible for reporting regex parse errors in a nice human
/// readable format. Most of its complexity is from interspersing notational
/// markers pointing out the position where an error occurred.
#[derive(Debug)]
pub struct Formatter<'e, E> {
    /// The original regex pattern in which the error occurred.
    pattern: &'e str,
    /// The error kind. It must impl fmt::Display.
    err: &'e E,
    /// The primary span of the error.
    span: &'e ast::Span,
}

impl<'e> From<&'e ast::Error> for Formatter<'e, ast::ErrorKind> {
    fn from(err: &'e ast::Error) -> Self {
        Formatter {
            pattern: err.pattern(),
            err: err.kind(),
            span: err.span(),
        }
    }
}

impl<'e, E: core::fmt::Display> core::fmt::Display for Formatter<'e, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // let _spans = Spans::from_formatter(self);
        // writeln!(f, "regex parse error:")?;
        // let notated = Spans::from_formatter(self).notate();
        // write!(f, "{}", notated)?;
        // write!(f, "error: {}", self.err)?;
        Ok(())
    }
}

/// This type represents an arbitrary number of error spans in a way that makes
/// it convenient to notate the regex pattern. ("Notate" means "point out
/// exactly where the error occurred in the regex pattern.")
///
/// Technically, we can only ever have two spans given our current error
/// structure. However, after toiling with a specific algorithm for handling
/// two spans, it became obvious that an algorithm to handle an arbitrary
/// number of spans was actually much simpler.
struct Spans<'p> {
    /// The original regex pattern string.
    pattern: &'p str,
    // /// The total width that should be used for line numbers. The width is
    // /// used for left padding the line numbers for alignment.
    // ///
    // /// A value of `0` means line numbers should not be displayed. That is,
    // /// the pattern is itself only one line.
    // line_number_width: usize,
    // /// All error spans that occur on a single line. This sequence always has
    // /// length equivalent to the number of lines in `pattern`, where the index
    // /// of the sequence represents a line number, starting at `0`. The spans
    // /// in each line are sorted in ascending order.
    // by_line: Vec<Vec<ast::Span>>,
    // /// All error spans that occur over one or more lines. That is, the start
    // /// and end position of the span have different line numbers. The spans are
    // /// sorted in ascending order.
    // multi_line: Vec<ast::Span>,
}

impl<'p> Spans<'p> {
    /// Build a sequence of spans from a formatter.
    fn from_formatter<'e, E: core::fmt::Display>(fmter: &'p Formatter<'e, E>) -> Spans<'p> {
        // let mut line_count = fmter.pattern.lines().count();
        // // If the pattern ends with a `\n` literal, then our line count is
        // // off by one, since a span can occur immediately after the last `\n`,
        // // which is consider to be an additional line.
        // if fmter.pattern.ends_with('\n') {
        //     line_count += 1;
        // }
        // let line_number_width = if line_count <= 1 {
        //     0
        // } else {
        //     line_count.to_string().len()
        // };
        let spans = Spans {
            pattern: &fmter.pattern,
            // line_number_width,
            // by_line: vec![vec![]; line_count],
            // multi_line: vec![],
        };
        // spans.add(fmter.span.clone());
        // if let Some(span) = fmter.aux_span {
        //     spans.add(span.clone());
        // }
        spans
    }

    // /// Add the given span to this sequence, putting it in the right place.
    // fn add(&mut self, span: ast::Span) {
    //     // This is grossly inefficient since we sort after each add, but right
    //     // now, we only ever add two spans at most.
    //     // if span.is_one_line() {
    //     let i = span.start.line - 1; // because lines are 1-indexed
    //     self.by_line[i].push(span);
    //     self.by_line[i].sort();
    //     // } else {
    //     //     self.multi_line.push(span);
    //     //     self.multi_line.sort();
    //     // }
    // }
    //
    // /// Notate the pattern string with carents (`^`) pointing at each span
    // /// location. This only applies to spans that occur within a single line.
    // fn notate(&self) -> String {
    //     let mut notated = String::new();
    //     for (i, line) in self.pattern.lines().enumerate() {
    //         if self.line_number_width > 0 {
    //             notated.push_str(&self.left_pad_line_number(i + 1));
    //             notated.push_str(": ");
    //         } else {
    //             notated.push_str("    ");
    //         }
    //         notated.push_str(line);
    //         notated.push('\n');
    //         if let Some(notes) = self.notate_line(i) {
    //             notated.push_str(&notes);
    //             notated.push('\n');
    //         }
    //     }
    //     notated
    // }
    //
    // /// Return notes for the line indexed at `i` (zero-based). If there are no
    // /// spans for the given line, then `None` is returned. Otherwise, an
    // /// appropriately space padded string with correctly positioned `^` is
    // /// returned, accounting for line numbers.
    // fn notate_line(&self, i: usize) -> Option<String> {
    //     let spans = &self.by_line[i];
    //     if spans.is_empty() {
    //         return None;
    //     }
    //     let mut notes = String::new();
    //     for _ in 0..self.line_number_padding() {
    //         notes.push(' ');
    //     }
    //     let mut pos = 0;
    //     for span in spans {
    //         for _ in pos..(span.start.column - 1) {
    //             notes.push(' ');
    //             pos += 1;
    //         }
    //         let note_len = span.end.column.saturating_sub(span.start.column);
    //         for _ in 0..core::cmp::max(1, note_len) {
    //             notes.push('^');
    //             pos += 1;
    //         }
    //     }
    //     Some(notes)
    // }
}
