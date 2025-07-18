use gstreamer::{
    self as gst,
    prelude::{ElementExt, GstBinExtManual},
};

mod channel;
mod pipeline;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gst::init()?;

    println!("GStreamer version: {}", gst::version_string());

    let pipeline = gst::Pipeline::new();

    let rtp_caps = gst::Caps::builder("application/x-rtp")
        .field("clock-rate", 48000)
        .field("media", "audio")
        .field("encoding-name", "OPUS")
        .build();

    let src = gst::ElementFactory::make("udpsrc")
        .property("multicast-group", "239.255.0.3")
        .property("port", 5006)
        .property("caps", rtp_caps)
        .build()?;

    let depay = gst::ElementFactory::make("rtpopusdepay").build()?;
    let decode = gst::ElementFactory::make("opusdec").build()?;
    let convert = gst::ElementFactory::make("audioconvert").build()?;
    let resample = gst::ElementFactory::make("audioresample").build()?;
    let queue = gst::ElementFactory::make("queue").build()?;
    let sink = gst::ElementFactory::make("autoaudiosink").build()?;

    pipeline.add_many([&src, &depay, &decode, &convert, &resample, &queue, &sink])?;
    gst::Element::link_many([&src, &depay, &decode, &convert, &resample, &queue, &sink])?;

    // src.link(&depay)?;

    /*
    let depay_weak = depay.downgrade();
    src.connect_pad_added(move |_, src_pad| {
        println!("new src pad triggered");

        let Some(depay) = depay_weak.upgrade() else {
            eprintln!("Failed to upgrade weak reference to depay element");
            return;
        };

        let Some(pad) = depay.static_pad("sink") else {
            eprintln!("Failed to request sink pad from depay element");
            return;
        };

        if pad.is_linked() {
            eprintln!("Pad is already linked, skipping link operation");
            return;
        }

        if let Err(err) = src_pad.link(&pad) {
            eprintln!("Failed to link source pad: {}", err);
        }
    });
    */

    pipeline.set_state(gst::State::Playing)?;

    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        match msg.view() {
            gst::MessageView::Eos(..) => break,
            gst::MessageView::Error(err) => {
                eprintln!("Error: {:?}", err.debug());
                break;
            }
            _ => (),
        }
    }

    pipeline.set_state(gst::State::Null)?;

    Ok(())
}
