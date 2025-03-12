use crate::core::context::decoder_stream::DecoderStream;
use crate::core::context::encoder_stream::EncoderStream;
use crate::core::context::obj_pool::ObjPool;
use crate::core::context::{FrameBox, FrameData};
use crate::core::scheduler::type_to_symbol;
use crate::error::Error::{FrameFilterInit, FrameFilterLinkLabelNoMatched, FrameFilterProcess, FrameFilterRequest, FrameFilterSendOOM, FrameFilterStreamTypeNoMatched, FrameFilterThreadExited, FrameFilterTypeNoMatched};
use crate::filter::frame_filter_context::FrameFilterContext;
use crate::filter::frame_pipeline::FramePipeline;
use crate::filter::frame_pipeline_builder::FramePipelineBuilder;
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use ffmpeg_next::Frame;
use ffmpeg_sys_next::{av_frame_copy_props, av_frame_ref};
use log::{debug, error, info, warn};
use std::cell::RefCell;
use std::ops::Deref;
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

pub(crate) fn input_pipeline_init(
    demux_idx: usize,
    pipeline_builder: FramePipelineBuilder,
    decoder_streams: &mut Vec<DecoderStream>,
    frame_pool: ObjPool<Frame>,
    scheduler_status: Arc<AtomicUsize>,
    scheduler_result: Arc<Mutex<Option<crate::error::Result<()>>>>,
) -> crate::error::Result<()> {
    if pipeline_builder.filters.is_empty() {
        warn!("pipeline filters is empty");
        return Ok(());
    }

    // Match type to find index and linklabel.
    let (stream_index, linklabel, encoder_frame_receiver, pipeline_frame_senders, fg_input_index) =
        match_decoder_stream(&pipeline_builder, decoder_streams)?;

    pipeline_init(
        true,
        demux_idx,
        pipeline_builder,
        stream_index,
        linklabel,
        encoder_frame_receiver,
        pipeline_frame_senders,
        fg_input_index,
        frame_pool,
        scheduler_status,
        scheduler_result,
    )
}
pub(crate) fn output_pipeline_init(
    mux_idx: usize,
    pipeline_builder: FramePipelineBuilder,
    encoder_streams: &mut Vec<EncoderStream>,
    frame_pool: ObjPool<Frame>,
    scheduler_status: Arc<AtomicUsize>,
    scheduler_result: Arc<Mutex<Option<crate::error::Result<()>>>>,
) -> crate::error::Result<()> {
    if pipeline_builder.filters.is_empty() {
        warn!("pipeline filters is empty");
        return Ok(());
    }

    // Match type to find index and linklabel.
    let (stream_index, linklabel, encoder_frame_receiver, pipeline_frame_sender) =
        match_encoder_stream(&pipeline_builder, encoder_streams)?;

    pipeline_init(
        false,
        mux_idx,
        pipeline_builder,
        stream_index,
        linklabel,
        encoder_frame_receiver,
        vec![pipeline_frame_sender],
        0,
        frame_pool,
        scheduler_status,
        scheduler_result,
    )
}

