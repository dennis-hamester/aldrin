use crate::test::{RunError, Test};
use anyhow::{anyhow, Error, Result};
use once_cell::sync::Lazy;
use std::cmp;
use std::str::FromStr;
use termcolor::{Color as TermColor, ColorSpec, StandardStream, WriteColor};

fn style(fg: Option<TermColor>, bold: bool) -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(fg);
    spec.set_bold(bold);
    spec
}

fn get_termwidth() -> usize {
    const MIN_TERMWIDTH: usize = 20;
    let termwidth = textwrap::termwidth();
    cmp::max(termwidth, MIN_TERMWIDTH)
}

static STYLE_REGULAR: Lazy<ColorSpec> = Lazy::new(|| style(None, false));
static STYLE_TEST_NAME: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Blue), false));
static STYLE_MESSAGE_TYPE: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Yellow), false));
static STYLE_PASSED: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Green), true));
static STYLE_FAILED: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Red), true));
static STYLE_FAILED_DETAIL: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Red), false));

#[derive(Copy, Clone)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl FromStr for ColorChoice {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "auto" => Ok(ColorChoice::Auto),
            "always" => Ok(ColorChoice::Always),
            "never" => Ok(ColorChoice::Never),
            _ => Err(anyhow!("expected one of auto, always or never")),
        }
    }
}

pub fn make_output(color_choice: ColorChoice) -> Result<impl WriteColor> {
    let color_choice = match color_choice {
        ColorChoice::Auto if atty::is(atty::Stream::Stdout) => termcolor::ColorChoice::Auto,
        ColorChoice::Auto | ColorChoice::Never => termcolor::ColorChoice::Never,
        ColorChoice::Always => termcolor::ColorChoice::Always,
    };

    let mut stream = StandardStream::stdout(color_choice);
    stream.reset()?;
    Ok(stream)
}

pub fn list_tests<O, I>(mut output: O, tests: I) -> Result<()>
where
    O: WriteColor,
    I: IntoIterator,
    I::Item: Test,
{
    for test in tests {
        output.set_color(&STYLE_TEST_NAME)?;
        write!(output, "{}", test.name())?;
        output.set_color(&STYLE_REGULAR)?;
        write!(output, ": {}", test.short())?;

        write!(output, " [")?;

        let mut first = true;
        for message_type in test.message_types() {
            if first {
                first = false;
            } else {
                write!(output, ", ")?;
            }

            output.set_color(&STYLE_MESSAGE_TYPE)?;
            write!(output, "{}", message_type)?;
            output.set_color(&STYLE_REGULAR)?;
        }

        writeln!(output, "]")?;
    }

    Ok(())
}

pub fn describe_test(mut output: impl WriteColor, test: impl Test) -> Result<()> {
    output.set_color(&STYLE_TEST_NAME)?;
    write!(output, "{}", test.name())?;
    output.set_color(&STYLE_REGULAR)?;
    writeln!(output, ": {}", test.short())?;

    writeln!(output)?;
    writeln!(output, "Primarily tested message(s):")?;
    for message_type in test.message_types() {
        write!(output, "  - ")?;
        output.set_color(&STYLE_MESSAGE_TYPE)?;
        writeln!(output, "{}", message_type)?;
        output.set_color(&STYLE_REGULAR)?;
    }

    writeln!(output)?;
    let desc = test.long().unwrap_or("No description available.");
    let termwidth = get_termwidth();

    for line in textwrap::wrap_iter(desc, termwidth - 4) {
        writeln!(output, "  {}", line)?;
    }

    Ok(())
}

pub fn prepare_report(mut output: impl WriteColor, name: &str) -> Result<()> {
    output.set_color(&STYLE_TEST_NAME)?;
    write!(output, "{}", name)?;

    output.set_color(&STYLE_REGULAR)?;
    write!(output, " ... ")?;

    output.flush()?;
    Ok(())
}

pub fn finish_report(mut output: impl WriteColor, res: Result<(), RunError>) -> Result<()> {
    let err = match res {
        Ok(()) => {
            output.set_color(&STYLE_PASSED)?;
            writeln!(output, "passed")?;
            output.set_color(&STYLE_REGULAR)?;
            return Ok(());
        }

        Err(e) => e,
    };

    output.set_color(&STYLE_FAILED)?;
    writeln!(output, "failed")?;

    output.set_color(&STYLE_FAILED_DETAIL)?;
    write!(output, "Error")?;
    output.set_color(&STYLE_REGULAR)?;
    writeln!(output, ": {:?}", err.error)?;
    writeln!(output)?;

    if !err.stderr.is_empty() {
        output.set_color(&STYLE_FAILED_DETAIL)?;
        write!(output, "Child's stderr")?;
        output.set_color(&STYLE_REGULAR)?;
        writeln!(output, ":")?;
        writeln!(output, "{}", String::from_utf8_lossy(&err.stderr))?;
        writeln!(output)?;
    }

    Ok(())
}
