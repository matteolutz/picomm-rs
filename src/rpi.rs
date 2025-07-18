fn setup_oled() {
    let mut i2c = rppal::i2c::I2c::new().unwrap();
    let interface = ssd1306::I2CDisplayInterface::new(i2c);
}
