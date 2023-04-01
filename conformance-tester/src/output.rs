use crate::run_error::RunError;
use crate::test::Test;
use crate::FilterArgs;
use anyhow::Result;
use clap::ColorChoice;
use is_terminal::IsTerminal;
use once_cell::sync::Lazy;
use std::cmp;
use std::io;
use std::time::Duration;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

fn style(fg: Option<Color>, bold: bool) -> ColorSpec {
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
static STYLE_TEST_NAME: Lazy<ColorSpec> = Lazy::new(|| style(Some(Color::Blue), false));
static STYLE_MESSAGE_TYPE: Lazy<ColorSpec> = Lazy::new(|| style(Some(Color::Yellow), false));
static STYLE_PASSED: Lazy<ColorSpec> = Lazy::new(|| style(Some(Color::Green), true));
static STYLE_FAILED: Lazy<ColorSpec> = Lazy::new(|| style(Some(Color::Red), true));
static STYLE_FAILED_DETAIL: Lazy<ColorSpec> = Lazy::new(|| style(Some(Color::Red), false));
static STYLE_SEPARATOR: Lazy<ColorSpec> = Lazy::new(|| style(Some(Color::Cyan), true));

pub fn make_output(color_choice: ColorChoice) -> Result<impl WriteColor> {
    let color_choice = match color_choice {
        ColorChoice::Auto if io::stdout().is_terminal() => termcolor::ColorChoice::Auto,
        ColorChoice::Auto | ColorChoice::Never => termcolor::ColorChoice::Never,
        ColorChoice::Always => termcolor::ColorChoice::Always,
    };

    let mut stream = StandardStream::stdout(color_choice);
    stream.reset()?;
    Ok(stream)
}

pub fn list_tests<O>(args: FilterArgs, mut output: O, tests: &[Test]) -> Result<()>
where
    O: WriteColor,
{
    for test in tests.iter().filter(|test| args.matches(test)) {
        output.set_color(&STYLE_TEST_NAME)?;
        write!(output, "{}", test.name)?;
        output.set_color(&STYLE_REGULAR)?;

        if let Some(ref description) = test.description {
            write!(output, ": {description}")?;
        }

        if !test.message_types.is_empty() {
            write!(output, " [")?;

            let mut first = true;
            for message_type in &test.message_types {
                if first {
                    first = false;
                } else {
                    write!(output, ", ")?;
                }

                output.set_color(&STYLE_MESSAGE_TYPE)?;
                write!(output, "{message_type}")?;
                output.set_color(&STYLE_REGULAR)?;
            }

            write!(output, "]")?;
        }

        writeln!(output)?;
    }

    Ok(())
}

pub fn describe_test(mut output: impl WriteColor, test: &Test) -> Result<()> {
    output.set_color(&STYLE_TEST_NAME)?;
    write!(output, "{}", test.name)?;
    output.set_color(&STYLE_REGULAR)?;

    if let Some(ref description) = test.description {
        writeln!(output, ": {}", description)?;
    }

    if let Some(long_description) = test.long_description.as_deref() {
        writeln!(output)?;
        writeln!(output, "Description:")?;

        let termwidth = get_termwidth();
        for line in textwrap::wrap(long_description, termwidth - 4) {
            writeln!(output, "  {line}")?;
        }
    }

    if !test.message_types.is_empty() {
        writeln!(output)?;
        writeln!(output, "Primarily tested message(s):")?;
        for message_type in &test.message_types {
            write!(output, "  - ")?;
            output.set_color(&STYLE_MESSAGE_TYPE)?;
            writeln!(output, "{message_type}")?;
            output.set_color(&STYLE_REGULAR)?;
        }
    }

    Ok(())
}

pub fn prepare_report(mut output: impl WriteColor, name: &str) -> Result<()> {
    output.set_color(&STYLE_TEST_NAME)?;
    write!(output, "{name}")?;

    output.set_color(&STYLE_REGULAR)?;
    write!(output, " ... ")?;

    output.flush()?;
    Ok(())
}

fn print_seperator(mut output: impl WriteColor) -> Result<()> {
    output.set_color(&STYLE_SEPARATOR)?;
    write!(output, "|")?;
    output.set_color(&STYLE_REGULAR)?;
    write!(output, " ")?;
    Ok(())
}

pub fn finish_report(mut output: impl WriteColor, res: Result<Duration, RunError>) -> Result<()> {
    let err = match res {
        Ok(dur) => {
            output.set_color(&STYLE_PASSED)?;
            write!(output, "passed")?;
            output.set_color(&STYLE_REGULAR)?;
            writeln!(output, " ({}ms)", dur.as_millis())?;
            return Ok(());
        }

        Err(e) => e,
    };

    output.set_color(&STYLE_FAILED)?;
    writeln!(output, "failed")?;

    print_seperator(&mut output)?;
    output.set_color(&STYLE_FAILED_DETAIL)?;
    write!(output, "Error")?;
    output.set_color(&STYLE_REGULAR)?;
    write!(output, ": ")?;

    let error = format!("{:?}", err.error);
    for (i, line) in error.lines().enumerate() {
        if i > 0 {
            print_seperator(&mut output)?;
        }

        writeln!(output, "{line}")?;
    }

    if !err.stderr.is_empty() {
        print_seperator(&mut output)?;
        writeln!(output)?;

        print_seperator(&mut output)?;
        write!(output, "Child's stderr")?;
        writeln!(output, ":")?;

        let stderr = String::from_utf8_lossy(&err.stderr);
        for line in stderr.lines() {
            print_seperator(&mut output)?;
            writeln!(output, "    {line}")?;
        }
    }

    writeln!(output)?;
    Ok(())
}
