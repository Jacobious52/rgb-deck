#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_rp::gpio::Level;
use embassy_rp::gpio::Output;
use embassy_rp::interrupt;
use embassy_rp::spi::Spi;
use embassy_rp::usb::Driver;
use embassy_sync::channel::Channel;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};

use rand::rngs::SmallRng;
use rand::RngCore;
use rand::SeedableRng;
use rgb_deck::keypad;
use rgb_deck::keypad::Keypad;
use rgb_deck::rgb;
use rgb_deck::rgb::Rgb;
use rgb_deck::usb;
use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut _small_rng = SmallRng::seed_from_u64(42);

    let p = embassy_rp::init(Default::default());

    // rgb setup

    let miso = p.PIN_16;
    let mosi = p.PIN_19;
    let clk = p.PIN_18;

    let spi = Spi::new(
        p.SPI0,
        clk,
        mosi,
        miso,
        p.DMA_CH0,
        p.DMA_CH1,
        embassy_rp::spi::Config::default(),
    );
    let cs = Output::new(p.PIN_17, Level::High);

    static RGB: StaticCell<Mutex<ThreadModeRawMutex, Rgb>> = StaticCell::new();
    let rgb = Mutex::new(Rgb::new(spi, cs));
    let rgb: &'static mut _ = RGB.init_with(|| rgb);
    rgb.lock().await.set_brightness(0.5);

    // keypad setup

    let mut config = embassy_rp::i2c::Config::default();
    config.frequency = 400_000;

    let i2c = embassy_rp::i2c::I2c::new(p.I2C0, p.PIN_5, p.PIN_4, p.DMA_CH3, p.DMA_CH4, config);
    static KEYPAD: StaticCell<Mutex<ThreadModeRawMutex, Keypad>> = StaticCell::new();
    let keypad = Mutex::new(Keypad::new(i2c));
    let keypad: &'static mut _ = KEYPAD.init_with( || keypad);

    // usb setup

    let irq = interrupt::take!(USBCTRL_IRQ);
    let driver = Driver::new(p.USB, irq);

    static USB_RECV_CHANNEL: Channel<ThreadModeRawMutex, [u8; 64], 1> = Channel::new();
    let usb = usb::Usb::new(driver);

    spawner.must_spawn(usb::run(usb, &USB_RECV_CHANNEL));
    spawner.must_spawn(keypad::run(keypad));
    spawner.must_spawn(rgb::run(rgb));

    loop {
        if let Ok(usb_buf) = USB_RECV_CHANNEL.try_recv() {
            // pass
        }
        let keypad = keypad.lock().await;
        let mut rgb = rgb.lock().await;

        for (i, pressed) in keypad.is_down().iter().enumerate() {
            if *pressed {
                rgb.set_color(i, 200, 0, 0);
            } else {
                rgb.set_color(i, 0, 0, 200);
            }
        }
    }
}

fn rand(small_rng: &mut SmallRng, n: usize) -> usize {
    (small_rng.next_u64() % (n as u64)) as usize
}
