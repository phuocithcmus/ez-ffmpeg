use crate::core::filter::frame_filter::FrameFilter;
use ffmpeg_sys_next::AVMediaType;
use std::cell::RefCell;
use std::rc::Rc;
use crate::filter::frame_pipeline::FramePipeline;

/// A builder for constructing [`FramePipeline`](crate::core::filter::frame_pipeline::FramePipeline) instances.
///
/// ## No Public `build` Method – Users Should Not Call It Manually
/// - `FramePipelineBuilder` does **not** expose a public `build` method.
/// - Instead, the `FfmpegScheduler` is responsible for building `FramePipeline`
///   at the appropriate time during execution.
///
/// ## Why Is `FramePipeline` Built Later?
///
/// - The `FfmpegScheduler` **delays** `FramePipeline` construction until execution starts.
/// - This ensures the correct stream mappings before creating `FramePipeline`.
///
/// ## Why Not Build `FramePipeline` Immediately?
///
/// 1️⃣ **Self-Referencing Requires `Rc<RefCell<T>>` or `Arc<Mutex<T>>`**
/// - `FramePipeline` needs **self-referencing** to allow dynamic modifications,
///   such as adding or removing filters during frame processing.
/// - Since Rust does not support self-referencing structs directly,
///   we must use `Rc<RefCell<T>>` or `Arc<Mutex<T>>` for internal mutability.
///
/// 2️⃣ **Cannot Use `Rc<RefCell<T>>` Before `FfmpegScheduler` Execution**
/// - If `FramePipeline` is pre-built as `Rc<RefCell<T>>`,
///   it **cannot be transferred** to the execution thread safely.
///
/// 3️⃣ **Avoiding Performance Overhead of `Arc<Mutex<T>>`**
/// - If `FramePipeline` were pre-built as `Arc<Mutex<T>>`,
///   it would introduce **unnecessary synchronization overhead**.
/// - Since `FramePipeline` is used **only within a single thread**,
///   `Arc<Mutex<T>>` is **not needed** and would waste performance.
///
/// ## Final Decision: Delayed Construction by `FfmpegScheduler`
/// - **To avoid unnecessary locking** (`Arc<Mutex<T>>`)
/// - **To ensure safe initialization within a single thread**
/// - **To allow `FramePipeline` self-referencing with `Rc<RefCell<T>>`**
///
/// The `FfmpegScheduler` builds `FramePipeline` **after** starting its execution thread,
/// ensuring correctness while maintaining optimal performance.
///
/// # Example
/// ```rust
/// let pipeline = FramePipelineBuilder::new(AVMediaType::AVMEDIA_TYPE_VIDEO)
///     .filter("opengl", Box::new(OpenGLFrameFilter::new())); // Add an OpenGL filter
/// ```
pub struct FramePipelineBuilder {
    /// The index of the stream being processed.
    ///
    /// This value corresponds to the `stream_index` of an input or output stream in FFmpeg.
    /// It is used to identify which stream the pipeline applies to.
    pub(crate) stream_index: Option<usize>,

    /// The FFmpeg-style link label for matching streams.
    ///
    /// In FFmpeg filter graph notation, link labels such as `0:v` (first video stream) or `1:a` (second audio stream)
    /// are used to connect different processing stages. This field allows explicit linking of streams
    /// in multi-stream processing scenarios.
    pub(crate) linklabel: Option<String>,

    /// The type of media this pipeline is processing.
    ///
    /// This field determines whether the pipeline is handling **video**, **audio**, or **subtitle** data.
    /// It is represented using `AVMediaType`, which can take values such as:
    /// - `AVMEDIA_TYPE_VIDEO` for video frames.
    /// - `AVMEDIA_TYPE_AUDIO` for audio frames.
    /// - `AVMEDIA_TYPE_SUBTITLE` for subtitle processing.
    pub(crate) media_type: AVMediaType,

    /// A list of filters to be applied in sequence.
    ///
    /// Each filter is represented by a tuple containing:
    /// - A `String` name that identifies the filter.
    /// - A `Box<dyn FrameFilter>` that holds the filter implementation.
    ///
    /// These filters will be applied to the media frames in the order they are added.
    pub(crate) filters: Vec<(String, Box<dyn FrameFilter>)>,
}

