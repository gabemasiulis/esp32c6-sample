use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    // esp_idf_svc::log::EspLogger::initialize_default();

    // info!("Hello, world!");
    let peripherals = Peripherals::take().unwrap();
    let mut led = PinDriver::output(peripherals.pins.gpio4).unwrap();
    let mut onboard_led = PinDriver::output(peripherals.pins.gpio8).unwrap();
    onboard_led.set_high().unwrap();
    println!("Hello world!");
    loop {
        led.set_high().unwrap(); // .set_high();
        // println!("Set High");
        // we are sleeping here to make sure the watchdog isn't triggered
        FreeRtos::delay_ms(2000);

        led.set_low().unwrap();
        // println!("Set Low");
        FreeRtos::delay_ms(2000);
    }
}
// use std::thread;
// use std::time::Duration;
// use esp_idf_sys as _;
// use embedded_hal::digital::blocking::OutputPin;
// use esp_idf_hal::peripherals::Peripherals;

// fn main() {
//   esp_idf_sys::link_patches();

//   let peripherals = Peripherals::take().unwrap();
//   let mut led = peripherals.pins.gpio5.into_output().unwrap();
//   let n = 1;

//   while n == 1 {
//     led.set_high().unwrap();
//     thread::sleep(Duration::from_millis(1000));

//     led.set_low().unwrap();
//     thread::sleep(Duration::from_millis(1000));

//     println!("blink");
//   }
// }