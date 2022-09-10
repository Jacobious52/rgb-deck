use embassy_rp::{
    gpio::{Level, Output},
    peripherals::{PIN_17, SPI0},
    spi::{Async, Config, Spi},
    Peripherals,
};

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
    pub fn new(p: Peripherals) -> Self {
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
            Config::default(),
        );

        let cs = Output::new(p.PIN_17, Level::High);
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
