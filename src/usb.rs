use embassy_futures::join::join;
use embassy_rp::{
    peripherals::USB,
    usb::{Driver, Instance},
};
use embassy_sync::{channel::Channel, blocking_mutex::raw::ThreadModeRawMutex};
use embassy_usb::{driver::EndpointError, Builder, Config};
use embassy_usb_serial::{CdcAcmClass, State};

use crate::UsbChan;

#[embassy_executor::task]
pub async fn run(driver: Driver<'static, USB>, usb_send_chan: UsbChan, usb_recv_chan: UsbChan) {
    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-serial example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut control_buf,
        None,
    );

    // Create classes on the builder.
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Do stuff with the class!
    let echo_fut = async {
        loop {
            class.wait_connection().await;
            let _ = echo(&mut class, &usb_send_chan, &usb_recv_chan).await;
        }
    };

    join(usb_fut, echo_fut).await;
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo<'d, T: Instance + 'd>(
    class: &mut CdcAcmClass<'d, Driver<'d, T>>,
    usb_send_chan: UsbChan,
    usb_recv_chan: UsbChan,
) -> Result<(), Disconnected> {
    let mut in_buf = [0; 64];
    loop {
        let in_n = class.read_packet(&mut in_buf).await?;
        let _ = usb_recv_chan.try_send((in_n, in_buf));

        if let Ok((out_n, out_buf)) = usb_send_chan.try_recv() {
            let data = &out_buf[..out_n];
            class.write_packet(data).await?;
        }
    }
}
