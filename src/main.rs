#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output, Pin};
use embassy_rp::spi::{Async, Config, Instance, Spi};
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let miso = p.PIN_16;
    let mosi = p.PIN_19;
    let clk = p.PIN_18;

    let mut spi = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, Config::default());

    let mut cs = Output::new(p.PIN_17, Level::High);

    let mut tx_buf = [0_u8; 72];
    set_brightness(0.5, &mut tx_buf);

    set_color(3, 25, 100, 0, &mut tx_buf);

    let mut rand = Instant::now().as_ticks() % 16;

    loop {
        update(&mut spi, &mut cs, &mut tx_buf).await;
        Timer::after(Duration::from_secs(1)).await;
        set_color(rand as usize, 20, 20, 20, &mut tx_buf);
        rand = Instant::now().as_ticks() % 16;
        set_color(rand as usize, 30, 200, 30, &mut tx_buf);
    }
}

fn set_color(i: usize, r: u8, g: u8, b: u8, tx_buf: &mut [u8]) {
    if i >= 16 {
        return;
    }

    let offset = i * 4;
    tx_buf[4 + (offset + 1)] = b;
    tx_buf[4 + (offset + 2)] = g;
    tx_buf[4 + (offset + 3)] = r;
}

fn set_brightness(brightness: f32, tx_buf: &mut [u8]) {
    if brightness < 0.0 || brightness > 1.0 {
        return;
    }

    for i in 0..16 {
        tx_buf[4 + (i * 4)] = 0b11100000 | ((brightness * (0b11111 as f32)) as u8);
    }
}

async fn update<'a, T: Instance, P: Pin>(spi: &mut Spi<'a, T, Async>, cs: &mut Output<'a, P>, tx_buf: &mut [u8]) {
    cs.set_low();
    spi.write(tx_buf).await.unwrap();
    cs.set_high();
}
