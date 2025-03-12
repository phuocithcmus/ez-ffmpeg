use ffmpeg_next::format;

/// Gets the duration of a media file in microseconds.
///
/// # Arguments
/// - `input`: The path to the input file (e.g., `"video.mp4"`).
///
/// # Returns
/// - `Result<i64, ffmpeg_next::Error>`: Returns the duration as an `i64` value in microseconds.
///   If an error occurs, it returns an `ffmpeg_next::Error`.
///
/// # Example
/// ```rust
/// let duration = get_duration_us("video.mp4").unwrap();
/// println!("Duration: {} us", duration);
/// ```
pub fn get_duration_us(input: &str) -> Result<i64, ffmpeg_next::Error> {
    // Open the media file using `format::input` and get the `FormatContext`
    let format_context = format::input(input)?;

    // Get the duration of the media file in microseconds
    let duration = format_context.duration();

    // Return the duration
    Ok(duration)
}

/// Gets the format name of a media file (e.g., "mp4", "avi").
///
/// # Arguments
/// - `input`: The path to the input file (e.g., `"video.mp4"`).
///
/// # Returns
/// - `Result<String, ffmpeg_next::Error>`: Returns a string representing the format of the media file.
///   If an error occurs, it returns an `ffmpeg_next::Error`.
///
/// # Example
/// ```rust
/// let format = get_format("video.mp4").unwrap();
/// println!("Format: {}", format);
/// ```
pub fn get_format(input: &str) -> Result<String, ffmpeg_next::Error> {
    // Open the media file using `format::input` and get the `FormatContext`
    let format_context = format::input(input)?;

    // Get the format name of the media file and return it as a string
    Ok(format_context.format().name().to_string())
}

/// Gets the metadata of a media file (e.g., title, artist).
///
/// # Arguments
/// - `input`: The path to the input file (e.g., `"video.mp4"`).
///
/// # Returns
/// - `Result<Vec<(String, String)>, ffmpeg_next::Error>`: Returns a vector of key-value pairs representing the metadata.
///   Each key is a metadata field (e.g., `"title"`, `"artist"`) and each value is the corresponding value.
///   If an error occurs, it returns an `ffmpeg_next::Error`.
///
/// # Example
/// ```rust
/// let metadata = get_metadata("video.mp4").unwrap();
/// for (key, value) in metadata {
///     println!("{}: {}", key, value);
/// }
/// ```
pub fn get_metadata(input: &str) -> Result<Vec<(String, String)>, ffmpeg_next::Error> {
    // Open the media file using `format::input` and get the `FormatContext`
    let format_context = format::input(input)?;

    // Get the metadata and convert it to a vector of key-value pairs
    Ok(format_context
        .metadata()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect())
}