impl FramePipelineBuilder {
    /// Creates a new `FramePipelineBuilder` instance for a specific media type.
    ///
    /// This initializes a builder that can be configured with stream index, link label, and filters.
    ///
    /// # Arguments
    /// - `media_type` - The type of media being processed (`AVMEDIA_TYPE_VIDEO`, `AVMEDIA_TYPE_AUDIO`, etc.).
    ///
    /// # Returns
    /// A new `FramePipelineBuilder` instance with the given `media_type`.
    ///
    /// # Example
    /// ```rust
    /// let builder = FramePipelineBuilder::new(AVMEDIA_TYPE_VIDEO);
    /// ```
    pub fn new(media_type: AVMediaType) -> Self {
        Self {
            stream_index: None,
            linklabel: None,
            media_type,
            filters: vec![],
        }
    }

    /// Sets the FFmpeg-style link label for this pipeline.
    ///
    /// This label is used for identifying and matching streams in FFmpeg filter graphs.
    /// Examples of link labels include:
    /// - `"0:v"` for the first video stream.
    /// - `"1:a"` for the second audio stream.
    ///
    /// # Arguments
    /// - `linklabel` - A `String` or type convertible to `String` that represents the link label.
    ///
    /// # Returns
    /// The modified `FramePipelineBuilder` instance, allowing method chaining.
    ///
    /// # Example
    /// ```rust
    /// let builder = FramePipelineBuilder::new(AVMEDIA_TYPE_AUDIO)
    ///     .set_linklabel("1:a");
    /// ```
    pub fn set_linklabel(mut self, linklabel: impl Into<String>) -> Self {
        self.linklabel = Some(linklabel.into());
        self
    }

    /// Sets the stream index for this pipeline.
    ///
    /// The stream index is used to specify which input or output stream this pipeline applies to.
    /// This should match the `stream_index` of the corresponding FFmpeg stream.
    ///
    /// # Arguments
    /// - `stream_index` - The index of the media stream in the input or output file.
    ///
    /// # Returns
    /// The modified `FramePipelineBuilder` instance, allowing method chaining.
    ///
    /// # Example
    /// ```rust
    /// let builder = FramePipelineBuilder::new(AVMEDIA_TYPE_VIDEO)
    ///     .set_stream_index(0);
    /// ```
    pub fn set_stream_index(mut self, stream_index: usize) -> Self {
        self.stream_index = Some(stream_index);
        self
    }

    /// Adds a filter to the pipeline.
    ///
    /// This method registers a filter to be applied to the media frames in the pipeline.
    /// Filters are applied in the order they are added.
    ///
    /// # Arguments
    /// - `name` - The name of the filter, which serves as an identifier.
    /// - `filter` - A boxed instance of a filter that implements `FrameFilter`.
    ///
    /// # Returns
    /// The modified `FramePipelineBuilder` instance, allowing method chaining.
    ///
    /// # Example
    /// ```rust
    /// let filter = Box::new(MyCustomFilter {});
    /// let builder = FramePipelineBuilder::new(AVMEDIA_TYPE_VIDEO)
    ///     .filter("scale", filter);
    /// ```
    pub fn filter(mut self, name: &str, filter: Box<dyn FrameFilter>) -> Self {
        assert_eq!(self.media_type, filter.media_type());
        self.filters.push((name.to_string(), filter));
        self
    }

    /// **[Internal Use]** Builds the `FramePipeline` instance.
    ///
    /// This method is **automatically called by the `scheduler`** when execution begins.
    /// Users should **not call `build` manually**, because:
    /// - The input and output stream mappings are only known at runtime.
    /// - The `scheduler` determines whether streams exist before constructing pipelines.
    ///
    /// # Arguments
    /// - `stream_index`: The final determined stream index.
    /// - `linklabel`: The final determined FFmpeg link label.
    ///
    /// # Returns
    /// A reference-counted `FramePipeline` instance.
    ///
    /// # Example
    /// ```rust
    /// let pipeline = builder.build(0, Some("0:v".to_string())); // Automatically invoked
    /// ```
    ///
    /// **Warning:** Do not call this method manually. It is managed by the `scheduler`.
    pub(crate) fn build(
        mut self,
        stream_index: usize,
        linklabel: Option<String>,
    ) -> Rc<RefCell<FramePipeline>> {
        let frame_pipeline = FramePipeline::new(stream_index, linklabel, self.media_type);

        for (name, filter) in self.filters.drain(..) {
            frame_pipeline.borrow_mut().add_last(&name, filter);
        }

        frame_pipeline
    }
}

impl From<AVMediaType> for FramePipelineBuilder {
    fn from(media_type: AVMediaType) -> Self {
        Self::new(media_type)
    }
}
