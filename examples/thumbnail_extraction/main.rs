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
        .output(Output::from("output.jpg").set_max_video_frames(1))
        // Build the context, start the process and wait for completion
        .build().unwrap()
        .start().unwrap()
        .wait().unwrap();
}