fn match_decoder_stream(
    pipeline_builder: &FramePipelineBuilder,
    decoder_streams: &mut Vec<DecoderStream>,
) -> crate::error::Result<(
    usize,
    Option<String>,
    Receiver<FrameBox>,
    Vec<Sender<FrameBox>>,
    usize
)> {
    let (stream_index, linklabel, pipeline_frame_receiver, decoder_frame_senders, fg_input_index) =
        match pipeline_builder.stream_index {
            Some(stream_index) => {
                match decoder_streams
                    .iter_mut()
                    .find(|decoder_stream| decoder_stream.stream_index == stream_index)
                {
                    None => {
                        return Err(FrameFilterStreamTypeNoMatched(
                            stream_index,
                            format!("{:?}", pipeline_builder.media_type),
                        ))
                    }
                    Some(decoder_stream) => {
                        let (pipeline_frame_sender, pipeline_frame_receiver) = crossbeam_channel::bounded(8);
                        let decoder_frame_senders =
                            decoder_stream.replace_dsts(pipeline_frame_sender);

                        (
                            stream_index,
                            decoder_stream.linklabel.clone(),
                            pipeline_frame_receiver,
                            decoder_frame_senders,
                            decoder_stream.fg_input_index,
                        )
                    }
                }
            }
            None => match pipeline_builder.linklabel.clone() {
                None => match decoder_streams
                    .iter_mut()
                    .find(|decoder_stream| decoder_stream.codec_type == pipeline_builder.media_type)
                {
                    None => {
                        return Err(FrameFilterTypeNoMatched(format!(
                            "{:?}",
                            pipeline_builder.media_type
                        )))
                    }
                    Some(decoder_stream) => {
                        let (pipeline_frame_sender, pipeline_frame_receiver) = crossbeam_channel::bounded(8);
                        let decoder_frame_senders =
                            decoder_stream.replace_dsts(pipeline_frame_sender);
                        (
                            decoder_stream.stream_index,
                            decoder_stream.linklabel.clone(),
                            pipeline_frame_receiver,
                            decoder_frame_senders,
                            decoder_stream.fg_input_index,
                        )
                    }
                },
                Some(linklabel) => match decoder_streams
                    .iter_mut()
                    .find(|decoder_stream| decoder_stream.linklabel == Some(linklabel.clone()))
                {
                    None => {
                        return Err(FrameFilterLinkLabelNoMatched(linklabel));
                    }
                    Some(decoder_stream) => {
                        let (pipeline_frame_sender, pipeline_frame_receiver) = crossbeam_channel::bounded(8);
                        let decoder_frame_senders =
                            decoder_stream.replace_dsts(pipeline_frame_sender);
                        (
                            decoder_stream.stream_index,
                            Some(linklabel),
                            pipeline_frame_receiver,
                            decoder_frame_senders,
                            decoder_stream.fg_input_index,
                        )
                    }
                },
            },
        };
    Ok((
        stream_index,
        linklabel,
        pipeline_frame_receiver,
        decoder_frame_senders,
        fg_input_index
    ))
}

fn match_encoder_stream(
    pipeline_builder: &FramePipelineBuilder,
    encoder_streams: &mut Vec<EncoderStream>,
) -> crate::error::Result<(usize, Option<String>, Receiver<FrameBox>, Sender<FrameBox>)> {
    let (stream_index, linklabel, encoder_frame_receiver, pipeline_frame_sender) =
        match pipeline_builder.stream_index {
            Some(stream_index) => {
                match encoder_streams
                    .iter_mut()
                    .find(|encoder_stream| encoder_stream.stream_index == stream_index)
                {
                    None => {
                        return Err(FrameFilterStreamTypeNoMatched(
                            stream_index,
                            format!("{:?}", pipeline_builder.media_type),
                        ))
                    }
                    Some(encoder_stream) => {
                        let (pipeline_frame_sender, pipeline_frame_receiver) = crossbeam_channel::bounded(8);
                        let encoder_frame_receiver =
                            encoder_stream.replace_src(pipeline_frame_receiver);

                        (
                            stream_index,
                            encoder_stream.linklabel.clone(),
                            encoder_frame_receiver,
                            pipeline_frame_sender,
                        )
                    }
                }
            }
            None => match pipeline_builder.linklabel.clone() {
                None => match encoder_streams
                    .iter_mut()
                    .find(|encoder_stream| encoder_stream.codec_type == pipeline_builder.media_type)
                {
                    None => {
                        return Err(FrameFilterTypeNoMatched(format!(
                            "{:?}",
                            pipeline_builder.media_type
                        )))
                    }
                    Some(encoder_stream) => {
                        let (pipeline_frame_sender, pipeline_frame_receiver) = crossbeam_channel::bounded(8);
                        let encoder_frame_receiver =
                            encoder_stream.replace_src(pipeline_frame_receiver);

                        (
                            encoder_stream.stream_index,
                            encoder_stream.linklabel.clone(),
                            encoder_frame_receiver,
                            pipeline_frame_sender,
                        )
                    }
                },
                Some(linklabel) => match encoder_streams
                    .iter_mut()
                    .find(|encoder_stream| encoder_stream.linklabel == Some(linklabel.clone()))
                {
                    None => {
                        return Err(FrameFilterLinkLabelNoMatched(linklabel));
                    }
                    Some(encoder_stream) => {
                        let (pipeline_frame_sender, pipeline_frame_receiver) = crossbeam_channel::bounded(8);
                        let encoder_frame_receiver =
                            encoder_stream.replace_src(pipeline_frame_receiver);

                        (
                            encoder_stream.stream_index,
                            Some(linklabel),
                            encoder_frame_receiver,
                            pipeline_frame_sender,
                        )
                    }
                },
            },
        };
    Ok((
        stream_index,
        linklabel,
        encoder_frame_receiver,
        pipeline_frame_sender,
    ))
}

