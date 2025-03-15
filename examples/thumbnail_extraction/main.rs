use ez_ffmpeg::{FfmpegContext, Output};

fn main() {
    // Build the FFmpeg context with the input file
    FfmpegContext::builder()
        // Specify the input video file
        .input("test.mp4")
        // Apply the scale filter to generate a thumbnail
        // The scale filter resizes the video to have a width of 160, preserving aspect ratio
        // The height is automatically adjusted with -1 to maintain the aspect ratio
        .filter_desc("scale='min(160,iw)':-1")
        // Set the output file to a JPEG image and limit the number of video frames to 1
        .output(Output::from("output.jpg")
            // Limit the output to only 1 frame, effectively creating a thumbnail
            // This is equivalent to the -vframes 1 option in FFmpeg CLI
            .set_max_video_frames(1)
            // Set the JPEG quality level to 2 (high quality)
            // For JPEG, the scale is 2-31, where 2-5 is high quality
            // Lower values produce better quality but larger file sizes
            .set_video_qscale(2)
        )
        // Build the context, start the process and wait for completion
        .build().unwrap()
        .start().unwrap()
        .wait().unwrap();
}
