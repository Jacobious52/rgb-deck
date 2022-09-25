use embassy_rp::{i2c::{I2c, Async, Instance}, peripherals::I2C0};
use bitvec::prelude::*;

pub struct Keypad {
    i2c: I2c<'static, I2C0, Async>,
    prev: u16,
    current: u16,
}

impl Keypad {
    pub fn new(i2c: I2c<'static, I2C0, Async>) -> Self {
      Self {
        i2c,
        current: 0,
        prev: 0,
      }
    }

    pub async fn update(&mut self) {
        let buttons_buffer: [u8; 2] = Keypad::read_buttons(&mut self.i2c, &[0]).await;

        // bit masking yay. turn 2 u8s into 1 u16
        self.current = !((buttons_buffer[0] as u16) | ((buttons_buffer[1] as u16) << 8));
        let changed = self.current ^ self.prev;
        let _pressed = self.current & changed;
        self.prev = self.current;
    }

    pub fn pressed(&self) -> &BitSlice<u16, Lsb0> {
        self.current.view_bits::<Lsb0>()
    }

    pub fn pressed_u16(&self) -> u16 {
        self.current
    }

    async fn read_buttons<T: Instance, const N: usize, const M: usize>(
        i2c: &mut embassy_rp::i2c::I2c<'_, T, Async>,
        offset: &[u8; N],
    ) -> [u8; M] {
        let button_address = 0x20;
    
        let mut button_buffer = [0u8; M];
        i2c.write_read(button_address, offset, &mut button_buffer).await.unwrap();
    
        button_buffer
    }
}