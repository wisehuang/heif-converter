mod cli;
mod convert;
mod logging;

use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use libheif_rs::LibHeif;
use log::{debug, info};

use cli::{Args, VERSION};

fn main() -> ExitCode {
    let mut args = Args::parse();

    let input = match cli::expand_input(&args.input) {
        Ok(input) => input,
        Err(message) => {
            eprintln!("usage: heif-convert [-h] [-o OUTPUT] [-p PATH]");
            eprintln!(
                "                    [-f {{jpg,png,webp,gif,tiff,bmp,ico}}] [-q QUALITY] [-n] [-v]"
            );
            eprintln!("                    [-vv] [-V]");
            eprintln!("                    input [input ...]");
            eprintln!("heif-convert: error: {message}");
            return ExitCode::from(2);
        }
    };
    args.input = input;

    logging::configure(args.verbose(), args.extra_verbose());

    debug!(
        "heif-convert {VERSION} run with arguments: {}",
        args.namespace()
    );

    debug!("Registering HEIF opener");
    let lib_heif = LibHeif::new();

    for input_file in &args.input {
        if let Err(error) = convert_file(&lib_heif, &args, input_file) {
            eprintln!("heif-convert: error: {error:#}");
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}

fn convert_file(lib_heif: &LibHeif, args: &Args, input_file: &str) -> anyhow::Result<()> {
    let input_path = Path::new(input_file);
    let absolute_input = absolute_path(input_path);
    info!("Reading {}", absolute_input.display());

    let image = convert::read_heif(lib_heif, input_path)?;

    let name = input_path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let output_filename = format!(
        "{}.{}",
        args.output.replace("{name}", &name),
        args.format.extension()
    );
    let parent = absolute_input
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let output_filepath =
        Path::new(&args.path.replace("{path}", &parent.to_string_lossy())).join(&output_filename);

    info!("Writing {}", output_filepath.display());
    convert::write_image(
        &image,
        &output_filepath,
        args.format,
        args.quality,
        args.exif,
    )?;
    println!("Wrote {}", output_filepath.display());
    Ok(())
}

/// Lexical equivalent of Python's `os.path.abspath`: makes the path absolute
/// and normalizes it without resolving symlinks.
fn absolute_path(path: &Path) -> PathBuf {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };

    let mut normalized = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push("..");
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}
