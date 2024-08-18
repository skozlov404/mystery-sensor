use anyhow::Result;
use esp_idf_svc::hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_svc::hal::delay;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::adc::oneshot::AdcDriver;
use esp_idf_svc::hal::adc::oneshot::AdcChannelDriver;
use esp_idf_svc::hal::prelude::Peripherals;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let mut led = PinDriver::output(peripherals.pins.gpio15)?;

    let adc1 = AdcDriver::new(peripherals.adc1)?;

    let channel_config = AdcChannelConfig{
        calibration: true,
        ..Default::default()
    };
    let mut adc_pin0 = AdcChannelDriver::new(adc1, peripherals.pins.gpio0, &channel_config)?;

    let delay = delay::Delay::new_default();

    loop {
        led.set_low()?;
        log::info!("read: {}, read_raw: {}", adc_pin0.read()?, adc_pin0.read_raw()?);
        led.set_high()?;
        delay.delay_ms(200);
    }
}
