use crate::{channel::Channel, volume::VolumeHandle};

use gstreamer::{
    self as gst,
    glib::object::ObjectExt,
    prelude::{ElementExtManual, GstBinExt, GstBinExtManual},
};

const NUM_CHANNELS: usize = 4;

fn get_audio_src() -> &'static str {
    #[cfg(feature = "rpi")]
    return "pulsesrc";

    "autoaudiosrc"
}

fn get_audio_sink() -> &'static str {
    #[cfg(feature = "rpi")]
    return "pulsesink";

    "autoaudiosink"
}

#[derive(Debug, Clone)]
pub enum PicommPipeline {
    Receiver([Channel; NUM_CHANNELS]),
    Transmitter(Channel),
}

impl PicommPipeline {
    pub fn construct(
        self,
    ) -> Result<
        (
            gst::Pipeline,
            Option<[VolumeHandle; NUM_CHANNELS]>,
            Option<VolumeHandle>,
        ),
        Box<dyn std::error::Error>,
    > {
        let pipeline = gst::Pipeline::new();

        match self {
            Self::Receiver(channels) => {
                let rtp_caps = gst::Caps::builder("application/x-rtp")
                    .field("clock-rate", 48000)
                    .field("media", "audio")
                    .field("encoding-name", "OPUS")
                    .build();

                let mixer = gst::ElementFactory::make("audiomixer").build()?;
                pipeline.add(&mixer)?;

                let mut volume_handles = Vec::with_capacity(NUM_CHANNELS);

                for channel in channels {
                    let (multicast_ip, multicast_port) = channel.get_multicast();

                    let src = gst::ElementFactory::make("udpsrc")
                        .property("multicast-group", multicast_ip)
                        .property("port", multicast_port as i32)
                        .property("caps", rtp_caps.clone())
                        .build()?;

                    let depay = gst::ElementFactory::make("rtpopusdepay").build()?;
                    let decode = gst::ElementFactory::make("opusdec").build()?;
                    let convert = gst::ElementFactory::make("audioconvert").build()?;
                    let resample = gst::ElementFactory::make("audioresample").build()?;
                    let volume = gst::ElementFactory::make("volume")
                        .name(format!("volume-{}", channel.get_id()))
                        .property("volume", 1.0)
                        .build()?;

                    pipeline.add_many([&src, &depay, &decode, &convert, &resample, &volume])?;
                    gst::Element::link_many([&src, &depay, &decode, &convert, &resample, &volume])?;

                    volume.link(&mixer)?;

                    volume_handles.push(VolumeHandle::new(&volume));
                }

                let local_src = gst::ElementFactory::make(get_audio_src()).build()?;
                let local_convert = gst::ElementFactory::make("audioconvert").build()?;
                let local_resample = gst::ElementFactory::make("audioresample").build()?;
                let local_volume = gst::ElementFactory::make("volume")
                    .property("volume", 1.0)
                    .property("mute", true)
                    .build()?;
                pipeline.add_many([&local_src, &local_convert, &local_resample, &local_volume])?;
                gst::Element::link_many([
                    &local_src,
                    &local_convert,
                    &local_resample,
                    &local_volume,
                ])?;
                local_volume.link(&mixer)?;

                let sink = gst::ElementFactory::make(get_audio_sink()).build()?;
                sink.set_property("sync", false);

                pipeline.add(&sink)?;
                mixer.link(&sink)?;

                Ok((
                    pipeline,
                    volume_handles.try_into().ok(),
                    Some(VolumeHandle::new(&local_volume)),
                ))
            }
            Self::Transmitter(channel) => {
                let (multicast_ip, multicast_port) = channel.get_multicast();

                let src = gst::ElementFactory::make(get_audio_src()).build()?;
                let convert = gst::ElementFactory::make("audioconvert").build()?;
                let resample = gst::ElementFactory::make("audioresample").build()?;
                let encode = gst::ElementFactory::make("opusenc").build()?;
                let pay = gst::ElementFactory::make("rtpopuspay").build()?;
                let udp_sink = gst::ElementFactory::make("udpsink")
                    .property("host", multicast_ip)
                    .property("port", multicast_port as i32)
                    .property("auto-multicast", true)
                    .build()?;

                pipeline.add_many([&src, &convert, &resample, &encode, &pay, &udp_sink])?;

                gst::Element::link_many([&src, &convert, &resample, &encode, &pay, &udp_sink])?;

                Ok((pipeline, None, None))
            }
        }
    }
}
