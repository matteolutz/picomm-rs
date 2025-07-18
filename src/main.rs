use gstreamer::{
    self as gst,
    glib::object::ObjectExt,
    prelude::{ElementExt, GstBinExt},
};

use crate::{channel::Channel, pipeline::PicommPipeline};

mod channel;
mod pipeline;
mod volume;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gst::init()?;

    #[cfg(feature = "rpi")]
    {
        std::thread::spawn(|| {
            let mut pin = rppal::gpio::Gpio::new()
                .unwrap()
                .get(23)
                .unwrap()
                .into_output();

            loop {
                pin.toggle();
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });
    }

    println!("GStreamer version: {}", gst::version_string());

    let picomm_pipeline = PicommPipeline::Receiver([
        Channel::ChannelBroadcast,
        Channel::Channel1,
        Channel::Channel2,
        Channel::Channel3,
    ]);

    let pipeline = picomm_pipeline.construct()?;
    println!(
        "volume 0: {}",
        pipeline
            .by_name("volume-0")
            .unwrap()
            .property::<f64>("volume")
    );

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
