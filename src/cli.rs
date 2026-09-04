use std::fmt;
use std::path::Path;

use clap::{ArgAction, Parser, ValueEnum};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Format {
    Jpg,
    Png,
    Webp,
    Gif,
    Tiff,
    Bmp,
    Ico,
}

impl Format {
    pub fn extension(self) -> &'static str {
        match self {
            Format::Jpg => "jpg",
            Format::Png => "png",
            Format::Webp => "webp",
            Format::Gif => "gif",
            Format::Tiff => "tiff",
            Format::Bmp => "bmp",
            Format::Ico => "ico",
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.extension())
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "heif-convert",
    version = VERSION,
    about = "Command line tool to convert HEIF images",
    disable_help_flag = false
)]
pub struct Args {
    /// HEIF input file(s)
    #[arg(required = true)]
    pub input: Vec<String>,

    /// output file name
    /// defaults to original file name (default: '{name}')
    #[arg(short, long, default_value = "{name}", verbatim_doc_comment)]
    pub output: String,

    /// output file path
    /// defaults to original file path (default: '{path}')
    #[arg(short, long, default_value = "{path}", verbatim_doc_comment)]
    pub path: String,

    /// output format (default: jpg)
    #[arg(short, long, value_enum, default_value_t = Format::Jpg)]
    pub format: Format,

    /// output quality, integer [0, 100] (default: 90)
    #[arg(short, long, default_value_t = 90)]
    pub quality: u8,

    /// Do not include EXIF metadata in the converted image
    #[arg(short = 'n', long = "no-exif", action = ArgAction::SetFalse)]
    pub exif: bool,

    /// enable verbose logging
    ///
    /// `-vv` is accepted as well and enables extra verbose logging
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    verbosity: u8,

    /// enable extra verbose logging
    #[arg(long = "extra-verbose")]
    extra_verbose_flag: bool,
}

impl Args {
    /// True when at least `-v` was given, matching argparse's `verbose` flag.
    pub fn verbose(&self) -> bool {
        self.verbosity == 1
    }

    /// True when `-vv` or `--extra-verbose` was given.
    pub fn extra_verbose(&self) -> bool {
        self.extra_verbose_flag || self.verbosity >= 2
    }

    /// Renders the parsed arguments the way Python's `argparse.Namespace` does,
    /// so the debug log line stays identical to the original implementation.
    pub fn namespace(&self) -> String {
        let inputs = self
            .input
            .iter()
            .map(|i| format!("'{i}'"))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "Namespace(input=[{}], output='{}', path='{}', format='{}', quality={}, exif={}, verbose={}, extra_verbose={})",
            inputs,
            self.output,
            self.path,
            self.format,
            self.quality,
            py_bool(self.exif),
            py_bool(self.verbose()),
            py_bool(self.extra_verbose()),
        )
    }
}

fn py_bool(value: bool) -> &'static str {
    if value {
        "True"
    } else {
        "False"
    }
}

/// Expands the wildcards found in the input arguments and checks that every
/// non-wildcard input exists. Mirrors `parse_args` of the Python version:
/// errors are reported through `parser.error`, which exits with status 2.
pub fn expand_input(input: &[String]) -> Result<Vec<String>, String> {
    let mut expanded_input_files = Vec::new();
    for input_file in input {
        if input_file.contains('*') {
            let matches: Vec<String> = glob::glob(input_file)
                .map_err(|err| format!("Invalid pattern '{input_file}': {err}"))?
                .filter_map(Result::ok)
                .map(|path| path.to_string_lossy().into_owned())
                .collect();
            if matches.is_empty() {
                return Err(format!("No matches found: {input_file}"));
            }
            expanded_input_files.extend(matches);
        } else {
            if !Path::new(input_file).is_file() {
                return Err(format!("Input file '{input_file}' does not exist"));
            }
            expanded_input_files.push(input_file.clone());
        }
    }
    Ok(expanded_input_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Args::command().debug_assert();
    }

    #[test]
    fn defaults_match_the_python_implementation() {
        let args = Args::parse_from(["heif-convert", "image.heic"]);
        assert_eq!(args.output, "{name}");
        assert_eq!(args.path, "{path}");
        assert_eq!(args.format, Format::Jpg);
        assert_eq!(args.quality, 90);
        assert!(args.exif);
        assert!(!args.verbose());
        assert!(!args.extra_verbose());
    }

    #[test]
    fn verbosity_flags() {
        let verbose = Args::parse_from(["heif-convert", "-v", "image.heic"]);
        assert!(verbose.verbose());
        assert!(!verbose.extra_verbose());

        let extra = Args::parse_from(["heif-convert", "-vv", "image.heic"]);
        assert!(!extra.verbose());
        assert!(extra.extra_verbose());

        let long = Args::parse_from(["heif-convert", "--extra-verbose", "image.heic"]);
        assert!(long.extra_verbose());
    }

    #[test]
    fn no_exif_flag() {
        let args = Args::parse_from(["heif-convert", "-n", "image.heic"]);
        assert!(!args.exif);
    }

    #[test]
    fn namespace_rendering() {
        let args = Args::parse_from(["heif-convert", "-vv", "-n", "a.heic"]);
        assert_eq!(
            args.namespace(),
            "Namespace(input=['a.heic'], output='{name}', path='{path}', format='jpg', quality=90, exif=False, verbose=False, extra_verbose=True)"
        );
    }

    #[test]
    fn missing_input_file_is_rejected() {
        let error = expand_input(&["does-not-exist.heic".to_owned()]).unwrap_err();
        assert_eq!(error, "Input file 'does-not-exist.heic' does not exist");
    }

    #[test]
    fn unmatched_wildcard_is_rejected() {
        let error = expand_input(&["*.does-not-exist".to_owned()]).unwrap_err();
        assert_eq!(error, "No matches found: *.does-not-exist");
    }
}
