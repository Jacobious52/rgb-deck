use embassy_futures::join::join;
use embassy_rp::{
    peripherals::USB,
    usb::{Driver, Instance},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_usb::{driver::EndpointError, Builder, Config};
use embassy_usb_serial::{CdcAcmClass, State};

pub struct Usb {
    device_descriptor: [u8; 256],
    config_descriptor: [u8; 256],
    bos_descriptor: [u8; 256],
    control_buf: [u8; 64],

    config: Config<'static>,
    driver: Option<Driver<'static, USB>>,
}

impl Usb {
    pub fn new(driver: Driver<'static, USB>) -> Usb {
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
        let device_descriptor = [0; 256];
        let config_descriptor = [0; 256];
        let bos_descriptor = [0; 256];
        let control_buf = [0; 64];

        Self {
            device_descriptor,
            config_descriptor,
            bos_descriptor,
            control_buf,

            config,
            driver: Some(driver),
        }
    }
}

#[embassy_executor::task]
pub async fn run(usb: Usb, chan: &'static Channel<ThreadModeRawMutex, [u8; 64], 1>) {
    let mut usb = usb;

    let driver = usb.driver.take().unwrap();

    let mut state = State::new();
    let mut builder = Builder::new(
        driver,
        usb.config,
        &mut usb.device_descriptor,
        &mut usb.config_descriptor,
        &mut usb.bos_descriptor,
        &mut usb.control_buf,
        None,
    );

    // Create classes on the builder.
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);

    let send_recv_fut = async {
        loop {
            class.wait_connection().await;
            let _ = echo(&mut class, &chan).await;
        }
    };

    // Build the builder.
    let mut device = builder.build();

    let usb_fut = device.run();

    join(usb_fut, send_recv_fut).await;
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
    recv: &Channel<ThreadModeRawMutex, [u8; 64], 1>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        //let data = &buf[..n];
        if let Err(e) = recv.try_send(buf) {
            
        }

        //class.write_packet(data).await?;
    }
}
