mod adc_external;
pub use adc_external::*;
mod adc_internal;
pub use adc_internal::*;
mod battery_level;
pub use battery_level::*;
mod dynamic_gpio;
pub use dynamic_gpio::*;
mod i2c1;
use embedded_hal::blocking::i2c;
pub use i2c1::*;
mod i2c2;
pub use i2c2::*;
mod oscillator_control;
pub use oscillator_control::*;
mod power;
pub use power::*;
mod rgb_led;
pub use rgb_led::*;
mod serial;
pub use serial::*;
mod usb;
pub use usb::*;

pub fn build(
    pins: crate::pins::Pins,
    cr: &mut crate::pins::GpioCr,
) -> (
    ExternalAdcPins,
    InternalAdcPins,
    BatteryLevelPins,
    DynamicGpioPins,
    I2c1Pins,
    I2c2Pins,
    OscillatorControlPins,
    PowerPins,
    RgbLedPin,
    SerialPins,
    UsbPins,
) {
    let external_adc =
        ExternalAdcPins::build(pins.enable_external_adc, pins.external_adc_reset, cr);

    let internal_adc = InternalAdcPins::build(
        pins.enable_avdd,
        pins.internal_adc1,
        pins.internal_adc2,
        pins.internal_adc3,
        pins.internal_adc4,
        pins.internal_adc5,
        cr,
    );

    let battery_level = BatteryLevelPins::build(pins.vin_measure, pins.enable_vin_measure, cr);

    let dynamic_gpio = DynamicGpioPins::build(
        pins.gpio1, pins.gpio2, pins.gpio3, pins.gpio4, pins.gpio5, pins.gpio6, pins.gpio7,
        pins.gpio8, cr,
    );

    let i2c1 = I2c1Pins::build(pins.i2c1_scl, pins.i2c1_sda, cr);

    let i2c2 = I2c2Pins::build(pins.i2c2_scl, pins.i2c2_sda, cr);

    let oscillator_control = OscillatorControlPins::build(pins.enable_hse, cr);

    let power = PowerPins::build(pins.enable_3v, pins.enable_5v, cr);

    let rgb_led = RgbLedPin::build(
        pins.rgb_red_and_wake_button,
        pins.rgb_green_and_spi2_chip_select,
        pins.rgb_blue,
        cr,
    );

    let serial = SerialPins::build(pins.tx, pins.rx, cr);

    let usb = UsbPins::build(pins.usb_p, pins.usb_n, cr);

    return (
        external_adc,
        internal_adc,
        battery_level,
        dynamic_gpio,
        i2c1,
        i2c2,
        oscillator_control,
        power,
        rgb_led,
        serial,
        usb,
    );
}
