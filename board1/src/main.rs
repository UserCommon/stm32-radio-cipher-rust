#![no_std]
#![no_main]


use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::{gpio::{Input, Level, Output, Pull, Speed}, peripherals};
use embassy_stm32::spi;
use embassy_time::{Timer, Duration};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let config = embassy_stm32::Config::default();
    let p = embassy_stm32::init(config);
    let led = Output::new(p.PC13, Level::Low, Speed::Medium);
    unwrap!(spawner.spawn(blinker(led, Duration::from_millis(1000))));

    // SPI 
    let spi_config = spi::Config::default();
    let mut spi = spi::Spi::new_txonly(
        p.SPI2,         // SPI peripheral
        p.PB13,         // SCK pin
        p.PB15,         // MOSI pin
        p.DMA1_CH4,     // TX DMA channel
        p.DMA1_CH3,     // RX DMA channel (not used, will be removed in the future)
        spi_config
    ); // PB13 (SCLK), PB15 (SDIO)

     // Configure the Chip Select (CS) pin on PB4
    let mut cs = Output::new(p.PB4, Level::High, Speed::Medium);

    // Configure FCSB pin on PB5 as output
    let mut fcsb = Output::new(p.PB5, Level::High, Speed::Medium);

    // Configure NIQR pin on PB3 as input (interrupt)
    let irq = Input::new(p.PB3, Pull::Up);

    // Reset the module
    cs.set_low();
    embassy_time::Timer::after(Duration::from_millis(10)).await;
    cs.set_high();

    // Prepare data for SPI transfer (e.g., read/write CMT2300A registers)
    let mut read_buf: [u16; 2] = [0x00; 2]; // Buffer for reading data
    let write_buf: [u16; 2] = [0xAA, 0xBB]; // Data to be written

    // Perform SPI transfer
    cs.set_low(); // Select the device
    let result = spi.blocking_transfer(&mut read_buf, &write_buf);
    cs.set_high(); // Deselect the device

    match result {
        Ok(_) => {
            // Handle successful transfer
            defmt::info!("SPI transfer successful: Read = {:?}", read_buf);
        }
        Err(e) => {
            // Handle SPI transfer error
            defmt::error!("SPI transfer error: {:?}", e);
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

