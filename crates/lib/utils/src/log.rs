use cfg::settings::LogLevel;
use diagnostics::errors::{
    CliError,
    LogError,
};
use std::fmt::Write;
use unicode_width::UnicodeWidthChar;

use miette::{
    Diagnostic,
    IntoDiagnostic,
    LabeledSpan,
    MietteError,
    ReportHandler,
    Result,
    Severity,
    SourceCode,
    SourceSpan,
    SpanContents,
};
use owo_colors::{
    OwoColorize,
    Style,
};
use std::{
    fmt,
    io::IsTerminal,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub fn init(verbosity: &LogLevel) -> Result<()> {
    // Initialize the logging subsystem
    let subscriber = FmtSubscriber::builder()
        .with_max_level(match verbosity {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
            LogLevel::Fatal => Level::ERROR,
        })
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .into_diagnostic()
        .map_err(|e| LogError::LogSystemInitialization(e.to_string().into()))?;

    // Initialize the error reporting subsystem
    miette::set_hook(Box::new(move |_| Box::new(ByakuganReportHandler::new())))
        .into_diagnostic()
        .map_err(|e| LogError::LogSystemInitialization(e.to_string().into()))?;

    Ok(())
}

// Most of the below code is taken verbatim from the `miette` crate to work in
// the context of `buck2`, which has issues vendoring the `miette` crate
// directly.

#[derive(Debug, Clone)]
pub struct ByakuganReportHandler {
    pub(crate) links:            LinkStyle,
    pub(crate) termwidth:        usize,
    pub(crate) theme:            GraphicalTheme,
    pub(crate) footer:           Option<String>,
    pub(crate) context_lines:    usize,
    pub(crate) tab_width:        usize,
    pub(crate) with_cause_chain: bool,
}

impl ByakuganReportHandler {
    /// Create a new `ByakuganReportHandler` with the default
    /// [`GraphicalTheme`].
    pub fn new() -> Self {
        Self {
            links:            LinkStyle::Link,
            termwidth:        200,
            theme:            GraphicalTheme::default(),
            footer:           None,
            context_lines:    1,
            tab_width:        4,
            with_cause_chain: true,
        }
    }

    ///Create a new `ByakuganReportHandler` with the given [`GraphicalTheme`].
    pub fn new_themed(theme: GraphicalTheme) -> Self {
        Self {
            links: LinkStyle::Link,
            termwidth: 200,
            theme,
            footer: None,
            context_lines: 1,
            tab_width: 4,
            with_cause_chain: true,
        }
    }

    /// Render a [`Diagnostic`]. This function is mostly internal and meant to
    /// be called by the toplevel [`ReportHandler`] handler, but is made public
    /// to make it easier (possible) to test in isolation from global state.
    pub fn render_report(
        &self,
        f: &mut fmt::Formatter<'_>,
        diagnostic: &(dyn Diagnostic),
    ) -> fmt::Result {
        self.render_header(f, diagnostic)?;
        writeln!(f)?;
        self.render_causes(f, diagnostic)?;
        let src = diagnostic.source_code();
        self.render_snippets(f, diagnostic, src)?;
        self.render_footer(f, diagnostic)?;
        self.render_related(f, diagnostic, src)?;
        Ok(())
    }

    fn render_header(&self, f: &mut impl fmt::Write, diagnostic: &(dyn Diagnostic)) -> fmt::Result {
        let severity_style = match diagnostic.severity() {
            Some(Severity::Error) | None => self.theme.styles.error,
            Some(Severity::Warning) => self.theme.styles.warning,
            Some(Severity::Advice) => self.theme.styles.advice,
        };

        let mut header = String::new();

        let error = "Unable to execute filesystem watching task".red();

        write!(f, "{error} ")?;

        let url = "https://docs.rs/byakugan";

        let code =
            if let Some(code) = diagnostic.code() { format!("{code} ") } else { "".to_string() };

        let link = format!(
            "\u{1b}]8;;{}\u{1b}\\{}{}\u{1b}]8;;\u{1b}\\",
            url,
            code.style(severity_style),
            "(link)".style(self.theme.styles.link)
        );
        write!(header, "{link}")?;
        writeln!(f, "{header}")?;
        Ok(())
    }

    fn render_causes(&self, f: &mut impl fmt::Write, diagnostic: &(dyn Diagnostic)) -> fmt::Result {
        let (severity_style, severity_icon) = match diagnostic.severity() {
            Some(Severity::Error) | None => (self.theme.styles.error, &self.theme.characters.error),
            Some(Severity::Warning) => (self.theme.styles.warning, &self.theme.characters.warning),
            Some(Severity::Advice) => (self.theme.styles.advice, &self.theme.characters.advice),
        };

        let initial_indent = format!("  {} ", severity_icon.style(severity_style));
        let rest_indent = format!("  {} ", self.theme.characters.vbar.style(severity_style));
        let width = self.termwidth.saturating_sub(2);
        let opts = textwrap::Options::new(width)
            .initial_indent(&initial_indent)
            .subsequent_indent(&rest_indent);

        writeln!(f, "{}", textwrap::fill(&diagnostic.to_string(), opts))?;

        if !self.with_cause_chain {
            return Ok(());
        }

        if let Some(mut cause_iter) = diagnostic
            .diagnostic_source()
            .map(DiagnosticChain::from_diagnostic)
            .or_else(|| diagnostic.source().map(DiagnosticChain::from_stderror))
            .map(|it| it.peekable())
        {
            while let Some(error) = cause_iter.next() {
                let is_last = cause_iter.peek().is_none();
                let char = if !is_last {
                    self.theme.characters.lcross
                } else {
                    self.theme.characters.lbot
                };
                let initial_indent = format!(
                    "  {}{}{} ",
                    char, self.theme.characters.hbar, self.theme.characters.rarrow
                )
                .style(severity_style)
                .to_string();
                let rest_indent =
                    format!("  {}   ", if is_last { ' ' } else { self.theme.characters.vbar })
                        .style(severity_style)
                        .to_string();
                let opts = textwrap::Options::new(width)
                    .initial_indent(&initial_indent)
                    .subsequent_indent(&rest_indent);
                writeln!(f, "{}", textwrap::fill(&error.to_string(), opts))?;
            }
        }

        Ok(())
    }

    fn render_footer(&self, f: &mut impl fmt::Write, diagnostic: &(dyn Diagnostic)) -> fmt::Result {
        if let Some(help) = diagnostic.help() {
            let width = self.termwidth.saturating_sub(4);
            let initial_indent = "  help: ".style(self.theme.styles.help).to_string();
            let opts = textwrap::Options::new(width)
                .initial_indent(&initial_indent)
                .subsequent_indent("        ");
            writeln!(f, "{}", textwrap::fill(&help.to_string(), opts))?;
        }
        Ok(())
    }

    fn render_related(
        &self,
        f: &mut impl fmt::Write,
        diagnostic: &(dyn Diagnostic),
        parent_src: Option<&dyn SourceCode>,
    ) -> fmt::Result {
        if let Some(related) = diagnostic.related() {
            writeln!(f)?;
            for rel in related {
                match rel.severity() {
                    Some(Severity::Error) | None => write!(f, "Error: ")?,
                    Some(Severity::Warning) => write!(f, "Warning: ")?,
                    Some(Severity::Advice) => write!(f, "Advice: ")?,
                };
                self.render_header(f, rel)?;
                writeln!(f)?;
                self.render_causes(f, rel)?;
                let src = rel.source_code().or(parent_src);
                // self.render_snippets(f, rel, src)?;
                self.render_footer(f, rel)?;
                self.render_related(f, rel, src)?;
            }
        }
        Ok(())
    }

    fn render_snippets(
        &self,
        f: &mut impl fmt::Write,
        diagnostic: &(dyn Diagnostic),
        opt_source: Option<&dyn SourceCode>,
    ) -> fmt::Result {
        if let Some(source) = opt_source {
            if let Some(labels) = diagnostic.labels() {
                let mut labels = labels.collect::<Vec<_>>();
                labels.sort_unstable_by_key(|l| l.inner().offset());
                if !labels.is_empty() {
                    let contents = labels
                        .iter()
                        .map(|label| {
                            source.read_span(label.inner(), self.context_lines, self.context_lines)
                        })
                        .collect::<Result<Vec<Box<dyn SpanContents<'_>>>, MietteError>>()
                        .map_err(|_| fmt::Error)?;
                    let mut contexts = Vec::with_capacity(contents.len());
                    for (right, right_conts) in labels.iter().cloned().zip(contents.iter()) {
                        if contexts.is_empty() {
                            contexts.push((right, right_conts));
                        } else {
                            let (left, left_conts) = contexts.last().unwrap().clone();
                            let left_end = left.offset() + left.len();
                            let right_end = right.offset() + right.len();
                            if left_conts.line() + left_conts.line_count() >= right_conts.line() {
                                // The snippets will overlap, so we create one Big Chunky Boi
                                let new_span = LabeledSpan::new(
                                    left.label().map(String::from),
                                    left.offset(),
                                    if right_end >= left_end {
                                        // Right end goes past left end
                                        right_end - left.offset()
                                    } else {
                                        // right is contained inside left
                                        left.len()
                                    },
                                );
                                if source
                                    .read_span(
                                        new_span.inner(),
                                        self.context_lines,
                                        self.context_lines,
                                    )
                                    .is_ok()
                                {
                                    contexts.pop();
                                    contexts.push((
                                        // We'll throw this away later
                                        new_span, left_conts,
                                    ));
                                } else {
                                    contexts.push((right, right_conts));
                                }
                            } else {
                                contexts.push((right, right_conts));
                            }
                        }
                    }
                    for (ctx, _) in contexts {
                        self.render_context(f, source, &ctx, &labels[..])?;
                    }
                }
            }
        }
        Ok(())
    }

    fn render_context<'a>(
        &self,
        f: &mut impl fmt::Write,
        source: &'a dyn SourceCode,
        context: &LabeledSpan,
        labels: &[LabeledSpan],
    ) -> fmt::Result {
        let (contents, lines) = self.get_lines(source, context.inner())?;

        // sorting is your friend
        let labels = labels
            .iter()
            .zip(self.theme.styles.highlights.iter().cloned().cycle())
            .map(|(label, st)| FancySpan::new(label.label().map(String::from), *label.inner(), st))
            .collect::<Vec<_>>();

        // The max number of gutter-lines that will be active at any given
        // point. We need this to figure out indentation, so we do one loop
        // over the lines to see what the damage is gonna be.
        let mut max_gutter = 0usize;
        for line in &lines {
            let mut num_highlights = 0;
            for hl in &labels {
                if !line.span_line_only(hl) && line.span_applies(hl) {
                    num_highlights += 1;
                }
            }
            max_gutter = std::cmp::max(max_gutter, num_highlights);
        }

        // Oh and one more thing: We need to figure out how much room our line
        // numbers need!
        let linum_width = lines[..]
            .last()
            .map(|line| line.line_number)
            // It's possible for the source to be an empty string.
            .unwrap_or(0)
            .to_string()
            .len();

        // Header
        write!(
            f,
            "{}{}{}",
            " ".repeat(linum_width + 2),
            self.theme.characters.ltop,
            self.theme.characters.hbar,
        )?;

        if let Some(source_name) = contents.name() {
            let source_name = source_name.style(self.theme.styles.link);
            writeln!(f, "[{}:{}:{}]", source_name, contents.line() + 1, contents.column() + 1)?;
        } else if lines.len() <= 1 {
            writeln!(f, "{}", self.theme.characters.hbar.to_string().repeat(3))?;
        } else {
            writeln!(f, "[{}:{}]", contents.line() + 1, contents.column() + 1)?;
        }

        // Now it's time for the fun part--actually rendering everything!
        for line in &lines {
            // Line number, appropriately padded.
            self.write_linum(f, linum_width, line.line_number)?;

            // Then, we need to print the gutter, along with any fly-bys We
            // have separate gutters depending on whether we're on the actual
            // line, or on one of the "highlight lines" below it.
            self.render_line_gutter(f, max_gutter, line, &labels)?;

            // And _now_ we can print out the line text itself!
            self.render_line_text(f, &line.text)?;

            // Next, we write all the highlights that apply to this particular line.
            let (single_line, multi_line): (Vec<_>, Vec<_>) = labels
                .iter()
                .filter(|hl| line.span_applies(hl))
                .partition(|hl| line.span_line_only(hl));
            if !single_line.is_empty() {
                // no line number!
                self.write_no_linum(f, linum_width)?;
                // gutter _again_
                self.render_highlight_gutter(f, max_gutter, line, &labels)?;
                self.render_single_line_highlights(
                    f,
                    line,
                    linum_width,
                    max_gutter,
                    &single_line,
                    &labels,
                )?;
            }
            for hl in multi_line {
                if hl.label().is_some() && line.span_ends(hl) && !line.span_starts(hl) {
                    // no line number!
                    self.write_no_linum(f, linum_width)?;
                    // gutter _again_
                    self.render_highlight_gutter(f, max_gutter, line, &labels)?;
                    self.render_multi_line_end(f, hl)?;
                }
            }
        }
        writeln!(
            f,
            "{}{}{}",
            " ".repeat(linum_width + 2),
            self.theme.characters.lbot,
            self.theme.characters.hbar.to_string().repeat(4),
        )?;
        Ok(())
    }

    fn render_line_gutter(
        &self,
        f: &mut impl fmt::Write,
        max_gutter: usize,
        line: &Line,
        highlights: &[FancySpan],
    ) -> fmt::Result {
        if max_gutter == 0 {
            return Ok(());
        }
        let chars = &self.theme.characters;
        let mut gutter = String::new();
        let applicable = highlights.iter().filter(|hl| line.span_applies(hl));
        let mut arrow = false;
        for (i, hl) in applicable.enumerate() {
            if line.span_starts(hl) {
                gutter.push_str(&chars.ltop.style(hl.style).to_string());
                gutter.push_str(
                    &chars
                        .hbar
                        .to_string()
                        .repeat(max_gutter.saturating_sub(i))
                        .style(hl.style)
                        .to_string(),
                );
                gutter.push_str(&chars.rarrow.style(hl.style).to_string());
                arrow = true;
                break;
            } else if line.span_ends(hl) {
                if hl.label().is_some() {
                    gutter.push_str(&chars.lcross.style(hl.style).to_string());
                } else {
                    gutter.push_str(&chars.lbot.style(hl.style).to_string());
                }
                gutter.push_str(
                    &chars
                        .hbar
                        .to_string()
                        .repeat(max_gutter.saturating_sub(i))
                        .style(hl.style)
                        .to_string(),
                );
                gutter.push_str(&chars.rarrow.style(hl.style).to_string());
                arrow = true;
                break;
            } else if line.span_flyby(hl) {
                gutter.push_str(&chars.vbar.style(hl.style).to_string());
            } else {
                gutter.push(' ');
            }
        }
        write!(
            f,
            "{}{}",
            gutter,
            " ".repeat(
                if arrow { 1 } else { 3 } + max_gutter.saturating_sub(gutter.chars().count())
            )
        )?;
        Ok(())
    }

    fn render_highlight_gutter(
        &self,
        f: &mut impl fmt::Write,
        max_gutter: usize,
        line: &Line,
        highlights: &[FancySpan],
    ) -> fmt::Result {
        if max_gutter == 0 {
            return Ok(());
        }
        let chars = &self.theme.characters;
        let mut gutter = String::new();
        let applicable = highlights.iter().filter(|hl| line.span_applies(hl));
        for (i, hl) in applicable.enumerate() {
            if !line.span_line_only(hl) && line.span_ends(hl) {
                gutter.push_str(&chars.lbot.style(hl.style).to_string());
                gutter.push_str(
                    &chars
                        .hbar
                        .to_string()
                        .repeat(max_gutter.saturating_sub(i) + 2)
                        .style(hl.style)
                        .to_string(),
                );
                break;
            } else {
                gutter.push_str(&chars.vbar.style(hl.style).to_string());
            }
        }
        write!(f, "{:width$}", gutter, width = max_gutter + 1)?;
        Ok(())
    }

    fn write_linum(&self, f: &mut impl fmt::Write, width: usize, linum: usize) -> fmt::Result {
        write!(
            f,
            " {:width$} {} ",
            linum.style(self.theme.styles.linum),
            self.theme.characters.vbar,
            width = width
        )?;
        Ok(())
    }

    fn write_no_linum(&self, f: &mut impl fmt::Write, width: usize) -> fmt::Result {
        write!(f, " {:width$} {} ", "", self.theme.characters.vbar_break, width = width)?;
        Ok(())
    }

    /// Returns an iterator over the visual width of each character in a line.
    fn line_visual_char_width<'a>(&self, text: &'a str) -> impl Iterator<Item = usize> + 'a {
        let mut column = 0;
        let tab_width = self.tab_width;
        text.chars().map(move |c| {
            let width = if c == '\t' {
                // Round up to the next multiple of tab_width
                tab_width - column % tab_width
            } else {
                c.width().unwrap_or(0)
            };
            column += width;
            width
        })
    }

    /// Returns the visual column position of a byte offset on a specific line.
    fn visual_offset(&self, line: &Line, offset: usize) -> usize {
        let line_range = line.offset..=(line.offset + line.length);
        assert!(line_range.contains(&offset));

        let text_index = offset - line.offset;
        let text = &line.text[..text_index.min(line.text.len())];
        let text_width = self.line_visual_char_width(text).sum();
        if text_index > line.text.len() {
            // Spans extending past the end of the line are always rendered as
            // one column past the end of the visible line.
            //
            // This doesn't necessarily correspond to a specific byte-offset,
            // since a span extending past the end of the line could contain:
            //  - an actual \n character (1 byte)
            //  - a CRLF (2 bytes)
            //  - EOF (0 bytes)
            text_width + 1
        } else {
            text_width
        }
    }

    /// Renders a line to the output formatter, replacing tabs with spaces.
    fn render_line_text(&self, f: &mut impl fmt::Write, text: &str) -> fmt::Result {
        for (c, width) in text.chars().zip(self.line_visual_char_width(text)) {
            if c == '\t' {
                for _ in 0..width {
                    f.write_char(' ')?
                }
            } else {
                f.write_char(c)?
            }
        }
        f.write_char('\n')?;
        Ok(())
    }

    fn render_single_line_highlights(
        &self,
        f: &mut impl fmt::Write,
        line: &Line,
        linum_width: usize,
        max_gutter: usize,
        single_liners: &[&FancySpan],
        all_highlights: &[FancySpan],
    ) -> fmt::Result {
        let mut underlines = String::new();
        let mut highest = 0;

        let chars = &self.theme.characters;
        let vbar_offsets: Vec<_> = single_liners
            .iter()
            .map(|hl| {
                let byte_start = hl.offset();
                let byte_end = hl.offset() + hl.len();
                let start = self.visual_offset(line, byte_start).max(highest);
                let end = self.visual_offset(line, byte_end).max(start + 1);

                let vbar_offset = (start + end) / 2;
                let num_left = vbar_offset - start;
                let num_right = end - vbar_offset - 1;
                if start < end {
                    underlines.push_str(
                        &format!(
                            "{:width$}{}{}{}",
                            "",
                            chars.underline.to_string().repeat(num_left),
                            if hl.len() == 0 {
                                chars.uarrow
                            } else if hl.label().is_some() {
                                chars.underbar
                            } else {
                                chars.underline
                            },
                            chars.underline.to_string().repeat(num_right),
                            width = start.saturating_sub(highest),
                        )
                        .style(hl.style)
                        .to_string(),
                    );
                }
                highest = std::cmp::max(highest, end);

                (hl, vbar_offset)
            })
            .collect();
        writeln!(f, "{}", underlines)?;

        for hl in single_liners.iter().rev() {
            if let Some(label) = hl.label() {
                self.write_no_linum(f, linum_width)?;
                self.render_highlight_gutter(f, max_gutter, line, all_highlights)?;
                let mut curr_offset = 1usize;
                for (offset_hl, vbar_offset) in &vbar_offsets {
                    while curr_offset < *vbar_offset + 1 {
                        write!(f, " ")?;
                        curr_offset += 1;
                    }
                    if *offset_hl != hl {
                        write!(f, "{}", chars.vbar.to_string().style(offset_hl.style))?;
                        curr_offset += 1;
                    } else {
                        let lines = format!(
                            "{}{} {}",
                            chars.lbot,
                            chars.hbar.to_string().repeat(2),
                            label,
                        );
                        writeln!(f, "{}", lines.style(hl.style))?;
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn render_multi_line_end(&self, f: &mut impl fmt::Write, hl: &FancySpan) -> fmt::Result {
        writeln!(
            f,
            "{} {}",
            self.theme.characters.hbar.style(hl.style),
            hl.label().unwrap_or_else(|| "".into()),
        )?;
        Ok(())
    }

    fn get_lines<'a>(
        &'a self,
        source: &'a dyn SourceCode,
        context_span: &'a SourceSpan,
    ) -> Result<(Box<dyn SpanContents<'a> + 'a>, Vec<Line>), fmt::Error> {
        let context_data = source
            .read_span(context_span, self.context_lines, self.context_lines)
            .map_err(|_| fmt::Error)?;
        let context = std::str::from_utf8(context_data.data()).expect("Bad utf8 detected");
        let mut line = context_data.line();
        let mut column = context_data.column();
        let mut offset = context_data.span().offset();
        let mut line_offset = offset;
        let mut iter = context.chars().peekable();
        let mut line_str = String::new();
        let mut lines = Vec::new();
        while let Some(char) = iter.next() {
            offset += char.len_utf8();
            let mut at_end_of_file = false;
            match char {
                '\r' => {
                    if iter.next_if_eq(&'\n').is_some() {
                        offset += 1;
                        line += 1;
                        column = 0;
                    } else {
                        line_str.push(char);
                        column += 1;
                    }
                    at_end_of_file = iter.peek().is_none();
                }
                '\n' => {
                    at_end_of_file = iter.peek().is_none();
                    line += 1;
                    column = 0;
                }
                _ => {
                    line_str.push(char);
                    column += 1;
                }
            }

            if iter.peek().is_none() && !at_end_of_file {
                line += 1;
            }

            if column == 0 || iter.peek().is_none() {
                lines.push(Line {
                    line_number: line,
                    offset:      line_offset,
                    length:      offset - line_offset,
                    text:        line_str.clone(),
                });
                line_str.clear();
                line_offset = offset;
            }
        }
        Ok((context_data, lines))
    }

    /// Set the displayed tab width in spaces.
    pub fn tab_width(mut self, width: usize) -> Self {
        self.tab_width = width;
        self
    }

    /// Whether to enable error code linkification using [`Diagnostic::url()`].
    pub fn with_links(mut self, links: bool) -> Self {
        self.links = if links { LinkStyle::Link } else { LinkStyle::Text };
        self
    }

    /// Include the cause chain of the top-level error in the graphical output,
    /// if available.
    pub fn with_cause_chain(mut self) -> Self {
        self.with_cause_chain = true;
        self
    }

    /// Do not include the cause chain of the top-level error in the graphical
    /// output.
    pub fn without_cause_chain(mut self) -> Self {
        self.with_cause_chain = false;
        self
    }

    /// Whether to include [`Diagnostic::url()`] in the output.
    ///
    /// Disabling this is not recommended, but can be useful for more easily
    /// reproducable tests, as `url(docsrs)` links are version-dependent.
    pub fn with_urls(mut self, urls: bool) -> Self {
        self.links = match (self.links, urls) {
            (_, false) => LinkStyle::None,
            (LinkStyle::None, true) => LinkStyle::Link,
            (links, true) => links,
        };
        self
    }

    /// Set a theme for this handler.
    pub fn with_theme(mut self, theme: GraphicalTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the width to wrap the report at.
    pub fn with_width(mut self, width: usize) -> Self {
        self.termwidth = width;
        self
    }

    /// Sets the 'global' footer for this handler.
    pub fn with_footer(mut self, footer: String) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Sets the number of lines of context to show around each error.
    pub fn with_context_lines(mut self, lines: usize) -> Self {
        self.context_lines = lines;
        self
    }
}

impl Default for ByakuganReportHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportHandler for ByakuganReportHandler {
    fn debug(&self, diagnostic: &(dyn Diagnostic), f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            return fmt::Debug::fmt(diagnostic, f);
        }

        self.render_report(f, diagnostic)
    }
}

/**
Theme used by [`GraphicalReportHandler`](crate::GraphicalReportHandler) to
render fancy [`Diagnostic`](crate::Diagnostic) reports.

A theme consists of two things: the set of characters to be used for drawing,
and the
[`owo_colors::Style`](https://docs.rs/owo-colors/latest/owo_colors/struct.Style.html)s to be used to paint various items.

You can create your own custom graphical theme using this type, or you can use
one of the predefined ones using the methods below.
*/
#[derive(Debug, Clone)]
pub struct GraphicalTheme {
    /// Characters to be used for drawing.
    pub characters: ThemeCharacters,
    /// Styles to be used for painting.
    pub styles:     ThemeStyles,
}

impl GraphicalTheme {
    /// ASCII-art-based graphical drawing, with ANSI styling.
    pub fn ascii() -> Self {
        Self { characters: ThemeCharacters::ascii(), styles: ThemeStyles::ansi() }
    }

    /// Graphical theme that draws using both ansi colors and unicode
    /// characters.
    ///
    /// Note that full rgb colors aren't enabled by default because they're
    /// an accessibility hazard, especially in the context of terminal themes
    /// that can change the background color and make hardcoded colors
    /// illegible. Such themes typically remap ansi codes properly, treating
    /// them more like CSS classes than specific colors.
    pub fn unicode() -> Self {
        Self { characters: ThemeCharacters::unicode(), styles: ThemeStyles::ansi() }
    }

    /// Graphical theme that draws in monochrome, while still using unicode
    /// characters.
    pub fn unicode_nocolor() -> Self {
        Self { characters: ThemeCharacters::unicode(), styles: ThemeStyles::none() }
    }

    /// A "basic" graphical theme that skips colors and unicode characters and
    /// just does monochrome ascii art. If you want a completely non-graphical
    /// rendering of your `Diagnostic`s, check out
    /// [crate::NarratableReportHandler], or write your own
    /// [crate::ReportHandler]!
    pub fn none() -> Self {
        Self { characters: ThemeCharacters::ascii(), styles: ThemeStyles::none() }
    }
}

impl Default for GraphicalTheme {
    fn default() -> Self {
        match std::env::var("NO_COLOR") {
            _ if !std::io::stdout().is_terminal() || !std::io::stderr().is_terminal() => {
                Self::ascii()
            }
            Ok(string) if string != "0" => Self::unicode_nocolor(),
            _ => Self::unicode(),
        }
    }
}

/**
Styles for various parts of graphical rendering for the [crate::GraphicalReportHandler].
*/
#[derive(Debug, Clone)]
pub struct ThemeStyles {
    /// Style to apply to things highlighted as "error".
    pub error:      Style,
    /// Style to apply to things highlighted as "warning".
    pub warning:    Style,
    /// Style to apply to things highlighted as "advice".
    pub advice:     Style,
    /// Style to apply to the help text.
    pub help:       Style,
    /// Style to apply to filenames/links/URLs.
    pub link:       Style,
    /// Style to apply to line numbers.
    pub linum:      Style,
    /// Styles to cycle through (using `.iter().cycle()`), to render the lines
    /// and text for diagnostic highlights.
    pub highlights: Vec<Style>,
}

fn style() -> Style {
    Style::new()
}

impl ThemeStyles {
    /// Nice RGB colors.
    /// [Credit](http://terminal.sexy/#FRUV0NDQFRUVrEFCkKlZ9L91ap-1qnWfdbWq0NDQUFBQrEFCkKlZ9L91ap-1qnWfdbWq9fX1).
    pub fn rgb() -> Self {
        Self {
            error:      style().fg_rgb::<255, 30, 30>(),
            warning:    style().fg_rgb::<244, 191, 117>(),
            advice:     style().fg_rgb::<106, 159, 181>(),
            help:       style().fg_rgb::<106, 159, 181>(),
            link:       style().fg_rgb::<92, 157, 255>().underline().bold(),
            linum:      style().dimmed(),
            highlights: vec![
                style().fg_rgb::<246, 87, 248>(),
                style().fg_rgb::<30, 201, 212>(),
                style().fg_rgb::<145, 246, 111>(),
            ],
        }
    }

    /// ANSI color-based styles.
    pub fn ansi() -> Self {
        Self {
            error:      style().red(),
            warning:    style().yellow(),
            advice:     style().cyan(),
            help:       style().cyan(),
            link:       style().cyan().underline().bold(),
            linum:      style().dimmed(),
            highlights: vec![
                style().magenta().bold(),
                style().yellow().bold(),
                style().green().bold(),
            ],
        }
    }

    /// No styling. Just regular ol' monochrome.
    pub fn none() -> Self {
        Self {
            error:      style(),
            warning:    style(),
            advice:     style(),
            help:       style(),
            link:       style(),
            linum:      style(),
            highlights: vec![style()],
        }
    }
}

// ----------------------------------------
// Most of these characters were taken from
// https://github.com/zesterer/ariadne/blob/e3cb394cb56ecda116a0a1caecd385a49e7f6662/src/draw.rs

/// Characters to be used when drawing when using
/// [crate::GraphicalReportHandler].
#[allow(missing_docs)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ThemeCharacters {
    pub hbar:       char,
    pub vbar:       char,
    pub xbar:       char,
    pub vbar_break: char,

    pub uarrow: char,
    pub rarrow: char,

    pub ltop: char,
    pub mtop: char,
    pub rtop: char,
    pub lbot: char,
    pub rbot: char,
    pub mbot: char,

    pub lbox: char,
    pub rbox: char,

    pub lcross: char,
    pub rcross: char,

    pub underbar:  char,
    pub underline: char,

    pub error:   String,
    pub warning: String,
    pub advice:  String,
}

impl ThemeCharacters {
    /// Fancy unicode-based graphical elements.
    pub fn unicode() -> Self {
        Self {
            hbar:       '─',
            vbar:       '│',
            xbar:       '┼',
            vbar_break: '·',
            uarrow:     '▲',
            rarrow:     '▶',
            ltop:       '╭',
            mtop:       '┬',
            rtop:       '╮',
            lbot:       '╰',
            mbot:       '┴',
            rbot:       '╯',
            lbox:       '[',
            rbox:       ']',
            lcross:     '├',
            rcross:     '┤',
            underbar:   '┬',
            underline:  '─',
            error:      "×".into(),
            warning:    "⚠".into(),
            advice:     "☞".into(),
        }
    }

    /// Emoji-heavy unicode characters.
    pub fn emoji() -> Self {
        Self {
            hbar:       '─',
            vbar:       '│',
            xbar:       '┼',
            vbar_break: '·',
            uarrow:     '▲',
            rarrow:     '▶',
            ltop:       '╭',
            mtop:       '┬',
            rtop:       '╮',
            lbot:       '╰',
            mbot:       '┴',
            rbot:       '╯',
            lbox:       '[',
            rbox:       ']',
            lcross:     '├',
            rcross:     '┤',
            underbar:   '┬',
            underline:  '─',
            error:      "💥".into(),
            warning:    "⚠️".into(),
            advice:     "💡".into(),
        }
    }
    /// ASCII-art-based graphical elements. Works well on older terminals.
    pub fn ascii() -> Self {
        Self {
            hbar:       '-',
            vbar:       '|',
            xbar:       '+',
            vbar_break: ':',
            uarrow:     '^',
            rarrow:     '>',
            ltop:       ',',
            mtop:       'v',
            rtop:       '.',
            lbot:       '`',
            mbot:       '^',
            rbot:       '\'',
            lbox:       '[',
            rbox:       ']',
            lcross:     '|',
            rcross:     '|',
            underbar:   '|',
            underline:  '^',
            error:      "x".into(),
            warning:    "!".into(),
            advice:     ">".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LinkStyle {
    None,
    Link,
    Text,
}

/// Iterator of a chain of cause errors.
#[derive(Clone, Default)]
#[allow(missing_debug_implementations)]
pub(crate) struct DiagnosticChain<'a> {
    state: Option<ErrorKind<'a>>,
}

impl<'a> DiagnosticChain<'a> {
    pub(crate) fn from_diagnostic(head: &'a dyn Diagnostic) -> Self {
        DiagnosticChain { state: Some(ErrorKind::Diagnostic(head)) }
    }

    pub(crate) fn from_stderror(head: &'a (dyn std::error::Error + 'static)) -> Self {
        DiagnosticChain { state: Some(ErrorKind::StdError(head)) }
    }
}

impl<'a> Iterator for DiagnosticChain<'a> {
    type Item = ErrorKind<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(err) = self.state.take() {
            self.state = err.get_nested();
            Some(err)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for DiagnosticChain<'_> {
    fn len(&self) -> usize {
        fn depth(d: Option<&ErrorKind<'_>>) -> usize {
            match d {
                Some(d) => 1 + depth(d.get_nested().as_ref()),
                None => 0,
            }
        }

        depth(self.state.as_ref())
    }
}

#[derive(Clone)]
pub(crate) enum ErrorKind<'a> {
    Diagnostic(&'a dyn Diagnostic),
    StdError(&'a (dyn std::error::Error + 'static)),
}

impl<'a> ErrorKind<'a> {
    fn get_nested(&self) -> Option<ErrorKind<'a>> {
        match self {
            ErrorKind::Diagnostic(d) => d
                .diagnostic_source()
                .map(ErrorKind::Diagnostic)
                .or_else(|| d.source().map(ErrorKind::StdError)),
            ErrorKind::StdError(e) => e.source().map(ErrorKind::StdError),
        }
    }
}

impl<'a> std::fmt::Debug for ErrorKind<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Diagnostic(d) => d.fmt(f),
            ErrorKind::StdError(e) => e.fmt(f),
        }
    }
}

