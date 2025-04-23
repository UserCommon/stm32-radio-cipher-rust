#![no_std]
#![no_main]


// RECV, BLUE LIGHT BOARD

use secure_radio::core::default_ciphers::MagmaHamming;
use secure_radio::core::GeneralCipher;


use embassy_time::with_timeout;
use defmt::{error, info, println, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::peripherals::{self, DMA1_CH4, DMA1_CH5, DMA1_CH6, DMA1_CH7, PA10, PA2, PA3, PA9, PC13, USART1, USART2};
use embassy_stm32::usart::{self, Config, Uart, UartRx, UartTx};
use embedded_hal::{self, digital::v2::OutputPin};
use embassy_time::{Delay, Duration, Timer};
use embassy_stm32::gpio::{Level, Speed, Output};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<USART1>;
    USART2 => usart::InterruptHandler<USART2>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Peripherals

    // LED TASK
    let config = embassy_stm32::Config::default();
    let p = embassy_stm32::init(config);
    let mut led = Output::new(p.PC13, Level::Low, Speed::Medium);
    led.set_high();

    let (uart1tx, uart1rx) = Uart::new(
        p.USART1,                   // Переферийный объект
        p.PA10,              // Пин приема данных (RX)
        p.PA9,               // Пин передачи данных (TX)
        Irqs,       // IRQ обработчик
        p.DMA1_CH4,                 // DMA для передачи данных
        p.DMA1_CH5,                 // DMA для приема данных
        Config::default(),          // Конфигурация по умолчанию
    ).unwrap().split();

    let (uart2tx, uart2rx) = Uart::new(
        p.USART2,          // Периферийный объект для USART2
        p.PA3,             // RX (прием)
        p.PA2,             // TX (передача)
        Irqs,              // Прерывания
        p.DMA1_CH7,        // DMA канал для TX
        p.DMA1_CH6,        // DMA канал для RX
        Config::default(), // Конфигурация по умолчанию
    ).unwrap().split();

    unwrap!(spawner.spawn(send_data(uart1tx, uart2rx, Duration::from_millis(100))));
    unwrap!(spawner.spawn(recv_data(uart1rx, uart2tx, led, Duration::from_millis(100))));
}

#[embassy_executor::task]
async fn send_data(
    mut uart1tx: UartTx<'static, USART1, DMA1_CH4>,
    mut uart2rx: UartRx<'static, USART2, DMA1_CH6>,
    interval: Duration,
)
{
    let encoder = MagmaHamming::default();
    // Blink led if sent
    
    let mut rcvd = [0u8;8];
    loop {
        // let n = read_uart_line(&mut uart2, &mut rcvd).await;
        uart2rx.read(&mut rcvd).await.unwrap();
        let line = core::str::from_utf8(&rcvd[..8]).unwrap_or("<bad utf8>");
        println!("{:?}", line);
        // led.set_low();
        let data: u64 = u64::from_be_bytes(rcvd);
        let data = encoder.general_encrypt(data).unwrap();
        uart1tx.write(&data).await.unwrap();

        // led.set_high();
        println!("sent: {:?}", data);
        Timer::after(interval).await;
    }
    //let _ = uart.read(&mut buffer).await.unwrap();
}

#[embassy_executor::task]
async fn recv_data(
    mut uart1rx: UartRx<'static, USART1, DMA1_CH5>,
    mut uart2tx: UartTx<'static, USART2, DMA1_CH7>,
    mut led: Output<'static, PC13>,
    interval: Duration,
)
{
    let decoder = MagmaHamming::default();
    let mut buffer = [0u8; 16];
    loop {
        led.set_low();
        // unwrap need to be handled
        match uart1rx
                    .read(&mut buffer)
                    .await {
            Ok(_) => {},
            Err(_) => {Timer::after(interval).await; continue;},
        }
        println!("read: {:?}", buffer);
        let data = decoder.general_decrypt(buffer).unwrap();
        let bytes: [u8; 8] = data.to_be_bytes();
        let s = core::str::from_utf8(&bytes).unwrap_or("<bad utf8>");
        println!("decoded: {:?}", s);
        uart2tx.write(&buffer).await.unwrap();
        led.set_high();
        buffer = [0u8; 16];

        Timer::after(interval).await;
    }
    //let _ = uart.read(&mut buffer).await.unwrap();
}

pub async fn read_uart_line<const N: usize>(
    rx: &mut Uart<'_, USART2, DMA1_CH7, DMA1_CH6>,
    buf: &mut [u8; N],
) -> usize {
    let mut i = 0;

    loop {
        let mut byte = [0u8; 1];

        let read_result = with_timeout(Duration::from_millis(100), rx.read(&mut byte)).await;

        match read_result {
            Ok(Ok(())) => {
                let b = byte[0];

                if b == b'\r' || b == b'\n' {
                    break;
                }

                if i < N {
                    buf[i] = b;
                    i += 1;
                } else {
                    break; // Переполнение
                }
            }
            _ => {
                // таймаут или ошибка — выходим
                break;
            }
        }
    }

    // Заполняем остаток нулями
    for j in i..N {
        buf[j] = 0;
    }

    i
}



async fn read_by_chunks(buf: &[u8; 256]) -> [u64; 4] {
    let mut result = [0u64; 4];

    for i in 0..4 {
        let start = i * 8;
        let end = start + 8;

        // Берем чанк и копируем его в массив из 8 байт
        let mut chunk = [0u8; 8];
        chunk.copy_from_slice(&buf[start..end]);

        result[i] = u64::from_be_bytes(chunk);
    }

    result
}
