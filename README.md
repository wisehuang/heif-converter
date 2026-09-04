# heif-convert

Multi-Platform command line tool written in Rust to convert HEIF images.

This project is inspired by, and is a Rust port of,
[NeverMendel/heif-convert](https://github.com/NeverMendel/heif-convert), the
original Python implementation. The command line interface, the log output and
the conversion behaviour are the same.

## 📝 Table of Contents

- [About](#about)
- [Installation](#installation)
- [Usage](#usage)
- [Arguments](#arguments)
- [Libraries](#libraries)
- [Differences from the Python version](#differences)
- [Supported operating systems](#supported-operating-systems)
- [Credits](#credits)
- [License](#license)

## 📕 About <a name="about"></a>

heif-convert is a multi-platform tool written in Rust to convert High Efficiency
Image File (HEIF) images to jpg, png, webp, gif, tiff, bmp, or ico.

heif-convert is designed to make HEIF batch conversion easy.

## ⚙️ Installation <a name="installation"></a>

### Requirements

heif-convert links against [libheif](https://github.com/strukturag/libheif)
1.17 or newer, which must be available at build and at run time:

```bash
# Debian / Ubuntu
sudo apt install libheif-dev pkg-config

# Alpine
apk add libheif-dev pkgconf

# macOS
brew install libheif pkg-config
```

### Building from source

```bash
git clone https://github.com/NeverMendel/heif-convert.git
cd heif-convert
cargo install --path .
```

`cargo install --path .` builds the release binary and installs it into
`~/.cargo/bin`, which is already on the `PATH` of a standard Rust toolchain
installation. No root privileges are required, and the binary can be removed
again with `cargo uninstall heif-convert`.

### System-wide installation

To make the command available to every user on the machine, build the release
binary and copy it into a system directory:

```bash
cargo build --release
sudo install -m 755 target/release/heif-convert /usr/local/bin/
```

`/usr/local/bin` is the correct destination for manually installed binaries.
Avoid installing into `/usr/bin`: that directory is owned by the system package
manager, and files placed there by hand can be overwritten or conflict with
distribution packages.

Note that the binary is dynamically linked against `libheif`, so the runtime
library (`libheif1` on Debian and Ubuntu) must also be installed on any machine
the binary is copied to. Uninstall with:

```bash
sudo rm /usr/local/bin/heif-convert
```

## Usage

heif-convert can be used from the command line by invoking the `heif-convert`
command.

Convert an HEIF image to a JPG image:

```bash
heif-convert input.heic
```

Convert all HEIF images in the current folder to JPG images:

```bash
heif-convert *.heic
```

## Arguments

```
Command line tool to convert HEIF images

Usage: heif-convert [OPTIONS] <INPUT>...

Arguments:
  <INPUT>...  HEIF input file(s)

Options:
  -o, --output <OUTPUT>    output file name
                           defaults to original file name (default: '{name}')
  -p, --path <PATH>        output file path
                           defaults to original file path (default: '{path}')
  -f, --format <FORMAT>    output format (default: jpg)
                           [possible values: jpg, png, webp, gif, tiff, bmp, ico]
  -q, --quality <QUALITY>  output quality, integer [0, 100] (default: 90)
  -n, --no-exif            Do not include EXIF metadata in the converted image
  -v, --verbose            enable verbose logging (-vv enables extra verbose logging)
      --extra-verbose      enable extra verbose logging
  -h, --help               Print help
  -V, --version            Print version
```

## Libraries

heif-convert uses the following libraries:

- [libheif-rs](https://github.com/Cykooz/libheif-rs) - HEIF decoding
- [image](https://github.com/image-rs/image) - image encoding
- [img-parts](https://github.com/paolobarbolini/img-parts) - EXIF embedding
- [ico](https://github.com/mdsteele/rust-ico) - multi resolution ICO encoding
- [clap](https://github.com/clap-rs/clap) - argument parsing

## Differences from the Python version <a name="differences"></a>

The behaviour of the tool is the same, but the underlying encoders are not the
same as the ones used by Pillow, so the produced files are not byte identical:

- decoded pixels differ from pillow-heif by at most 1 per channel, due to a
  different YUV to RGB rounding;
- the JPEG, PNG, WebP, GIF, TIFF and BMP encoders are the Rust ones from the
  `image` crate;
- EXIF metadata is copied to JPG, PNG and WebP outputs. GIF, BMP and ICO do not
  carry EXIF, same as Pillow; TIFF outputs are written without EXIF;
- WebP output is lossless, so `--quality` only affects JPG.

## Supported operating systems

heif-convert works on Linux, Mac OS and Windows systems, wherever libheif is
available.

## Credits <a name="credits"></a>

Inspired by [NeverMendel/heif-convert](https://github.com/NeverMendel/heif-convert)
by Davide Cazzin, the original Python implementation this port is based on.

## License

[MIT License](LICENSE)

This project is derived from
[NeverMendel/heif-convert](https://github.com/NeverMendel/heif-convert), which
is also MIT licensed. The original copyright notice is retained in
[LICENSE](LICENSE).
