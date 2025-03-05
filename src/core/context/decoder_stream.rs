use crate::core::codec::Codec;
use crate::core::context::{FrameBox, PacketBox, Stream};
use crate::core::hwaccel::HWAccelID;
use crossbeam_channel::{Receiver, Sender};
use ffmpeg_sys_next::{
    AVCodec, AVCodecDescriptor, AVCodecID, AVCodecParameters, AVHWDeviceType, AVMediaType,
    AVPixelFormat, AVRational, AVStream,
};

#[derive(Clone)]
pub(crate) struct DecoderStream {
    pub(crate) stream_index: usize,
    pub(crate) linklabel: Option<String>,
    pub(crate) stream: Stream,
    pub(crate) codec_parameters: *mut AVCodecParameters,
    pub(crate) codec_type: AVMediaType,
    pub(crate) codec_id: AVCodecID,
    pub(crate) codec: Codec,
    pub(crate) codec_desc: *const AVCodecDescriptor,
    pub(crate) duration: i64,
    pub(crate) time_base: AVRational,
    pub(crate) avg_framerate: AVRational,
    pub(crate) have_sub2video: bool,

    pub(crate) hwaccel_id: HWAccelID,
    pub(crate) hwaccel_device_type: AVHWDeviceType,
    pub(crate) hwaccel_device: Option<String>,
    pub(crate) hwaccel_output_format: AVPixelFormat,

    pub(crate) fg_input_index: usize,

    src: Option<Receiver<PacketBox>>,
    dsts: Vec<Sender<FrameBox>>,
}

impl DecoderStream {
    pub(crate) fn new(
        stream_index: usize,
        linklabel: Option<String>,
        stream: *mut AVStream,
        codec_parameters: *mut AVCodecParameters,
        codec_type: AVMediaType,
        codec_id: AVCodecID,
        codec: *const AVCodec,
        codec_desc: *const AVCodecDescriptor,
        duration: i64,
        time_base: AVRational,
        avg_framerate: AVRational,
        hwaccel_id: HWAccelID,
        hwaccel_device_type: AVHWDeviceType,
        hwaccel_device: Option<String>,
        hwaccel_output_format: AVPixelFormat,
    ) -> Self {
        Self {
            stream_index,
            linklabel,
            stream: Stream { inner: stream },
            codec_parameters,
            codec_type,
            codec_id,
            codec: Codec::new(codec),
            codec_desc,
            duration,
            time_base,
            avg_framerate,
            have_sub2video: false,
            hwaccel_id,
            hwaccel_device_type,
            hwaccel_device,
            hwaccel_output_format,
            fg_input_index: 0,
            src: None,
            dsts: vec![],
        }
    }

    pub(crate) fn is_used(&self) -> bool {
        self.src.is_some()
    }

    pub(crate) fn set_src(&mut self, src: Receiver<PacketBox>) {
        self.src = Some(src);
    }

    pub(crate) fn add_dst(&mut self, frame_dst: Sender<FrameBox>) {
        self.dsts.push(frame_dst);
    }

    pub(crate) fn take_src(&mut self) -> Option<Receiver<PacketBox>> {
        self.src.take()
    }

    pub(crate) fn take_dsts(&mut self) -> Vec<Sender<FrameBox>> {
        std::mem::take(&mut self.dsts)
    }

    pub fn replace_dsts(&mut self, new_dsts: Sender<FrameBox>) -> Vec<Sender<FrameBox>> {
        std::mem::replace(&mut self.dsts, vec![new_dsts])
    }
}
