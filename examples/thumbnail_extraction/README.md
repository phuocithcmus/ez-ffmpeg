# ez-ffmpeg Example: Thumbnail Extraction

This example demonstrates how to extract a thumbnail image from a video using `ez-ffmpeg`.

## Code Explanation

- **Input Video:** `test.mp4`
- **Thumbnail Extraction:**
    - The `scale` filter resizes the video to a width of `160`, while automatically adjusting the height to maintain the aspect ratio (using `-1`).
    - The `min(160, iw)` expression ensures that the width doesn't exceed 160 pixels.

- **Output:** The first frame of the video is extracted and saved as `output.jpg`.

### Key FFmpeg Filter:

- `scale='min(160,iw)':-1`: Resizes the video to a maximum width of 160 pixels while preserving the aspect ratio by adjusting the height automatically.

### Key FFmpeg Options:

- `set_max_video_frames(1)`: Limits the output to 1 video frame, effectively extracting only a single frame (the thumbnail).

## When to Use

- **Use this method** when you need to extract a thumbnail or preview image from a video, resizing it to fit within a specific width while maintaining the aspect ratio.
