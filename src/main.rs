use anyhow::Result;
use esp32_nimble::enums::ConnMode;
use esp32_nimble::enums::DiscMode;
use esp32_nimble::utilities::BleUuid;
use esp32_nimble::BLEAdvertisementData;
use esp32_nimble::BLEDevice;
use esp_idf_svc::hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_svc::hal::delay::FreeRtos;
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

    let ble = BLEDevice::take();
    let ble_advertiser = ble.get_advertising();

    ble_advertiser.lock().advertisement_type(ConnMode::Non).disc_mode(DiscMode::Gen).scan_response(false);
    ble_advertiser.lock().start()?;

    loop {
        led.set_low()?;
        let sensor_data = adc_pin0.read()?;
        let sensor_data_raw = adc_pin0.read_raw()?;
        log::info!("read: {}, read_raw: {}", sensor_data, sensor_data_raw);
        
        let mut ble_ad_packet = Vec::<u8>::from([
            0x40, // Header
            0x0E, // pm10 sensor
        ]);
        ble_ad_packet.extend(sensor_data_raw.to_le_bytes());

        let mut ble_advertisement_data = BLEAdvertisementData::new();
        ble_advertisement_data.name("MisterySensor");
        ble_advertisement_data.service_data(BleUuid::from_uuid16(0xFCD2), &ble_ad_packet);
        ble_advertiser.lock().set_data(&mut ble_advertisement_data).unwrap();
        
        led.set_high()?;
        FreeRtos::delay_ms(1000);
    }
}