fn pipeline_init(
    is_input: bool,
    demux_mux_idx: usize,
    pipeline_builder: FramePipelineBuilder,
    stream_index: usize,
    linklabel: Option<String>,
    frame_receiver: Receiver<FrameBox>,
    frame_senders: Vec<Sender<FrameBox>>,
    fg_input_index: usize,
    frame_pool: ObjPool<Frame>,
    scheduler_status: Arc<AtomicUsize>,
    scheduler_result: Arc<Mutex<Option<crate::error::Result<()>>>>,
) -> crate::error::Result<()> {
    let pipeline_name = if is_input {
        "input-frame-pipeline".to_string()
    } else {
        "output-frame-pipeline".to_string()
    };
    let result = std::thread::Builder::new()
        .name(format!(
            "{pipeline_name}:{}:{stream_index}:{demux_mux_idx}",
            type_to_symbol(pipeline_builder.media_type),
        ))
        .spawn(move || {
            let mut pipeline = pipeline_builder.build(stream_index, linklabel);
            if let Err(e) = frame_filter_init(&pipeline) {
                pipeline_uninit(&mut pipeline);
                crate::core::scheduler::ffmpeg_scheduler::set_scheduler_error(
                    &scheduler_status,
                    &scheduler_result,
                    e,
                );
                return;
            }

            if let Err(e) = run_pipeline(
                &pipeline,
                frame_receiver,
                frame_senders,
                fg_input_index,
                &frame_pool,
                &scheduler_status,
            ) {
                crate::core::scheduler::ffmpeg_scheduler::set_scheduler_error(
                    &scheduler_status,
                    &scheduler_result,
                    e,
                );
            }

            pipeline_uninit(&mut pipeline);
        });

    if let Err(e) = result {
        error!("Pipeline thread exited with error: {e}");
        return Err(FrameFilterThreadExited);
    }

    Ok(())
}

