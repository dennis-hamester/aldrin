use crate::error::{Error, SubprocessError};
use crate::Options;
use aldrin_parser::{Parsed, Schema};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RustOptions<'a> {
    pub rustfmt: bool,
    pub rustfmt_toml: Option<&'a Path>,
}

impl<'a> RustOptions<'a> {
    pub fn new() -> Self {
        RustOptions {
            rustfmt: false,
            rustfmt_toml: None,
        }
    }
}

impl<'a> Default for RustOptions<'a> {
    fn default() -> Self {
        RustOptions::new()
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct RustOutput {
    pub module_name: String,
    pub module_content: String,
}

pub(crate) fn generate(
    parsed: &Parsed,
    options: &Options,
    rust_options: &RustOptions,
) -> Result<RustOutput, Error> {
    let schema = parsed.main_schema();

    let gen = RustGenerator {
        schema,
        options,
        rust_options,
        output: RustOutput {
            module_name: schema.name().to_owned(),
            module_content: String::new(),
        },
    };

    gen.generate()
}

struct RustGenerator<'a> {
    schema: &'a Schema,
    options: &'a Options,
    rust_options: &'a RustOptions<'a>,
    output: RustOutput,
}

impl<'a> RustGenerator<'a> {
    fn generate(mut self) -> Result<RustOutput, Error> {
        if self.rust_options.rustfmt {
            self.format()?;
        }

        Ok(self.output)
    }

    fn format(&mut self) -> Result<(), Error> {
        let mut cmd = Command::new("rustfmt");
        cmd.arg("--edition").arg("2018");
        if let Some(rustfmt_toml) = self.rust_options.rustfmt_toml {
            cmd.arg("--config-path").arg(rustfmt_toml);
        }

        let mut rustfmt = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        rustfmt
            .stdin
            .as_mut()
            .unwrap()
            .write_all(self.output.module_content.as_bytes())?;

        let rustfmt = rustfmt.wait_with_output()?;
        if rustfmt.status.success() {
            self.output.module_content =
                String::from_utf8(rustfmt.stdout).expect("got invalid UTF-8 from rustfmt");
            Ok(())
        } else {
            Err(SubprocessError {
                command: "rustfmt".to_owned(),
                code: rustfmt.status.code(),
                stderr: String::from_utf8(rustfmt.stderr).ok(),
            }
            .into())
        }
    }
}
