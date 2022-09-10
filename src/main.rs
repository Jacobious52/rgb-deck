#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use rand::rngs::SmallRng;
use rand::RngCore;
use rand::SeedableRng;
use rgb_deck::{Rgb, NUM_LEDS};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut rgb = Rgb::new(p);
    rgb.set_brightness(0.5);

    let mut small_rng = SmallRng::seed_from_u64(42);

    let mut lit_led = rand(&mut small_rng, NUM_LEDS);
    loop {
        rgb.update().await;
        Timer::after(Duration::from_millis(200)).await;

        rgb.set_color(lit_led, 20, 20, 20);

        lit_led = rand(&mut small_rng, NUM_LEDS);

        rgb.set_color(
            lit_led,
            rand(&mut small_rng, 255).try_into().unwrap(),
            rand(&mut small_rng, 255).try_into().unwrap(),
            rand(&mut small_rng, 255).try_into().unwrap(),
        );
    }
}

fn rand(small_rng: &mut SmallRng, n: usize) -> usize {
    (small_rng.next_u64() % (n as u64)) as usize
}
