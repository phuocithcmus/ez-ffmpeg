use crate::core::filter::frame_filter::{FrameFilter, NoopFilter};
use crate::core::filter::frame_pipeline::FramePipeline;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};

pub struct FrameFilterContext {
    pub(crate) name: String,
    pub(crate) frame_filter: Rc<RefCell<Box<dyn FrameFilter>>>,
    pub(crate) prev: Option<Weak<RefCell<FrameFilterContext>>>,
    pub(crate) next: Option<Rc<RefCell<FrameFilterContext>>>,
    pub(crate) frame_pipeline: Rc<RefCell<FramePipeline>>,
}

impl FrameFilterContext {

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn pipeline(&self) -> Rc<RefCell<FramePipeline>> {
        self.frame_pipeline.clone()
    }

    pub(crate) fn new(
        name: &str,
        frame_filter: Box<dyn FrameFilter>,
        frame_pipeline: Rc<RefCell<FramePipeline>>,
    ) -> Self {
        Self {
            name: name.to_string(),
            frame_filter:Rc::new(RefCell::new(frame_filter)),
            prev: None,
            next: None,
            frame_pipeline,
        }
    }


    pub(crate) fn filter(&self) -> Rc<RefCell<Box<dyn FrameFilter>>> {
        self.frame_filter.clone()
    }

    pub(crate) fn filter_ref(&self) -> Ref<Box<dyn FrameFilter>> {
        self.frame_filter.borrow()
    }

    pub(crate) fn filter_mut(&mut self) -> RefMut<Box<dyn FrameFilter>> {
        self.frame_filter.borrow_mut()
    }

    pub(crate) fn take_filter(&self) -> Box<dyn FrameFilter> {
        std::mem::replace(&mut *self.frame_filter.borrow_mut(), Box::new(NoopFilter {}))
    }

    pub(crate) fn replace_filter(&self, new_filter: Box<dyn FrameFilter>) -> Box<dyn FrameFilter> {
        std::mem::replace(&mut *self.frame_filter.borrow_mut(), new_filter)
    }
}
