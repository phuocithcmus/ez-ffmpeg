use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use crossbeam_channel::{Receiver, Sender};
use crate::core::context::FrameBox;
use crate::core::context::input_filter::InputFilter;
use crate::core::context::output_filter::OutputFilter;
use crate::core::scheduler::input_controller::SchNode;

pub(crate) struct FilterGraph {
    pub(crate) graph_desc: String,
    pub(crate) hw_device: Option<String>,

    pub(crate) inputs: Vec<InputFilter>,
    pub(crate) outputs: Vec<OutputFilter>,

    pub(crate) src: Option<(crossbeam_channel::Sender<FrameBox>, crossbeam_channel::Receiver<FrameBox>)>,

    pub(crate) node: Arc<SchNode>
}

impl FilterGraph {
    pub(crate) fn new(graph_desc: String,
                      hw_device: Option<String>,
                      inputs: Vec<InputFilter>,
                      outputs: Vec<OutputFilter>) -> Self {
        Self {
            graph_desc,
            hw_device,
            inputs,
            outputs,
            src: Some(crossbeam_channel::bounded(8)),
            node: Arc::new(SchNode::Filter { inputs: Vec::new(), best_input: Arc::new(AtomicUsize::from(0)) })
        }
    }

    pub(crate) fn take_src(&mut self) -> Receiver<FrameBox> {
        let (_sender, receiver) = self.src.take().unwrap();
        receiver
    }

    pub(crate) fn get_src_sender(&mut self) -> Sender<FrameBox> {
        if self.src.is_none() {
            self.src = Some(crossbeam_channel::bounded(8));
        }
        let (sender, _) = self.src.as_ref().unwrap();
        sender.clone()
    }
}