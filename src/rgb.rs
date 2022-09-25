use embassy_rp::{
    gpio::Output,
    peripherals::{PIN_17, SPI0},
    spi::{Async, Spi},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};

const RGB_WIDTH: usize = 4;
const RGB_HIGHT: usize = 4;
const LED_LEN: usize = 4;
pub const NUM_LEDS: usize = RGB_HIGHT * RGB_WIDTH;
const BUF_PADDING: usize = 8;
const BUF_LEN: usize = (NUM_LEDS * LED_LEN) + BUF_PADDING;
const BUF_RGB_OFFSET: usize = 4;

pub struct Rgb {
    spi: Spi<'static, SPI0, Async>,
    cs: Output<'static, PIN_17>,
    buf: [u8; BUF_LEN],
}

impl Rgb {
    pub fn new(spi: Spi<'static, SPI0, Async>, cs: Output<'static, PIN_17>) -> Self {
        let buf = [0_u8; BUF_LEN];

        Self { spi, cs, buf }
    }

    pub fn set_brightness(&mut self, brightness: f32) {
        if !(0.0..=1.0).contains(&brightness) {
            return;
        }

        for i in 0..NUM_LEDS {
            self.buf[BUF_RGB_OFFSET + (i * LED_LEN)] =
                0b11100000 | ((brightness * (0b11111 as f32)) as u8);
        }
    }

    pub fn set_color(&mut self, i: usize, r: u8, g: u8, b: u8) {
        if i >= NUM_LEDS {
            return;
        }

        let offset = i * LED_LEN;
        self.buf[BUF_RGB_OFFSET + (offset + 1)] = b;
        self.buf[BUF_RGB_OFFSET + (offset + 2)] = g;
        self.buf[BUF_RGB_OFFSET + (offset + 3)] = r;
    }

    pub async fn update(&mut self) {
        self.cs.set_low();
        self.spi.write(&self.buf).await.unwrap();
        self.cs.set_high();
    }
}

#[embassy_executor::task]
pub async fn run(rgb: &'static Mutex<ThreadModeRawMutex, Rgb>) {
    loop {
        Timer::after(Duration::from_millis(20)).await;

        rgb.lock().await.update().await;
    }
}
