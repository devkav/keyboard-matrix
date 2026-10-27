#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{Peripherals, bind_interrupts, gpio::{Input, Level, Output, Pull}, peripherals::USB, usb::{Driver, InterruptHandler}};
use embassy_usb::{Builder, Config, class::hid::{self, HidReaderWriter, State}};
use heapless::String;
use embassy_time::Timer;
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};
use {defmt_rtt as _, panic_probe as _};
use defmt::info;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

const KEYMAP: [[u8; 2]; 2] = [
    [0x04, 0x05],
    [0x06, 0x07]
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: Peripherals = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);

    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Devan");
    config.product = Some("Keyboard");
    config.serial_number = Some("0");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control = [0; 64];
    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control
    );

    let hid_config = hid::Config {
        report_descriptor: KeyboardReport::desc(),
        request_handler: None,
        poll_ms: 1,
        max_packet_size: 8,
        hid_subclass: hid::HidSubclass::Boot,
        hid_boot_protocol: hid::HidBootProtocol::Keyboard
    };

    let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, &mut state, hid_config);
    let (read, mut writer) = hid.split();

    let mut usb = builder.build();
    let usb_future = usb.run();

    let mut rows = [
        Output::new(p.PIN_2, Level::Low),
        Output::new(p.PIN_3, Level::Low),
    ];

    let cols = [
        Input::new(p.PIN_4, Pull::Down),
        Input::new(p.PIN_5, Pull::Down),
    ];

    loop {
        let mut output: String<10> = String::new();

        for (row_index, row_pin) in rows.iter_mut().enumerate() {
            row_pin.set_high();
            Timer::after_micros(10).await; // let the column rise before reading

            for (col_index, col_pin) in cols.iter().enumerate() {
                if col_pin.is_high() {
                    output.push('X').ok();
                    info!("{}, {}", row_index, col_index);
                } else {
                    output.push(' ').ok();
                }
            }

            if row_index == 0 {
                output.push('\n').ok();
            }

            row_pin.set_low();
            Timer::after_micros(10).await; // let the column drain before the next row
        }
        
        // info!("\n{}", output.as_str());

        Timer::after_millis(5).await;
    }
}
