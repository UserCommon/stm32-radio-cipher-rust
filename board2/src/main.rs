#![no_std]
#![no_main]


use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::{gpio::{Input, Level, Output, Pull, Speed}, peripherals::{self, DMA1_CH1, DMA1_CH2, DMA1_CH3, DMA1_CH4, DMA1_CH5, PA3, PA4, PB0, PB3, PB4, PB5, SPI1}};
use embassy_stm32::spi;
use embassy_time::{Timer, Duration};
use {defmt_rtt as _, panic_probe as _};

use cmt2300a::CMT2300A;

type CMT = CMT2300A<'static, SPI1, DMA1_CH3, DMA1_CH2, PA4>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let config = embassy_stm32::Config::default();
    let p = embassy_stm32::init(config);
    let led = Output::new(p.PC13, Level::Low, Speed::Medium);

    // SPI 
    let spi_config = spi::Config::default();
    // Initialize GPIO pins
    let csb = Output::new(p.PA4, Level::High, Speed::Medium); // CSB pin
    // let fcsb = Output::new(p.PA3, Level::High, Speed::Medium); // FCSB pin
    // let irq = Input::new(p.PB0, Pull::Up); // IRQ pin (interrupt)

    // Initialize SPI pins
    let spi = spi::Spi::new_rxonly(
        p.SPI1,           // SPI peripheral
        p.PA5,            // SCK
        p.PA6,            // MISO
        p.DMA1_CH3,      // Rx DMA channel
        p.DMA1_CH2,      // Tx DMA channel (if needed, otherwise can be removed)
        spi_config
    );

    let reciever = CMT2300A::new(spi, csb);
    unwrap!(spawner.spawn(blinker(led, Duration::from_millis(1000))));
    unwrap!(spawner.spawn(rx(reciever)));
}

#[embassy_executor::task]
async fn rx(
    mut reciever: CMT
) {
    let buffer: &mut [u8; 6] = &mut [0, 0, 0, 0, 0, 0];

    loop {
        let res = reciever.recieve(&mut buffer[..]).await;
        match res {
            Ok(_) => {
                info!(
                    "<***> Succesfully recieved data:\nRaw: {:?}\nAs str: {:?}",
                    buffer,
                    core::str::from_utf8(buffer).map_err(|_| spi::Error::Overrun)
                );
            }
            Err(e) => {
                info!(
                    "<***> Error while recieving data: {:?}",
                    e
                );
            }
        }
    }
}


#[embassy_executor::task]
async fn blinker(mut led: Output<'static, peripherals::PC13>, interval: Duration) {
    loop {
        led.set_high();
        Timer::after(interval).await;
        led.set_low();
        Timer::after(interval).await;
    }
}

