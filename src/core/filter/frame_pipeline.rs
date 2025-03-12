use crate::core::filter::frame_filter::FrameFilter;
use crate::core::filter::frame_filter_context::FrameFilterContext;
use ffmpeg_sys_next::AVMediaType;
use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

pub struct FramePipeline {
    pub(crate) stream_index: usize,
    pub(crate) linklabel: Option<String>,
    pub(crate) media_type: AVMediaType,
    pub(crate) head: Option<Rc<RefCell<FrameFilterContext>>>,
    pub(crate) tail: Option<Rc<RefCell<FrameFilterContext>>>,
    frame_pipeline: Weak<RefCell<FramePipeline>>,
    attribute_map: HashMap<String, Box<dyn Any>>,
}
impl FramePipeline {
    pub(crate) fn new(
        stream_index: usize,
        linklabel: Option<String>,
        media_type: AVMediaType,
    ) -> Rc<RefCell<FramePipeline>> {
        let frame_pipeline = Rc::new(RefCell::new(Self {
            stream_index,
            linklabel,
            media_type,
            head: None,
            tail: None,
            frame_pipeline: Weak::new(),
            attribute_map: Default::default(),
        }));

        frame_pipeline.borrow_mut().frame_pipeline = Rc::downgrade(&frame_pipeline);

        frame_pipeline
    }

    pub fn add_first(&mut self, name: &str, filter: Box<dyn FrameFilter>) {
        assert_eq!(self.media_type, filter.media_type());
        let context = Rc::new(RefCell::new(FrameFilterContext::new(
            name,
            filter,
            self.frame_pipeline.upgrade().unwrap(),
        )));
        if let Some(head) = self.head.take() {
            head.borrow_mut().prev = Some(Rc::downgrade(&context));
            context.borrow_mut().next = Some(head);
        }
        self.head = Some(context.clone());
        if self.tail.is_none() {
            self.tail = Some(context);
        }
    }

    pub fn add_last(&mut self, name: &str, filter: Box<dyn FrameFilter>) {
        assert_eq!(self.media_type, filter.media_type());
        let context = Rc::new(RefCell::new(FrameFilterContext::new(
            name,
            filter,
            self.frame_pipeline.upgrade().unwrap(),
        )));
        if let Some(tail) = self.tail.take() {
            tail.borrow_mut().next = Some(context.clone());
            context.borrow_mut().prev = Some(Rc::downgrade(&tail));
        }
        self.tail = Some(context.clone());
        if self.head.is_none() {
            self.head = Some(context);
        }
    }

    pub fn add_before(
        &mut self,
        base_name: &str,
        name: &str,
        filter: Box<dyn FrameFilter>,
    ) -> bool {
        assert_eq!(self.media_type, filter.media_type());
        let mut current = self.head.clone();
        while let Some(node) = current {
            if node.borrow().name == base_name {
                let context = Rc::new(RefCell::new(FrameFilterContext::new(
                    name,
                    filter,
                    self.frame_pipeline.upgrade().unwrap(),
                )));
                let mut node_mut = node.borrow_mut();
                context.borrow_mut().next = Some(node.clone());
                if let Some(prev) = node_mut.prev.take() {
                    if let Some(prev) = prev.upgrade() {
                        prev.borrow_mut().next = Some(context.clone());
                        context.borrow_mut().prev = Some(Rc::downgrade(&prev));
                    }
                } else {
                    self.head = Some(context.clone());
                }
                node_mut.prev = Some(Rc::downgrade(&context));
                return true;
            }
            current = node.borrow().next.clone();
        }
        false
    }

    pub fn add_after(&mut self, base_name: &str, name: &str, filter: Box<dyn FrameFilter>) -> bool {
        assert_eq!(self.media_type, filter.media_type());
        let mut current = self.head.clone();
        while let Some(node) = current {
            if node.borrow().name == base_name {
                let context = Rc::new(RefCell::new(FrameFilterContext::new(
                    name,
                    filter,
                    self.frame_pipeline.upgrade().unwrap(),
                )));
                let mut node_mut = node.borrow_mut();
                context.borrow_mut().prev = Some(Rc::downgrade(&node));
                if let Some(next) = node_mut.next.take() {
                    next.borrow_mut().prev = Some(Rc::downgrade(&context));
                    context.borrow_mut().next = Some(next);
                } else {
                    self.tail = Some(context.clone());
                }
                node_mut.next = Some(context.clone());
                return true;
            }
            current = node.borrow().next.clone();
        }
        false
    }

    pub fn remove(&mut self, name: &str) -> Option<Box<dyn FrameFilter>> {
        let mut current = self.head.clone();
        while let Some(node) = current {
            if node.borrow().name == name {
                let mut node_mut = node.borrow_mut();

                let filter = node_mut.take_filter();

                if let Some(prev) = node_mut.prev.take() {
                    if let Some(prev) = prev.upgrade() {
                        prev.borrow_mut().next = node_mut.next.clone();
                    }
                } else {
                    self.head = node_mut.next.clone();
                }

                if let Some(next) = node_mut.next.take() {
                    next.borrow_mut().prev = node_mut.prev.clone();
                } else {
                    self.tail = node_mut.prev.clone().and_then(|prev| prev.upgrade());
                }

                node_mut.prev = None;
                node_mut.next = None;

                return Some(filter);
            }
            current = node.borrow().next.clone();
        }
        None
    }

    pub fn find(&self, name: &str) -> Option<Rc<RefCell<FrameFilterContext>>> {
        let mut current = self.head.clone();
        while let Some(node) = current {
            if node.borrow().name == name {
                return Some(node);
            }
            current = node.borrow().next.clone();
        }
        None
    }

    pub fn replace(
        &mut self,
        old_name: &str,
        new_name: &str,
        new_filter: Box<dyn FrameFilter>,
    ) -> Option<Box<dyn FrameFilter>> {
        assert_eq!(self.media_type, new_filter.media_type());
        let mut current = self.head.clone();
        while let Some(node) = current {
            if node.borrow().name == old_name {
                let mut node_mut = node.borrow_mut();

                let old_filter = node_mut.replace_filter(new_filter);

                node_mut.name = new_name.to_string();

                return Some(old_filter);
            }
            current = node.borrow().next.clone();
        }
        None
    }

    pub fn first_context(&self) -> Option<Ref<FrameFilterContext>> {
        self.head.as_ref().map(|head| head.borrow())
    }

    pub fn first_context_mut(&mut self) -> Option<RefMut<FrameFilterContext>> {
        self.head.as_mut().map(|head| head.borrow_mut())
    }

    pub fn last_context(&self) -> Option<Ref<FrameFilterContext>> {
        self.tail.as_ref().map(|tail| tail.borrow())
    }

    pub fn last_context_mut(&mut self) -> Option<RefMut<FrameFilterContext>> {
        self.tail.as_mut().map(|tail| tail.borrow_mut())
    }

    pub fn set_attribute<T: 'static>(&mut self, key: String, value: T) {
        self.attribute_map.insert(key, Box::new(value));
    }

    pub fn get_attribute<T: 'static>(&self, key: &str) -> Option<&T> {
        self.attribute_map
            .get(key)
            .and_then(|value| value.downcast_ref::<T>())
    }

    pub fn remove_attribute<T: 'static>(&mut self, key: &str) -> Option<Box<dyn Any>> {
        self.attribute_map.remove(key)
    }
}
