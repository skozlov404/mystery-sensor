use anyhow::Result;
use esp32_nimble::enums::ConnMode;
use esp32_nimble::enums::DiscMode;
use esp32_nimble::utilities::BleUuid;
use esp32_nimble::BLEAdvertisementData;
use esp32_nimble::BLEDevice;
use esp_idf_svc::hal::adc::attenuation::DB_11;
use esp_idf_svc::hal::adc::attenuation::DB_6;
use esp_idf_svc::hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::adc::oneshot::AdcDriver;
use esp_idf_svc::hal::adc::oneshot::AdcChannelDriver;
use esp_idf_svc::hal::prelude::Peripherals;

const VBAT_FULL: u16 = 1400;
const VBAT_EMPTY: u16 = 1100;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let mut led = PinDriver::output(peripherals.pins.gpio15)?;
    led.set_high()?;

    let adc = AdcDriver::new(peripherals.adc1)?;
    let sensor_adc_config = AdcChannelConfig{
        attenuation: DB_11,
        calibration: true,
        ..Default::default()
    };
    let mut adc_pin0 = AdcChannelDriver::new(&adc, peripherals.pins.gpio0, &sensor_adc_config)?;
    let battery_adc_config = AdcChannelConfig{
        attenuation: DB_6,
        calibration: true,
        ..Default::default()
    };
    let mut adc_pin1 = AdcChannelDriver::new(&adc, peripherals.pins.gpio1, &battery_adc_config)?;

    let ble = BLEDevice::take();
    let ble_advertiser = ble.get_advertising();

    ble_advertiser.lock().advertisement_type(ConnMode::Non).disc_mode(DiscMode::Gen).scan_response(false);
    ble_advertiser.lock().start()?;

    loop {
        led.set_low()?;
        let vbat = adc_pin1.read()?;
        let sensor_data = adc_pin0.read()?;
        let sensor_data_raw = adc_pin0.read_raw()?;
        log::info!("vbat: {vbat}, read: {sensor_data}, read_raw: {sensor_data_raw}");
        
        // Initialize BLE data
        let mut ble_ad_packet = Vec::<u8>::with_capacity(6); 
        ble_ad_packet.push(0x40); // header

        // Battery data
        let bat_percentage: u8 = ((vbat - VBAT_EMPTY)*100/(VBAT_FULL - VBAT_EMPTY)) as u8;
        ble_ad_packet.push(0x01); // battery percentage
        ble_ad_packet.push(bat_percentage.to_le());

        // Sensor data
        ble_ad_packet.push(0x0E); // pm10 sensor
        ble_ad_packet.extend(sensor_data.to_le_bytes());

        let mut ble_advertisement_data = BLEAdvertisementData::new();
        ble_advertisement_data.name("MisterySensor");
        ble_advertisement_data.service_data(BleUuid::from_uuid16(0xFCD2), &ble_ad_packet);
        ble_advertiser.lock().set_data(&mut ble_advertisement_data).unwrap();
        
        led.set_high()?;
        FreeRtos::delay_ms(1000);
    }
}
