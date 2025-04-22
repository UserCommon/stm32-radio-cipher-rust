#![no_std]
#![no_main]


// RECV, BLUE LIGHT BOARD

use secure_radio::core::default_ciphers::MagmaHamming;
use secure_radio::core::GeneralCipher;

use core::{convert::Infallible, fmt};

use defmt::{error, info, println, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::peripherals::{self, DMA1_CH5, DMA1_CH4, PA10, PA9, USART1};
use embassy_stm32::usart::{self, Config, Uart};
use embedded_hal::{self, digital::v2::OutputPin};
use embassy_time::{Delay, Duration, Timer};
use embassy_stm32::gpio::{Level, Speed, Output};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<USART1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Peripherals

    // LED TASK
    // let led = Output::new(p.PC13, Level::Low, Speed::Medium);
    unwrap!(spawner.spawn(send_data(Duration::from_millis(1000))));
}



#[embassy_executor::task]
async fn send_data(interval: Duration)
{
    let encoder = MagmaHamming::default();
    let config = embassy_stm32::Config::default();
    let p = embassy_stm32::init(config);
    // Blink led if sent
    let mut led = Output::new(p.PC13, Level::Low, Speed::Medium);

    let mut uart = Uart::new(
        p.USART1,                   // Переферийный объект
        p.PA10,              // Пин приема данных (RX)
        p.PA9,               // Пин передачи данных (TX)
        Irqs,       // IRQ обработчик
        p.DMA1_CH4,                 // DMA для передачи данных
        p.DMA1_CH5,                 // DMA для приема данных
        Config::default(),          // Конфигурация по умолчанию
    ).unwrap();

    loop {
        led.set_low();
        let data: u64 = 131;
        let data = encoder.general_encrypt(data).unwrap();
        uart.write(&data).await.unwrap();

        led.set_high();
        println!("sent: {:?}", data);
        // let mut buffer = [0u8; 128];
        Timer::after(interval).await;
    }
    //let _ = uart.read(&mut buffer).await.unwrap();
}