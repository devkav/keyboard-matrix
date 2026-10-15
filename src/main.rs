#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{Peripherals, gpio::{Input, Level, Output, Pull}};
use heapless::String;
// use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use defmt::info;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: Peripherals = embassy_rp::init(Default::default());

    let mut rows = [
        Output::new(p.PIN_2, Level::Low),
        Output::new(p.PIN_3, Level::Low),
    ];

    let cols = [
        Input::new(p.PIN_4, Pull::Down),
        Input::new(p.PIN_5, Pull::Down),
    ];

    loop {
        let mut output: String<5> = String::new();

        for (row_index, row_pin) in rows.iter_mut().enumerate() {
            row_pin.set_high();

            for col_pin in cols.iter() {
                if col_pin.is_high() {
                    output.push('X').ok();
                } else {
                    output.push(' ').ok();
                }
            }

            if row_index == 0 {
                output.push('\n').ok();
            }

            row_pin.set_low();
        }
        
        info!("\n{}", output.as_str());
    }
}
