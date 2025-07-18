use ssd1306::mode::DisplayConfig;

pub fn setup_oled() {
    let i2c = rppal::i2c::I2c::new().unwrap();
    let interface = ssd1306::I2CDisplayInterface::new(i2c);
    let mut display = ssd1306::Ssd1306::new(
        interface,
        ssd1306::size::DisplaySize128x32,
        ssd1306::rotation::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    display.init().unwrap();

    let text_style = embedded_graphics::mono_font::MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_7X14_BOLD)
        .text_color(embedded_graphics::pixelcolor::BinaryColor::On)
        .build();

    embedded_graphics::text::Text::with_baseline(
        "piComm",
        embedded_graphics::prelude::Point::zero(),
        text_style,
        embedded_graphics::text::Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    display.flush().unwrap();
}
