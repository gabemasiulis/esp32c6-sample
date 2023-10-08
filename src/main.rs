use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_hal::{
    peripherals::Peripherals,
    // i2c::*,
    // ledc,
    spi::{self, SpiDeviceDriver},
    spi::{
        // SpiDeviceDriver,
        SpiDriver,
        SpiDriverConfig,
    },
    delay::Ets,
    delay::Delay,
    prelude::*,
    gpio::PinDriver,
    gpio
};
use epd_waveshare::{
    color::*,
    // epd7in5_v2::{Epd7in5, WIDTH, HEIGHT},
    epd2in13_v2::{
        // Display2in13,
        Epd2in13, Display2in13
    },
    // epd2in13bc::{Display2in13bc, Epd2in13bc},
    // graphics::DisplayRotation,
    prelude::*,
};

use esp_idf_svc::{
    // nvs::EspDefaultNvsPartition,
    http::client::{Configuration, EspHttpConnection},
    
};
use embedded_graphics::{
    mono_font::{
        // ascii::FONT_6X10,
        MonoTextStyleBuilder
    },
    // pixelcolor::{self, BinaryColor, },
    prelude::*,
    text::{
        Baseline,
        Text,
        TextStyleBuilder
    },
};

use embedded_svc::http::client::Client as HttpClient;
use embedded_svc::http::Method;

// use esp_idf_svc::nvs::EspDefaultNvsPartition;
mod rgb_led;

use esp_idf_sys as _;

mod wifi;
mod config;

fn create_client() -> anyhow::Result<HttpClient<EspHttpConnection>> {
    let config = Configuration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    };

    Ok(HttpClient::wrap(EspHttpConnection::new(&config)?))
}

fn post_test(client: &mut HttpClient<EspHttpConnection>, url: &str) {
    let headers = [("content-type", "application/json")];
    let mut req = client.request(Method::Post, &url, &headers).unwrap();
    println!("request initiated");
    req.write(&"Hello World!".as_bytes()).expect("Send a hello world post body");
    println!("request appended maybe?");
    let res = req.submit().unwrap();
    println!("Request submitted");
    let status = res.status();
    println!("Response status: {status}");

}
type SPI = SpiDeviceDriver<
    'static,
    SpiDriver<'static>>;

