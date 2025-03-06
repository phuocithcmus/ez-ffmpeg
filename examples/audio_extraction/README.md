# ez-ffmpeg Example: Audio Extraction

This example demonstrates how to extract the audio stream from a video file using `ez-ffmpeg`.

## Code Explanation

- **Input Video:** `test.mp4` (provided in the repository)
- **Extraction Process:**
    - The example builds an FFmpeg processing pipeline using the ez‑ffmpeg builder API.
    - The input video is processed to retain only the audio stream.
    - The output file extension `.aac` indicates that the resulting file will contain audio only.
- **Output File:** The extracted audio is saved as `output.aac`.

### Key FFmpeg Behavior

- **Output File Format:** The use of the `.aac` extension instructs FFmpeg to drop the video stream and process only the audio.
- **Default Processing:** No additional filters or modifications are applied—the audio is extracted in its original form.

## When to Use

- **Use this method** when you need to separate the audio track from a video file for analysis, further processing, or standalone audio playback.

