#![no_std]
#![no_main]


// RECV, BLUE LIGHT BOARD

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::{gpio::{Input, Level, Output, Pull, Speed}, peripherals::{self, DMA1_CH3, DMA1_CH4, DMA1_CH5, PB3, PB4, PB5, SPI2}};
use embassy_stm32::spi;
use embassy_sync::channel::SendFuture;
use embassy_time::{Delay, Duration, Timer};
use {defmt_rtt as _, panic_probe as _};
use cmt2300a::CMT2300A;


type SPI = spi::Spi<'static, SPI2, DMA1_CH5, DMA1_CH3>;
type CMT = CMT2300A<'static, SPI2, DMA1_CH5, DMA1_CH3, PB4>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let config = embassy_stm32::Config::default();
    let p = embassy_stm32::init(config);
    let led = Output::new(p.PC13, Level::Low, Speed::Medium);
    unwrap!(spawner.spawn(blinker(led, Duration::from_millis(1000))));

    // SPI 
    let spi_config = spi::Config::default();
    let spi = spi::Spi::new_txonly(
        p.SPI2,         // SPI peripheral
        p.PB13,         // SCK pin
        p.PB15,         // MOSI pin
        p.DMA1_CH5,     // TX DMA channel
        p.DMA1_CH3,     // RX DMA channel (not used, will be removed in the future)
        spi_config
    ); // PB13 (SCLK), PB15 (SDIO)
     // Configure the Chip Select (CS) pin on PB4
    let csb = Output::new(p.PB4, Level::High, Speed::Medium);
    // Configure FCSB pin on PB5 as output
    // let mut fcsb = Output::new(p.PB5, Level::Low, Speed::Medium);
    // Configure NIQR pin on PB3 as input (interrupt)
    // let irq = Input::new(p.PB3, Pull::Up);
    // unwrap!(spawner.spawn(tx(spi, cs, irq)));
    let transmitter = CMT2300A::new(spi, csb);
    unwrap!(spawner.spawn(tx(transmitter)));

}

#[embassy_executor::task]
async fn tx(
    mut transmitter: CMT
) {
    let buf: &[u8; 6] = b"Hello!";
    loop {
        Timer::after_millis(10).await;
        let res = transmitter.transmit(buf).await;
        if let Err(err) = res {
            info!("Failed to tranmit data: {:?}", err);
        } else {
            info!("Successfully tranmitted data!");
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

// // Configure CMT2300A for RX mode
// async fn configure_cmt2300a(spi: &mut SPI, delay: &mut Delay) {
//     // Configure for RX
//     cmt2300a_write_register(spi, 0x20, 0x10).await;
//     delay.delay_ms(1).await;
// }

// async fn cmt2300a_receive_data(spi: &mut SPI) -> [u8; 16] {
//     let mut buffer = [0u8; 16];
//     spi.blocking_transfer_in_place(&mut buffer).await.unwrap();
//     buffer
// }

// // SPI write helper
// async fn cmt2300a_write_register(spi: &mut SPI, address: u8, value: u8) {
//     let mut buffer = [address, value];
//     spi.blocking_transfer_in_place(&mut buffer).await.unwrap();
// }

