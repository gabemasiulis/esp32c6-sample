use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_hal::{
    peripherals::Peripherals,
    // i2c::*,
    // ledc,
    spi,
    spi::{
        // SpiDeviceDriver,
        // SpiDriver,
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
    primitives::{Circle, Line, PrimitiveStyle}
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

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    // esp_idf_svc::log::EspLogger::initialize_default();

    // info!("Hello, world!");
    println!("Hello world!");
    let peripherals = match Peripherals::take() {
        Some(x) => x,
        None => {
            return;
        } 
    };
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
    // display.set_rotation(DisplayRotation::Rotate0);
    // draw_text(&mut display, "Rotate 0!", 0, 0);

    // display.set_rotation(DisplayRotation::Rotate90);
    // draw_text(&mut display, "Rotate 90!", 5, 50);

    // display.set_rotation(DisplayRotation::Rotate180);
    // draw_text(&mut display, "Rotate 180!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate270);
    draw_text(&mut display, "Rotate 270!", 5, 10);

    epd_driver.update_frame(&mut driver, display.buffer(), &mut delay)
        .expect("have updated the frame buffer");
    epd_driver.display_frame(&mut driver, &mut delay)
        .expect("display frame new graphics");

    // Ets::delay_ms(1000);

    draw_text(&mut display, "A second line of text!", 5, 20);
    epd_driver.display_frame(&mut driver, &mut delay)
        .expect("display frame new graphics");

    display.clear(Color::White).ok();

    // draw a analog clock
    let _ = Circle::with_center(Point::new(64, 64), 80)
        .into_styled(PrimitiveStyle::with_stroke(Color::Black, 1))
        .draw(&mut display);
    let _ = Line::new(Point::new(64, 64), Point::new(30, 40))
        .into_styled(PrimitiveStyle::with_stroke(Color::Black, 4))
        .draw(&mut display);
    let _ = Line::new(Point::new(64, 64), Point::new(80, 40))
        .into_styled(PrimitiveStyle::with_stroke(Color::Black, 1))
        .draw(&mut display);

    // draw white on black background
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(Color::White)
        .background_color(Color::Black)
        .build();
    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style("It's working-WoB!", Point::new(90, 10), style, text_style)
        .draw(&mut display);

    // use bigger/different font
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_10X20)
        .text_color(Color::White)
        .background_color(Color::Black)
        .build();

    let _ = Text::with_text_style("It's working\nWoB!", Point::new(90, 40), style, text_style)
        .draw(&mut display);

    // Demonstrating how to use the partial refresh feature of the screen.
    // Real animations can be used.
    epd_driver
        .set_refresh(&mut driver, &mut delay, RefreshLut::Quick)
        .unwrap();
    epd_driver.clear_frame(&mut driver, &mut delay).unwrap();

    // a moving `Hello World!`
    let limit = 10;
    for i in 0..limit {
        draw_text(&mut display, "  Hello World! ", 5 + i * 12, 50);

        epd_driver
            .update_and_display_frame(&mut driver, display.buffer(), &mut delay)
            .expect("display frame new graphics");
        Ets::delay_ms(1000);
    }

    // Show a spinning bar without any delay between frames. Shows how «fast»
    // the screen can refresh for this kind of change (small single character)
    display.clear(Color::White).ok();
    epd_driver
        .update_and_display_frame(&mut driver, display.buffer(), &mut delay)
        .unwrap();

    let spinner = ["|", "/", "-", "\\"];
    for i in 0..10 {
        display.clear(Color::White).ok();
        draw_text(&mut display, spinner[i % spinner.len()], 10, 100);
        epd_driver
            .update_and_display_frame(&mut driver, display.buffer(), &mut delay)
            .unwrap();
    }

    println!("Finished tests - going to sleep");
    epd_driver.sleep(&mut driver, &mut delay).expect("putting display driver to sleep");
    Ets::delay_ms(2000);
    println!("sysloop");
    let sysloop = EspSystemEventLoop::take().expect("initialize the sysloop");
    // let sysloop = match EspSystemEventLoop::take(){
    //     Ok(sysloop) => sysloop,
    //     Err(_) => {return}
    // };

    Ets::delay_ms(2000);
    println!("led change");
    let mut led = rgb_led::WS2812RMT::new(peripherals.pins.gpio8, peripherals.rmt.channel0).unwrap();
    led.set_pixel(rgb_led::RGB8::new(50, 5, 5)).expect("set the led to red");

    Ets::delay_ms(2000);
    println!("load app config");
    let app_config = config::CONFIG;
    println!("SSID: {}, PSK: {}", app_config.wifi_ssid, app_config.wifi_psk);
    Ets::delay_ms(2000);
    println!("initialize wifi");
    let _wifi = wifi::wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
    ).expect("failed to connect");
    Ets::delay_ms(2000);
    println!("initialize client");
    let mut client = create_client().unwrap();
    Ets::delay_ms(2000);
    println!("post request");
    post_test(&mut client, "https://rustembedded.requestcatcher.com/");
    Ets::delay_ms(2000);
    println!("done");
    Ets::delay_ms(2000);
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