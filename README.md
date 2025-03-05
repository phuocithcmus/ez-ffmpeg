<p align="center">
  <img src="https://raw.githubusercontent.com/YeautyYE/ez-ffmpeg/main/logo.jpg" alt="Logo" width="300">
</p>

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/ez-ffmpeg.svg)](https://crates.io/crates/ez-ffmpeg)
[![Documentation](https://img.shields.io/badge/docs.rs-ez--ffmpeg-blue)](https://docs.rs/ez-ffmpeg)
[![License: WTFPL](https://img.shields.io/badge/License-WTFPL-brightgreen.svg)](https://github.com/YeautyYE/ez-ffmpeg/blob/master/LICENSE)
[![Rust](https://img.shields.io/badge/Rust-%3E=1.80.0-orange)](https://www.rust-lang.org/)
[![FFmpeg](https://img.shields.io/badge/FFmpeg-%3E=7.0-blue)](https://ffmpeg.org)

</div>



## Overview

**ez-ffmpeg** provides a simple, safe, and ergonomic interface for integrating FFmpeg into Rust projects. Designed to closely mirror FFmpeg's original logic and parameter structures, this library offers:

- Complete safety without unsafe interfaces
- Preservation of FFmpeg's core parameter and processing logic
- Intuitive API for media processing
- Supports custom Rust filters
- Custom Inputs/Outputs
- Optional RTMP and OpenGL support

The library abstracts away the complexity of the raw C API while maintaining the fundamental approach of FFmpeg, allowing developers to configure media pipelines, run transcoding or filtering jobs, and inspect streams with minimal overhead.

## Version Requirements

- **Rust:** Version 1.80.0 or higher.
- **FFmpeg:** Version 7.0 or higher. (Other FFmpeg versions have not been thoroughly tested and are not recommended.)

## Documentation

More information about this crate can be found in the [crate documentation](https://docs.rs/ez-ffmpeg).

## Quick Start

### Installation Prerequisites

#### macOS
```bash
brew install ffmpeg
```

#### Windows
```bash
# For dynamic linking
vcpkg install ffmpeg

# For static linking (requires 'static' feature)
vcpkg install ffmpeg:x64-windows-static-md

# Set VCPKG_ROOT environment variable
```

### Adding the Dependency

Add **ez-ffmpeg** to your project by including it in your `Cargo.toml`:

```toml
[dependencies]
ez-ffmpeg = "*"
```

### Basic Usage

Below is a basic example to get you started. Create or update your `main.rs` with the following code:

```rust
use ez_ffmpeg::core::context::ffmpeg_context::FfmpegContext;
use ez_ffmpeg::core::scheduler::ffmpeg_scheduler::FfmpegScheduler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build the FFmpeg context
    let context = FfmpegContext::builder()
        .input("input.mp4")
        .filter_desc("hue=s=0") // Example filter: desaturate (optional)
        .output("output.mov")
        .build()?;

    // 2. Run it via FfmpegScheduler (synchronous mode)
    let result = FfmpegScheduler::new(context)
        .start()?
        .wait();
    result?; // Propagate any errors that occur
    Ok(())
}
```
More examples can be found [here][examples].

[examples]: https://github.com/YeautyYE/ez-ffmpeg/tree/master/examples

## Features

**ez-ffmpeg** offers several optional features that can be enabled in your `Cargo.toml` as needed:

- **opengl:** Enables GPU-accelerated OpenGL filters for high-performance video processing.
- **rtmp:** Includes an embedded RTMP server for local streaming scenarios.
- **flv:** Provides support for FLV container parsing and handling.
- **async:** Adds asynchronous functionality (allowing you to `.await` operations).
- **static:** Enables static linking for FFmpeg libraries (via `ffmpeg-next/static`).

## License

ez-ffmpeg is distributed under the WTFPL (Do What The F*ck You Want To Public License).
**Important:** While ez-ffmpeg is freely usable, FFmpeg has its own licensing terms. Ensure your usage complies with FFmpeg's license when using its components.