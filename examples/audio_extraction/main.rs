use ez_ffmpeg::{FfmpegContext, FfmpegScheduler};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build the FFmpeg context for audio extraction.
    // Use "test.mp4" (provided in the repository) as the input file.
    // The output file "output.aac" indicates that only the audio stream will be extracted.
    let context = FfmpegContext::builder()
        .input("test.mp4")
        .output("output.aac")
        .build()?;

    // Start the processing pipeline synchronously.
    FfmpegScheduler::new(context)
        .start()?
        .wait()?;

    println!("Audio extraction complete: output.aac");
    Ok(())
}