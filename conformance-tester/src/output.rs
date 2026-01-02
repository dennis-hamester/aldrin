use crate::FilterArgs;
use crate::run_error::RunError;
use crate::test::Test;
use anstream::{print, println};
use anstyle::{AnsiColor, Color, Style};
use anyhow::Result;
use std::io::Write;
use std::time::Duration;
use textwrap::Options;

const fn style(fg: Option<AnsiColor>, bold: bool) -> Style {
    let mut style = Style::new();

    if let Some(fg) = fg {
        style = style.fg_color(Some(Color::Ansi(fg)));
    }

    if bold {
        style = style.bold();
    }

    style
}

const STYLE_TEST_NAME: Style = style(Some(AnsiColor::Blue), false);
const STYLE_MESSAGE_TYPE: Style = style(Some(AnsiColor::Yellow), false);
const STYLE_PASSED: Style = style(Some(AnsiColor::Green), true);
const STYLE_FAILED: Style = style(Some(AnsiColor::Red), true);
const STYLE_FAILED_DETAIL: Style = style(Some(AnsiColor::Red), false);
const STYLE_SEPARATOR: Style = style(Some(AnsiColor::Cyan), true);

fn get_termwidth() -> usize {
    const MIN_TERMWIDTH: usize = 20;
    const DEFAULT_TERMWIDTH: usize = 80;

    terminal_size::terminal_size()
        .map_or(DEFAULT_TERMWIDTH, |(size, _)| size.0 as usize)
        .max(MIN_TERMWIDTH)
}

pub(crate) fn list_tests(args: &FilterArgs, tests: &[Test]) {
    for test in tests.iter().filter(|test| args.matches(test)) {
        print!("{style}{}{style:#}", test.name, style = STYLE_TEST_NAME);

        if let Some(ref description) = test.description {
            print!(": {description}");
        }

        if !test.message_types.is_empty() {
            print!(" [");

            let mut first = true;
            for message_type in &test.message_types {
                if first {
                    first = false;
                } else {
                    print!(", ");
                }

                print!("{style}{message_type}{style:#}", style = STYLE_MESSAGE_TYPE);
            }

            print!("]");
        }

        println!();
    }
}

pub(crate) fn describe_test(test: &Test) {
    print!("{style}{}{style:#}", test.name, style = STYLE_TEST_NAME);

    if let Some(ref description) = test.description {
        println!(": {}", description);
    }

    if let Some(long_description) = test.long_description.as_deref() {
        println!();
        println!("Description:");

        let options = Options::new(get_termwidth())
            .initial_indent("  ")
            .subsequent_indent("  ");

        for line in textwrap::wrap(long_description, options) {
            println!("{line}");
        }
    }

    println!();
    println!("Minimum protocol version: {}", test.version);

    if !test.message_types.is_empty() {
        println!();
        println!("Primarily tested message(s):");

        for message_type in &test.message_types {
            println!(
                "  - {style}{message_type}{style:#}",
                style = STYLE_MESSAGE_TYPE,
            );
        }
    }
}

pub(crate) fn prepare_report(name: &str) {
    print!("{style}{name}{style:#} ... ", style = STYLE_TEST_NAME);
    let _ = anstream::stdout().flush();
}

fn print_seperator() {
    print!("{style}|{style:#} ", style = STYLE_SEPARATOR);
}

pub(crate) fn finish_report(res: Result<Duration, RunError>) {
    let err = match res {
        Ok(dur) => {
            println!(
                "{style}passed{style:#} ({}ms)",
                dur.as_millis(),
                style = STYLE_PASSED,
            );

            return;
        }

        Err(e) => e,
    };

    println!("{style}failed{style:#}", style = STYLE_FAILED);

    print_seperator();
    print!("{style}Error{style:#}: ", style = STYLE_FAILED_DETAIL);

    let error = format!("{:?}", err.error);
    for (i, line) in error.lines().enumerate() {
        if i > 0 {
            print_seperator();
        }

        println!("{line}");
    }

    if !err.stderr.is_empty() {
        print_seperator();
        println!();

        print_seperator();
        println!("Child's stderr:");

        let stderr = String::from_utf8_lossy(&err.stderr);
        for line in stderr.lines() {
            print_seperator();
            println!("    {line}");
        }
    }

    println!();
}

pub(crate) fn summary(passed: usize, total: usize) {
    println!();
    println!("{style}Summary{style:#}:", style = STYLE_TEST_NAME);

    print_seperator();
    if passed == total {
        print!("{style}passed{style:#}", style = STYLE_PASSED);
    } else {
        print!("passed");
    }
    println!(": {passed} test(s)");

    print_seperator();
    if total == passed {
        print!("failed");
    } else {
        print!("{style}failed{style:#}", style = STYLE_FAILED);
    }
    println!(": {} test(s)", total - passed);

    print_seperator();
    println!("total:  {} test(s)", total);
}
