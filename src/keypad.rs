// // use embedded_hal_async::i2c::*;

// use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDeviceWithConfig;
// use embassy_sync::{mutex::Mutex, blocking_mutex::raw::ThreadModeRawMutex};
// use static_cell::StaticCell;

// fn hello() {

//     static I2C_BUS: StaticCell<Mutex::<ThreadModeRawMutex, Twim<TWISPI0>>> = StaticCell::new();
// //! let config = twim::Config::default();
// //! let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
// //! let i2c = Twim::new(p.TWISPI0, irq, p.P0_03, p.P0_04, config);
// //! let i2c_bus = Mutex::<ThreadModeRawMutex, _>::new(i2c);
// //! let i2c_bus = I2C_BUS.init(i2c_bus);
// //! 

//     I2cDeviceWithConfig::new(bus, config)
// }
