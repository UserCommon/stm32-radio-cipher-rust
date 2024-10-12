#![no_std]
#![no_main]


// RECV, BLUE LIGHT BOARD

use core::{convert::Infallible, fmt};

use defmt::{info, unwrap, error};
use embassy_executor::Spawner;
use embassy_stm32::{gpio::{Input, Level, Output, Pull, Speed}, pac::SPI1, peripherals::{self, DMA1_CH3, DMA1_CH4, DMA1_CH5, PB3, PB4, PB5, SPI1, SPI2}};
use embassy_stm32::spi;
use embassy_stm32::interrupt;
use embassy_sync::channel::SendFuture;
use embedded_hal::{self, digital::v2::OutputPin};
use embassy_time::{Delay, Duration, Timer};
use embedded_hal_async::{delay, spi::SpiBus};
use {defmt_rtt as _, panic_probe as _};
use embedded_nrf24l01_async::{Configuration, DataRate, Device, StandbyMode, NRF24L01};
use nrf24_driver::*;
// type SPI = spi::Spi<'static, SPI1, DMA1_CH5, DMA1_CH4;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Peripherals
    let config = embassy_stm32::Config::default();
    let p = embassy_stm32::init(config);

    // LED TASK
    let led = Output::new(p.PC13, Level::Low, Speed::Medium);
    // unwrap!(spawner.spawn(blinker(led, Duration::from_millis(1000))));

    // SPI
    let spi_config = spi::Config::default();
    let spi = spi::Spi::new(
        p.SPI1,           // SPI peripheral
        p.PB3,            // SCK pin on B3
        p.PB5,            // MOSI pin on B5
        p.PB4,            // MISO pin on B4
        p.DMA1_CH3,       // TX DMA
        p.DMA1_CH2,       // RX DMA
        spi_config,       // SPI configuration
    );
    // GPIO for CE (Chip Enable)
    let mut ce = Output::new(p.PA11, Level::Low, Speed::Medium);  // CE pin on PA11

    // GPIO for CSN (Chip Select Not)
    let mut csn = Output::new(p.PA12, Level::High, Speed::Medium); // CSN pin on PA12

    // GPIO for IRQ (Interrupt)
    let irq_pin = Input::new(p.PA15, Pull::None);  // IRQ pin on PA15

    // nrf
    // Wrap the SPI
    let mut nrf = Nrf24l01Bulider::default()
        .spi(spi)
        .csn(csn)
        .ce(ce)
        .timer(embassy_time::Delay)
        .rx_address(b"00000")
        .tx_address(b"00000")
        .channel(5)
        .payload_size(8)
        .build().await.unwrap();
    unwrap!(spawner.spawn(recv_packages(nrf, led)));
}

#[embassy_executor::task]
async fn recv_packages(mut nrf: Nrf24l01<impl SpiBus<u8> + 'static,
                                       impl OutputPin + 'static,
                                       impl OutputPin + 'static,
                                       impl delay::DelayNs + 'static>,
                        mut led: Output<'static, peripherals::PC13>) {
    led.set_low();
    loop {
        // if let Ok(false) = nrf.is_sending().await {
            if let Ok(true) = nrf.data_ready().await {
                let mut buff = [1u8; 8];
                if let Ok(()) = nrf.receive(&mut buff).await {
                    info!("read: {:?}", buff);
                }
                Timer::after_secs(1).await;

                // if let Ok(()) = nrf.send(&buff).await {
                //     info!("send!");
                // }
            }
        // }
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