fn run_pipeline(
    pipeline: &Rc<RefCell<FramePipeline>>,
    frame_receiver: Receiver<FrameBox>,
    mut frame_senders: Vec<Sender<FrameBox>>,
    fg_input_index: usize,
    frame_pool: &ObjPool<Frame>,
    scheduler_status: &Arc<AtomicUsize>,
) -> crate::error::Result<()> {
    let mut src_finished_flag = false;

    loop {
        if crate::core::scheduler::ffmpeg_scheduler::wait_until_not_paused(&scheduler_status)
            == crate::core::scheduler::ffmpeg_scheduler::STATUS_END
        {
            info!("Receiver end command, finishing.");
            return Ok(());
        }

        if !src_finished_flag {
            let result = frame_receiver.recv_timeout(Duration::from_millis(1));
            match result {
                Err(e) => {
                    if e == RecvTimeoutError::Disconnected {
                        src_finished_flag = true;
                        debug!("Source[decoder/filtergraph] thread exit.");
                        continue;
                    }
                }
                Ok(frame_box) => {
                    // filter frame
                    let frame_filter_context = { pipeline.borrow().head.clone() };
                    run_filter_frame(
                        pipeline,
                        frame_box.frame,
                        frame_filter_context,
                        &mut frame_senders,
                        fg_input_index,
                        frame_pool,
                    )?;

                    if frame_senders.len() == 0 {
                        debug!("All frame sender finished, finishing.");
                        return Ok(());
                    }
                }
            }
        } else { sleep(Duration::from_millis(1)) }

        // request frame
        let mut next = { pipeline.borrow().head.clone() };
        loop {
            if next.is_none() {
                break;
            }

            let frame_filter_context = next.unwrap();
            // request frame and send to next filter or destination
            loop {
                let (next_filter, tmp_frame) = do_request_frame(pipeline, &frame_filter_context)?;

                if tmp_frame.is_none() {
                    break;
                }

                run_filter_frame(
                    pipeline,
                    tmp_frame.unwrap(),
                    next_filter,
                    &mut frame_senders,
                    fg_input_index,
                    frame_pool,
                )?;
            }

            next = frame_filter_context.borrow().next.clone();
        }

        if frame_senders.len() == 0 {
            debug!("All frame sender finished, finishing.");
            return Ok(());
        }
    }
}

fn run_filter_frame(
    pipeline: &Rc<RefCell<FramePipeline>>,
    frame: Frame,
    mut next: Option<Rc<RefCell<FrameFilterContext>>>,
    frame_senders: &mut Vec<Sender<FrameBox>>,
    fg_input_index: usize,
    frame_pool: &ObjPool<Frame>,
) -> crate::error::Result<()> {
    if frame_senders.len() == 0 {
        return Ok(());
    }
    let mut tmp_frame = Some(frame);
    loop {
        if tmp_frame.is_none() {
            return Ok(());
        }
        if next.is_none() {
            break;
        }
        let frame = tmp_frame.unwrap();
        (next, tmp_frame) = do_filter_frame(pipeline, &next.unwrap(), frame)?;
    }
    if let Some(frame) = tmp_frame {
        let frame_box = FrameBox {
            frame,
            frame_data: FrameData {
                framerate: None,
                bits_per_raw_sample: 0,
                input_stream_width: 0,
                input_stream_height: 0,
                subtitle_header_size: 0,
                subtitle_header: null_mut(),
                fg_input_index,
            },
        };

        let mut finished_senders = Vec::new();
        for (i, sender) in frame_senders.iter().enumerate() {
            if i < frame_senders.len() - 1 {
                let mut to_send = frame_pool.get()?;

                // frame may sometimes contain props only,
                // e.g. to signal EOF timestamp
                unsafe {
                    if !(*frame_box.frame.as_ptr()).buf[0].is_null() {
                        let ret = av_frame_ref(to_send.as_mut_ptr(), frame_box.frame.as_ptr());
                        if ret < 0 {
                            return Err(FrameFilterSendOOM);
                        }
                    } else {
                        let ret =
                            av_frame_copy_props(to_send.as_mut_ptr(), frame_box.frame.as_ptr());
                        if ret < 0 {
                            return Err(FrameFilterSendOOM);
                        }
                    };
                }
                let frame_box = FrameBox {
                    frame: to_send,
                    frame_data: frame_box.frame_data.clone(),
                };
                if let Err(_) = sender.send(frame_box) {
                    debug!(
                        "Pipeline [index:{} linklabel:{}] send frame failed, destination already finished",
                        pipeline.borrow().stream_index,
                        pipeline
                            .borrow()
                            .linklabel
                            .clone()
                            .unwrap_or("".to_string())
                    );
                    finished_senders.push(i);
                    continue;
                }
            } else {
                if let Err(_) = sender.send(frame_box) {
                    debug!("Pipeline [index:{} linklabel:{}] send frame failed, destination already finished",
                        pipeline.borrow().stream_index,
                        pipeline
                            .borrow()
                            .linklabel
                            .clone()
                            .unwrap_or("".to_string())
                    );
                    finished_senders.push(i);
                }
                break;
            }
        }

        for i in finished_senders {
            frame_senders.remove(i);
        }
    }

    Ok(())
}

