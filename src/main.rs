#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use bitvec::prelude::*;
use bitvec::view::BitView;
use embassy_executor::Spawner;
use embassy_rp::gpio::Level;
use embassy_rp::gpio::Output;
use embassy_rp::i2c::Async;
use embassy_rp::i2c::Instance;
use embassy_rp::spi::Spi;
use embassy_time::{Duration, Timer};

use rand::rngs::SmallRng;
use rand::RngCore;
use rand::SeedableRng;
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

    let mut i2c = embassy_rp::i2c::I2c::new(p.I2C0, p.PIN_5, p.PIN_4, p.DMA_CH3, p.DMA_CH4, config);

    let mut small_rng = SmallRng::seed_from_u64(42);

    let mut lit_led = rand(&mut small_rng, NUM_LEDS);

    let mut prev = 0;

    loop {
        rgb.update().await;
        Timer::after(Duration::from_millis(20)).await;

        rgb.set_color(lit_led, 20, 20, 20);

        lit_led = rand(&mut small_rng, NUM_LEDS);

        rgb.set_color(
            lit_led,
            rand(&mut small_rng, 255).try_into().unwrap(),
            rand(&mut small_rng, 255).try_into().unwrap(),
            rand(&mut small_rng, 255).try_into().unwrap(),
        );

        let buttons_buffer: [u8; 2] = read_button(&mut i2c, &[0]).await;

        // bit masking yay. turn 2 u8s into 1 u16
        let current = !((buttons_buffer[0] as u16) | ((buttons_buffer[1] as u16) << 8));
        let changed = current ^ prev;
        let _pressed = current & changed;
        prev = current;

        let bits = current.view_bits::<Lsb0>();
        for (i, bit) in bits.iter().enumerate() {
            if *bit {
                rgb.set_color(i, 200, 0, 0);
            } else {
                rgb.set_color(i, 0, 0, 200);
            }
        }
    }
}

async fn read_button<T: Instance, const N: usize, const M: usize>(
    i2c: &mut embassy_rp::i2c::I2c<'_, T, Async>,
    offset: &[u8; N],
) -> [u8; M] {
    let button_address = 0x20;

    let mut button_buffer = [0u8; M];
    i2c.write_read(button_address, offset, &mut button_buffer).await.unwrap();

    button_buffer
}

fn rand(small_rng: &mut SmallRng, n: usize) -> usize {
    (small_rng.next_u64() % (n as u64)) as usize
}
