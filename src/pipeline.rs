use crate::channel::Channel;

use gstreamer::{
    self as gst,
    prelude::{ElementExtManual, GstBinExtManual},
};

#[derive(Clone)]
pub enum PicommPipeline {
    Receiver([Channel; 4]),
}

impl PicommPipeline {
    pub fn construct(self) -> Result<gst::Pipeline, Box<dyn std::error::Error>> {
        let pipeline = gst::Pipeline::new();

        match self {
            Self::Receiver(channels) => {
                let rtp_caps = gst::Caps::builder("application/x-rtp")
                    .field("clock-rate", 48000)
                    .field("media", "audio")
                    .field("encoding-name", "OPUS")
                    .build();

                let mixer = gst::ElementFactory::make("audiomixer").build()?;

                for channel in channels {
                    let (multicast_ip, multicast_port) = channel.get_multicast();

                    let src = gst::ElementFactory::make("udpsrc")
                        .property("multicast-group", multicast_ip)
                        .property("port", multicast_port as u32)
                        .property("caps", rtp_caps.clone())
                        .build()?;

                    let depay = gst::ElementFactory::make("rtpopusdepay").build()?;
                    let decode = gst::ElementFactory::make("opusdec").build()?;
                    let convert = gst::ElementFactory::make("audioconvert").build()?;
                    let resample = gst::ElementFactory::make("audioresample").build()?;
                    let queue = gst::ElementFactory::make("queue").build()?;

                    pipeline.add_many([&src, &depay, &decode, &convert, &resample, &queue])?;
                    gst::Element::link_many([&src, &depay, &decode, &convert, &resample, &queue])?;

                    queue.link(&mixer)?;
                }

                let sink = gst::ElementFactory::make("autoaudiosink").build()?;
                pipeline.add_many([&mixer, &sink])?;
                mixer.link(&sink)?;
            }
        }

        Ok(pipeline)
    }
}