fn do_filter_frame(
    pipeline: &Rc<RefCell<FramePipeline>>,
    frame_filter_context: &Rc<RefCell<FrameFilterContext>>,
    frame: Frame,
) -> crate::error::Result<(Option<Rc<RefCell<FrameFilterContext>>>, Option<Frame>)> {
    let mut_frame_filter_context = frame_filter_context.borrow_mut();
    let frame_filter = mut_frame_filter_context.filter();
    let mut frame_filter = frame_filter.borrow_mut();

    let result = frame_filter.filter_frame(frame, mut_frame_filter_context.deref());
    if let Err(e) = result {
        error!(
            "Pipeline [index:{} linklabel:{}] failed, during filter frame. error: {e}",
            pipeline.borrow().stream_index,
            pipeline
                .borrow()
                .linklabel
                .clone()
                .unwrap_or("".to_string())
        );
        return Err(FrameFilterProcess(e));
    }


    Ok((mut_frame_filter_context.next.clone(), result.unwrap()))
}

fn do_request_frame(
    pipeline: &Rc<RefCell<FramePipeline>>,
    frame_filter_context: &Rc<RefCell<FrameFilterContext>>,
) -> crate::error::Result<(Option<Rc<RefCell<FrameFilterContext>>>, Option<Frame>)> {
    let mut_frame_filter_context = frame_filter_context.borrow_mut();
    let frame_filter = mut_frame_filter_context.filter();
    let mut frame_filter = frame_filter.borrow_mut();

    let result = frame_filter.request_frame(mut_frame_filter_context.deref());
    if let Err(e) = result {
        error!(
            "Pipeline [index:{} linklabel:{}] failed, during request frame.",
            pipeline.borrow().stream_index,
            pipeline
                .borrow()
                .linklabel
                .clone()
                .unwrap_or("".to_string())
        );
        return Err(FrameFilterRequest(e));
    }

    Ok((mut_frame_filter_context.next.clone(), result.unwrap()))
}

fn pipeline_uninit(pipeline: &mut Rc<RefCell<FramePipeline>>) {
    let mut frame_filter_ctx = { pipeline.borrow_mut().head.take().unwrap() };
    loop {
        let next = {
            let mut mut_frame_filter_context = frame_filter_ctx.borrow_mut();
            let frame_filter = mut_frame_filter_context.filter();
            let mut frame_filter = frame_filter.borrow_mut();
            frame_filter.uninit(mut_frame_filter_context.deref());

            let next = mut_frame_filter_context.next.take();
            next
        };

        if let Some(next_context) = next {
            frame_filter_ctx = next_context;
        } else {
            break;
        }
    }
    pipeline.borrow_mut().tail.take();
}

fn frame_filter_init(pipeline: &Rc<RefCell<FramePipeline>>) -> crate::error::Result<()> {
    let mut frame_filter_ctx = { pipeline.borrow().head.clone().unwrap() };
    loop {
        let next = {
            let mut_frame_filter_context = frame_filter_ctx.borrow_mut();
            let frame_filter = mut_frame_filter_context.filter();
            let mut frame_filter = frame_filter.borrow_mut();
            if let Err(e) = frame_filter.init(mut_frame_filter_context.deref()) {
                return Err(FrameFilterInit(e));
            }

            mut_frame_filter_context.next.clone()
        };

        if let Some(next_context) = next {
            frame_filter_ctx = next_context;
        } else {
            break;
        }
    }
    Ok(())
}
