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
const CHANNEL_BUTTONS: [u8; N_CHANNELS] = [17, 27, 22, 23];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gst::init()?;

    println!("GStreamer version: {}", gst::version_string());

    let picomm_pipeline = PicommPipeline::Receiver(CHANNELS);

    let (pipeline, Some(remote_volume_handles), Some(local_volume_handle)) =
        picomm_pipeline.construct().unwrap()
    else {
        return Err("Failed to construct pipeline".into());
    };

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

        let gpio = rppal::gpio::Gpio::new().unwrap();
        let mut input_pins = CHANNEL_BUTTONS
            .iter()
            .map(|&pin| gpio.get(pin).unwrap().into_input_pullup())
            .collect::<Vec<_>>();

        for (idx, input_pin) in input_pins.iter_mut().enumerate() {
            input_pin.set_interrupt(
                rppal::gpio::Trigger::FallingEdge,
                Some(std::time::Duration::from_millis(1000)),
            );
        }

        let input_pins = input_pins.iter().collect::<Vec<_>>();

        println!("waiting for button presses");
        loop {
            let Some((pin, _)) = gpio.poll_interrupts(&input_pins, false, None).unwrap() else {
                continue;
            };

            println!("Button pressed on pin: {}", pin.pin());

            let channel_idx = input_pins
                .iter()
                .position(|p| p.pin() == pin.pin())
                .expect("Pin not found in input_pins");

            remote_volume_handles[channel_idx].mute();
            local_volume_handle.unmute();

            let channel = CHANNELS[channel_idx];
            let transmission_stream = PicommPipeline::Transmitter(channel);
            let (pipeline, _, _) = transmission_stream.construct().unwrap();

            pipeline.set_state(gst::State::Playing).unwrap();

            println!("Transmitting on channel: {:?}", channel);

            // wait for button to be released
            while pin.is_low() {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            println!("Button released, stopping transmission");

            pipeline.set_state(gst::State::Null).unwrap();
            remote_volume_handles[channel_idx].unmute();
            local_volume_handle.mute();
        }
    }

    pipeline.set_state(gst::State::Null).unwrap();

    Ok(())
}
