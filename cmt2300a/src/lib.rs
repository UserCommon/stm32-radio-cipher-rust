#![no_std]
#![no_main]

use defmt::info;
use embassy_stm32::{spi, Peripheral};
use embassy_stm32::{gpio, peripherals};

/// CMT2300A embassy driver for re-using same code for both boards
pub struct CMT2300A<'a, T, Tx, Rx, Cs>
where 
    T: spi::Instance,
    Cs: gpio::Pin
{
    spi: spi::Spi<'a, T, Tx, Rx>,
    csb: gpio::Output<'a, Cs>
}

impl<'a, T, Tx, Rx, Cs> CMT2300A<'a, T, Tx, Rx, Cs>
where 
    T: spi::Instance,
    Cs: gpio::Pin
{
    pub fn new(spi: spi::Spi<'a, T, Tx, Rx>, csb: gpio::Output<'a, Cs>) -> Self {
        Self {
            spi,
            csb
        }
    }

    pub async fn transmit(&mut self, data: &[u8]) -> Result<(), spi::Error>
    where
        Tx: spi::TxDma<T>,
    {
        info!("<$> Transmitting: {:?}", data);
        self.csb.set_low();
        self.spi.write(data).await?;
        self.csb.set_high();
        info!("<$> Transmitted data succesful!");
        Ok(())
    }

    pub async fn recieve(&mut self, buffer: &mut [u8]) -> Result<(), spi::Error>
    where
        Rx: spi::RxDma<T>,
        Tx: spi::TxDma<T>,
    {
        info!("<$> Recieving");
        self.csb.set_low();
        self.spi.read(buffer).await?;
        self.csb.set_high();
        info!("<$> Recieved buffer: {:?} succesful!", buffer);
        Ok(())
    }
}