type EpdDriver = Epd2in13<
    SPI,
    PinDriver<'static, gpio::Gpio4, gpio::Output>,
    PinDriver<'static, gpio::Gpio9, gpio::Input>,
    PinDriver<'static, gpio::Gpio5, gpio::Output>,
    PinDriver<'static, gpio::Gpio6, gpio::Output>,
    Delay>;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    // esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = match Peripherals::take() {
        Some(x) => x,
        None => {
            return;
        } 
    };
    let mut led = rgb_led::WS2812RMT::new(peripherals.pins.gpio8, peripherals.rmt.channel0).unwrap();
    led.set_pixel(rgb_led::RGB8::new(20, 20, 0)).expect("set the led to yellow");
    let spi_p = peripherals.spi2;
    let sclk = peripherals.pins.gpio20; // SCK
    let sdo = peripherals.pins.gpio18; // MOSI
    // let sdi = peripherals.pins.gpio19; // MISO
    let cs = PinDriver::output(peripherals.pins.gpio4)
        .expect("initialize gpio4 as an output pin"); // ECS
    let busy = PinDriver::input(peripherals.pins.gpio9)
        .expect("initialize gpio9 as an input pin"); // BUSY
    let dc = PinDriver::output(peripherals.pins.gpio5)
        .expect("initialize gpio5 as an output pin"); // D/C
    let rst = PinDriver::output(peripherals.pins.gpio6)
        .expect("initialize gpio6 as an output pin"); // RST
    let bus_config = SpiDriverConfig::new();
    let baudrate = Hertz(10_000_000);
    let driver_config = spi::config::Config::new()
        .baudrate(baudrate);
    let mut driver = spi::SpiDeviceDriver::new_single(
        spi_p,
        sclk,
        sdo,
        Option::<gpio::AnyIOPin>::None,
        Option::<gpio::AnyOutputPin>::None,
        &bus_config,
        &driver_config
    ).expect("have created the spi driver");

    let mut delay = Delay{};

    let mut epd_driver = Epd2in13::new(
        &mut driver,
        cs,
        busy,
        dc,
        rst,
        &mut delay,
        None
    ).unwrap();
    let mut display = Display2in13::default();

    display.set_rotation(DisplayRotation::Rotate270);

    // Demonstrating how to use the partial refresh feature of the screen.
    // Real animations can be used.
    epd_driver
        .set_refresh(&mut driver, &mut delay, RefreshLut::Quick)
        .unwrap();
    epd_driver.clear_frame(&mut driver, &mut delay).unwrap();

    let mut line_index: i32 = 0;

    let message = ["Never", "Gonna", "Give", "You", "Up", "Never", "Gonna", "Let", "You", "Down", "Never", "Gonna", "Run", "Around", "You", "And", "Desert", "You"];
    for word in message {
        (line_index, epd_driver, driver) = append_log(word, line_index, &mut display, epd_driver, driver, delay);
    }
    
    (line_index, epd_driver, driver) = append_log("Finished tests - going to sleep", line_index, &mut display, epd_driver, driver, delay);
    led.set_pixel(rgb_led::RGB8::new(0, 50, 0)).expect("set the led to green");
    
    Ets::delay_ms(2000);
    led.set_pixel(rgb_led::RGB8::new(0, 0, 0)).expect("set the led to red");
    println!("sysloop");
    let sysloop = EspSystemEventLoop::take().expect("initialize the sysloop");
    // let sysloop = match EspSystemEventLoop::take(){
    //     Ok(sysloop) => sysloop,
    //     Err(_) => {return}
    // };

    Ets::delay_ms(2000);
    
    led.set_pixel(rgb_led::RGB8::new(50, 5, 5)).expect("set the led to red");

    // Ets::delay_ms(2000);
    // println!("load app config");
    // let app_config = config::CONFIG;
    // println!("SSID: {}, PSK: {}", app_config.wifi_ssid, app_config.wifi_psk);
    // Ets::delay_ms(2000);
    // println!("initialize wifi");
    // let _wifi = wifi::wifi(
    //     app_config.wifi_ssid,
    //     app_config.wifi_psk,
    //     peripherals.modem,
    //     sysloop,
    // ).expect("failed to connect");
    // Ets::delay_ms(2000);
    // println!("initialize client");
    // let mut client = create_client().unwrap();
    // Ets::delay_ms(2000);
    // println!("post request");
    // post_test(&mut client, "https://rustembedded.requestcatcher.com/");
    // Ets::delay_ms(2000);
    // println!("done");
    // Ets::delay_ms(2000);
    epd_driver.sleep(&mut driver, &mut delay).expect("putting display driver to sleep");
}

fn append_log(new_message: &str, mut line_index: i32, mut display: &mut Display2in13, mut epd_driver: EpdDriver, mut driver: SPI, mut delay:Delay) -> (i32, EpdDriver, SPI) {
    if line_index == 0 {
        // clear the screen, gently?
        display.clear(Color::Black).ok();
    }
    let input_length = new_message.len();
    let num_spaces = 41 - input_length;
    let whitespace = " ";

    let new_message_with_whitespace = new_message.to_owned() + whitespace.repeat(num_spaces).as_str();

    // do a partial screen refresh of a single line of code
    draw_text(&mut display, new_message_with_whitespace.as_str(), 5, 10 + (line_index * 10));
    line_index = line_index + 1;
    epd_driver
        .update_and_display_frame(&mut driver, display.buffer(), &mut delay)
        .expect("display frame new graphics");
    if line_index == 10 {
        line_index = 0;
    }
    (line_index, epd_driver, driver)
} 

fn draw_text(display: &mut Display2in13, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(Color::White)
        .background_color(Color::Black)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}