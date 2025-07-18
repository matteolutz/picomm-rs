use gstreamer::{
    self as gst,
    glib::object::ObjectExt,
    prelude::{ElementExt, GstBinExt},
};

use crate::{channel::Channel, pipeline::PicommPipeline};

mod channel;
mod pipeline;

#[cfg(feature = "rpi")]
mod rpi;

mod volume;

const N_CHANNELS: usize = 4;
const CHANNELS: [Channel; N_CHANNELS] = [
    Channel::ChannelBroadcast,
    Channel::Channel1,
    Channel::Channel2,
    Channel::Channel3,
];
const CHANNEL_BUTTONS: [u32; N_CHANNELS] = [17, 27, 22, 23];

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

    std::thread::spawn(|| {
        let picomm_pipeline = PicommPipeline::Receiver(CHANNELS);

        let pipeline = picomm_pipeline.construct().unwrap();
        println!(
            "volume 0: {}",
            pipeline
                .by_name("volume-0")
                .unwrap()
                .property::<f64>("volume")
        );

        pipeline.set_state(gst::State::Playing).unwrap();

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

        pipeline.set_state(gst::State::Null).unwrap();
    });

    #[cfg(feature = "rpi")]
    {}

    #[cfg(not(feature = "rpi"))]
    {
        loop {}
    }
}
