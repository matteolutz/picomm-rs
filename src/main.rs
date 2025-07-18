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

    let picomm_pipeline = PicommPipeline::Receiver(CHANNELS);

    let (pipeline, _volume_handles) = picomm_pipeline.construct().unwrap();

    pipeline.set_state(gst::State::Playing).unwrap();

    #[cfg(not(feature = "rpi"))]
    {
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
    }

    #[cfg(feature = "rpi")]
    {
        rpi::setup_oled();
        let mut input_pins = CHANNEL_BUTTONS
            .iter()
            .map(|&pin| {
                rppal::gpio::Gpio::new()
                    .unwrap()
                    .get(pin)
                    .unwrap()
                    .into_input_pullup()
            })
            .collect::<Vec<_>>();

        for (idx, input_pin) in input_pins.iter().enumerate() {
            input_pin.set_async_interrupt(
                rppal::gpio::Trigger::FallingEdge,
                Some(std::time::Duration::from_millis(1000)),
                |_| {
                    let channel = CHANNELS[idx];
                    let transmission_stream = PicommPipeline::Transmitter(channel);
                    let (pipeline, _) = transmission_stream.construct().unwrap();

                    pipeline.set_state(gst::State::Playing).unwrap();

                    println!("Transmitting on channel: {:?}", channel);

                    // wait for button to be released
                    while input_pin.is_low() {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }

                    pipeline.set_state(gst::State::Null).unwrap();
                },
            );
        }

        loop {}
    }

    pipeline.set_state(gst::State::Null).unwrap();

    Ok(())
}
