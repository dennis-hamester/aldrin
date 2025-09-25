use crate::diag;
use aldrin_parser::{FilesystemResolver, Formatter, MemoryResolver, Parser};
use anstream::{eprint, eprintln};
use anstyle::{AnsiColor, Color, Style};
use anyhow::{anyhow, Context, Result};
use diffy::{Hunk, Line};
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};

#[derive(clap::Parser)]
#[clap(arg_required_else_help = true)]
pub(crate) struct FmtArgs {
    /// Check if the specified schemas are formatted.
    ///
    /// If a schema is not formatted correctly, then this command will print a diff and exit with an
    /// error.
    #[clap(long)]
    check: bool,

    /// Paths to one or more Aldrin schema files.
    ///
    /// Alternatively, a single `-` can be specified to read the schema from stdin.
    #[clap(required = true)]
    schemas: Vec<PathBuf>,
}

pub(crate) fn run(args: FmtArgs) -> Result<bool> {
    let is_stdin =
        (args.schemas.len() == 1) && (args.schemas.first().map(|p| &**p) == Some(Path::new("-")));

    let parsers = if is_stdin {
        let schema = io::read_to_string(io::stdin());
        let parser = Parser::parse(MemoryResolver::new("stdin", schema));
        vec![parser]
    } else {
        args.schemas
            .into_iter()
            .map(|path| Parser::parse(FilesystemResolver::new(path)))
            .collect()
    };

    let mut res = true;
    let mut first = true;

    for parser in &parsers {
        let formatter = match Formatter::new(parser) {
            Ok(formatter) => formatter,

            Err(errs) => {
                res = false;
                print_error_header(parser, &parsers, &mut first);
                diag::print_errors(parser, errs);
                continue;
            }
        };

        if args.check {
            res &= check(formatter, parser, &parsers, &mut first)?;
        } else {
            fmt(formatter, parser, is_stdin)?;
        }
    }

    Ok(res)
}

fn fmt(formatter: Formatter, parser: &Parser, is_stdin: bool) -> Result<()> {
    if is_stdin {
        formatter.to_writer(io::stdout())?;
    } else {
        let path = parser.main_schema().path();

        let file = File::options()
            .write(true)
            .truncate(true)
            .open(path)
            .with_context(|| anyhow!("failed to open `{path}`"))?;

        formatter.to_writer(BufWriter::new(file))?;
    }

    Ok(())
}

fn check(
    formatter: Formatter,
    parser: &Parser,
    parsers: &[Parser],
    first: &mut bool,
) -> Result<bool> {
    let original = parser.main_schema().source().unwrap();
    let formatted = formatter.to_string();

    if original == formatted {
        Ok(true)
    } else {
        print_error_header(parser, parsers, first);
        let patch = diffy::create_patch(original, &formatted);

        for hunk in patch.hunks() {
            print_hunk(hunk);
        }

        Ok(false)
    }
}

fn print_hunk(hunk: &Hunk<str>) {
    eprintln!("@@ -{} +{} @@", hunk.old_range(), hunk.new_range());

    for line in hunk.lines() {
        print_hunk_line(line);
    }
}

fn print_hunk_line(line: &Line<str>) {
    const CONTEXT: Style = Style::new();
    const DELETE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
    const INSERT: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));

    let (sign, line, style) = match line {
        Line::Context(line) => (' ', line, CONTEXT),
        Line::Delete(line) => ('-', line, DELETE),
        Line::Insert(line) => ('+', line, INSERT),
    };

    eprint!("{style}");
    eprint!("{sign}{line}");

    if !line.ends_with('\n') {
        eprintln!();
        eprintln!("\\ No newline at end of file");
    }

    eprint!("{style:#}");
}

fn print_error_header(parser: &Parser, parsers: &[Parser], first: &mut bool) {
    if parsers.len() > 1 {
        if *first {
            *first = false;
        } else {
            eprintln!();
        }

        eprintln!("{}:", parser.main_schema().path());
    }
}