impl<'a> std::fmt::Display for ErrorKind<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Diagnostic(d) => d.fmt(f),
            ErrorKind::StdError(e) => e.fmt(f),
        }
    }
}

#[derive(Debug)]
struct Line {
    line_number: usize,
    offset:      usize,
    length:      usize,
    text:        String,
}

impl Line {
    fn span_line_only(&self, span: &FancySpan) -> bool {
        span.offset() >= self.offset && span.offset() + span.len() <= self.offset + self.length
    }

    fn span_applies(&self, span: &FancySpan) -> bool {
        let spanlen = if span.len() == 0 { 1 } else { span.len() };
        // Span starts in this line
        (span.offset() >= self.offset && span.offset() < self.offset + self.length)
        // Span passes through this line
        || (span.offset() < self.offset && span.offset() + spanlen > self.offset + self.length) //todo
        // Span ends on this line
        || (span.offset() + spanlen > self.offset && span.offset() + spanlen <= self.offset + self.length)
    }

    // A 'flyby' is a multi-line span that technically covers this line, but
    // does not begin or end within the line itself. This method is used to
    // calculate gutters.
    fn span_flyby(&self, span: &FancySpan) -> bool {
        // The span itself starts before this line's starting offset (so, in a
        // prev line).
        span.offset() < self.offset
            // ...and it stops after this line's end.
            && span.offset() + span.len() > self.offset + self.length
    }

    // Does this line contain the *beginning* of this multiline span?
    // This assumes self.span_applies() is true already.
    fn span_starts(&self, span: &FancySpan) -> bool {
        span.offset() >= self.offset
    }

    // Does this line contain the *end* of this multiline span?
    // This assumes self.span_applies() is true already.
    fn span_ends(&self, span: &FancySpan) -> bool {
        span.offset() + span.len() >= self.offset &&
            span.offset() + span.len() <= self.offset + self.length
    }
}

#[derive(Debug, Clone)]
struct FancySpan {
    label: Option<String>,
    span:  SourceSpan,
    style: Style,
}

impl PartialEq for FancySpan {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label && self.span == other.span
    }
}

impl FancySpan {
    fn new(label: Option<String>, span: SourceSpan, style: Style) -> Self {
        FancySpan { label, span, style }
    }

    fn style(&self) -> Style {
        self.style
    }

    fn label(&self) -> Option<String> {
        self.label.as_ref().map(|l| l.style(self.style()).to_string())
    }

    fn offset(&self) -> usize {
        self.span.offset()
    }

    fn len(&self) -> usize {
        self.span.len()
    }
}
