#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_rp::gpio::Level;
use embassy_rp::gpio::Output;
use embassy_rp::spi::Spi;
use embassy_time::{Duration, Timer};

use rand::rngs::SmallRng;
use rand::RngCore;
use rand::SeedableRng;
use rgb_deck::Keypad;
use rgb_deck::{Rgb, NUM_LEDS};

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

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

    let mut rgb = Rgb::new(spi, cs);
    rgb.set_brightness(0.5);

    let mut config = embassy_rp::i2c::Config::default();
    config.frequency = 400_000;

    let i2c = embassy_rp::i2c::I2c::new(p.I2C0, p.PIN_5, p.PIN_4, p.DMA_CH3, p.DMA_CH4, config);
    let mut keypad = Keypad::new(i2c);

    let mut _small_rng = SmallRng::seed_from_u64(42);

    loop {
        Timer::after(Duration::from_millis(20)).await;

        rgb.update().await;
        keypad.update().await;
    
        
        for (i, pressed) in keypad.pressed().iter().enumerate() {
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
