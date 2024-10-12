#![no_std]
#![no_main]
//TODO! critical section
pub const MIRF_CONFIG: u8 = (1 << 3) | (0 << 2); // 0 << 2 PRIM RX if 1 then prx
pub const MIRF_ADDR_LEN: u8 = 5; // Adress length TODO! move to builder

use embedded_hal::digital::v2::OutputPin;
use embedded_hal_async::delay;
use embedded_hal_async::spi::{self, ErrorType, Operation, SpiBus, SpiDevice};
use core::default::Default;

mod memory;
mod mnemonic;
mod command;
mod error;

use error::*;

/// ## NRF24L01_Builder
/// Is a builder for *NRF24L01* struct for convient 
/// configuration. spi, csn and ce fields are required!
/// 
/// Default channel = 1
/// 
/// Default payload size = 32d
pub struct Nrf24l01Bulider<SPI, CSN, CE, D>
where
    SPI: spi::SpiBus,
    CSN: OutputPin,
    CE: OutputPin,
    D: delay::DelayNs // maybe there exist another way idk
{
    spi: Option<SPI>,
    csn: Option<CSN>,
    ce:  Option<CE>,
    timer: Option<D>, 

    rx_address: Option<&'static [u8]>,
    tx_address: Option<&'static [u8]>,
    channel: Option<u8>,
    payload_size: Option<u8>,
}

impl<SPI, CSN, CE, D> Default for Nrf24l01Bulider<SPI, CSN, CE, D>
where
    SPI: spi::SpiBus,
    CSN: OutputPin,
    CE: OutputPin,
    D: delay::DelayNs
{
    fn default() -> Self {
        Self {
            spi: None,
            csn: None,
            ce: None,
            timer: None,

            rx_address: None,
            tx_address: None,
            channel: None,
            payload_size: None
        }
    }
}

impl<SPI, CSN, CE, D> Nrf24l01Bulider<SPI, CSN, CE, D>
where
    SPI: spi::SpiBus,
    CSN: OutputPin,
    CE: OutputPin,
    D: delay::DelayNs
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spi(mut self, spi: SPI) -> Self {
        self.spi = Some(spi);
        self
    }

    pub fn csn(mut self, csn: CSN) -> Self {
        self.csn = Some(csn);
        self
    }

    pub fn ce(mut self, ce: CE) -> Self {
        self.ce = Some(ce);
        self
    }

    pub fn channel(mut self, channel: u8) -> Self {
        self.channel = Some(channel);
        self
    }

    pub fn payload_size(mut self, payload_size: u8) -> Self {
        self.payload_size = Some(payload_size);
        self
    }

    pub fn timer(mut self, timer: D) -> Self {
        self.timer = Some(timer);
        self
    }
    
    pub fn rx_address(mut self, addr: &'static [u8]) -> Self {
        self.rx_address = Some(addr);
        self
    }

    pub fn tx_address(mut self, addr: &'static [u8]) -> Self {
        self.tx_address = Some(addr);
        self
    }

    pub async fn build(self) -> Result<Nrf24l01<SPI, CSN, CE, D>, BuilderError> {
        let mut nrf = Nrf24l01 {
            spi: self.spi.ok_or(BuilderError::SpiNotSpecified)?,
            csn: self.csn.ok_or(BuilderError::CsnNotSpecified)?,
            ce: self.ce.ok_or(BuilderError::CeNotSpecified)?,
            timer: self.timer.ok_or(BuilderError::TimerNotSpecified)?,
            channel: self.channel.unwrap_or(1),
            payload_size: self.payload_size.unwrap_or(32),
            tx_power_status: false
        };

        if let Some(raddr) = self.rx_address {
            nrf.set_rxaddr(raddr).await
        } else {
            nrf.set_rxaddr(&[0, 0, 0, 0, 0]).await
        }.map_err(|_| BuilderError::Other)?;

        if let Some(taddr) = self.tx_address {
            nrf.set_txaddr(taddr).await
        } else {
            nrf.set_txaddr(&[0, 0, 0, 0, 0]).await
        }.map_err(|_| BuilderError::Other)?;

        nrf.config().await.map_err(|_| BuilderError::Other)?;
        Ok(nrf)
    }
}


pub struct Nrf24l01<SPI, CSN, CE, D>
where
    SPI: spi::SpiBus,
    CSN: OutputPin,
    CE: OutputPin,
    D: delay::DelayNs
{
    spi: SPI,
    csn: CSN,
    ce:  CE,
    timer: D,

    channel: u8,
    payload_size: u8,
    tx_power_status: bool
}

impl<SPI, CSN, CE, D> Nrf24l01<SPI, CSN, CE, D>
where
    SPI: spi::SpiBus,
    CSN: OutputPin,
    CE: OutputPin,
    D: delay::DelayNs
{
    /// Create new nrf24l01 instance
    /// It is recommended to use Nrf24l01Builder though
    pub fn new(
        spi: SPI,
        csn: CSN,
        ce:  CE,
        timer: D,
        channel: u8,
        payload_size: u8,
    ) -> Result<Self, Error> {
        let mut nrf = Nrf24l01 { 
            spi,
            csn,
            ce,
            timer,
            channel,
            payload_size,
            tx_power_status: false
        };

        nrf.ce.set_high().map_err(|_| Error::Gpio)?;
        nrf.csn.set_high().map_err(|_| Error::Gpio)?;

        Ok(nrf)
    }

    pub async fn read_register(&mut self, register: u8) -> Result<u8, Error> {
        self
            .write(&[command::R_REGISTER | (command::REGISTER_MASK & register)])
            .await
            .map_err(|_| Error::IOError("write error"))?;

        let mut buffer = [0];

        self
            .read(&mut buffer)
            .await
            .map_err(|_| Error::IOError("read error"))?;

        Ok(buffer[0])
    }

    pub async fn write_register(&mut self, register: u8, value: &[u8]) -> Result<(), Error> {
        self
            .write(&[command::W_REGISTER | (command::REGISTER_MASK & register)])
            .await
            .map_err(|_| Error::IOError("write error"))?;
        
        self
            .write(value)
            .await
            .map_err(|_| Error::IOError("write error"))?;

        Ok(())
    }

    async fn config_register(&mut self, register: u8, value: &u8) -> Result<(), Error> {
        self
            .write(&[command::W_REGISTER | (command::REGISTER_MASK & register)])
            .await
            .map_err(|_| Error::IOError("write error"))?;
        
        self
            .write(&[*value])
            .await
            .map_err(|_| Error::IOError("write error"))?;

        Ok(())
    }

    pub async fn config(&mut self) -> Result<(), Error> {
        let (channel, ps) = (self.channel, self.payload_size);
        self.config_register(memory::RF_CH, &channel).await?;
        self.config_register(memory::RX_PW_P0, &ps).await?;
        self.config_register(memory::RX_PW_P1, &ps).await?;

        self.power_up_rx().await?;
        self.flush_rx().await?;
        Ok(())
    }

    pub async fn power_down(&mut self) -> Result<(), Error> {
        self.ce.set_low().map_err(|_| Error::Gpio)?;
        self
            .config_register(memory::CONFIG, &MIRF_CONFIG)
            .await?;
        // todo
        Ok(())
    }

    async fn power_up_rx(&mut self) -> Result<(), Error> {
        self.tx_power_status = false;
        self.config_register(
            memory::CONFIG,
            &(MIRF_CONFIG | ((1 << mnemonic::PWR_UP) | (1 << mnemonic::PRIM_RX)))
        ).await?;

        Ok(())
    }

    async fn power_up_tx(&mut self) -> Result<(), Error> {
        self.tx_power_status = true;
        self.config_register(
            memory::CONFIG,
            &(MIRF_CONFIG | ((1 << mnemonic::PWR_UP) | (0 << mnemonic::PRIM_RX))),
        ).await.map_err(|_| Error::ConfigError)?;

        Ok(())
    }

    pub async fn flush_rx(&mut self) -> Result<(), Error> {
        self.write(&[command::FLUSH_RX]).await.map_err(|_| Error::Spi("write err"))?;
        Ok(())
    }

    pub async fn flush_interrupts(&mut self) -> Result<(), Error>{
        const STATUS_REGISTER: u8 = 0x07;
        const RX_DR: u8 = 1 << 6;  // Данные готовы
        const TX_DS: u8 = 1 << 5;  // Передача завершена
        const MAX_RT: u8 = 1 << 4; // Достигнут максимум повторов

        self.write_register(STATUS_REGISTER, &[RX_DR | TX_DS | MAX_RT]).await?;
        Ok(())
    }

    pub async fn set_rxaddr(&mut self, addr: &[u8]) -> Result<(), Error> {
        self.write_register(memory::RX_ADDR_P1, addr).await?;
        Ok(())
    }

    pub async fn set_txaddr(&mut self, addr: &[u8]) -> Result<(), Error> {
        self.write_register(memory::RX_ADDR_P0, addr).await?;
        self.write_register(memory::TX_ADDR, addr).await?;
        Ok(())
    }

    pub async fn get_status(&mut self) -> Result<u8, Error> {
        let response = self.read_register(memory::STATUS).await?;
        Ok(response)
    }

    pub async fn send(&mut self, data: &[u8]) -> Result<(), Error> {
        // this looks like a shid + camed
        let _ = self.get_status().await?; // I'm not entirely sure why, but Mirf does this, so we do as well.
        while self.tx_power_status {
            let status = self.get_status().await?;
            if (status & ((1 << mnemonic::TX_DS) | (1 << mnemonic::MAX_RT))) != 0 {
                self.tx_power_status = false;
                break;
            }
        }

        self.ce.set_low().map_err(|_| Error::Gpio)?;
        self.power_up_tx().await?;

        self
            .write(&[command::FLUSH_TX])
            .await
            .map_err(|_| Error::IOError("write error"))?;

        self
            .write(&[command::W_TX_PAYLOAD])
            .await
            .map_err(|_| Error::IOError("write error"))?;
        
        self
            .write(data)
            .await
            .map_err(|_| Error::IOError("write error"))?;

        self.ce.set_high().map_err(|_| Error::Gpio)?;
        Ok(())
    }

    pub async fn is_sending(&mut self) -> Result<bool, Error> {
        if self.tx_power_status {
            let status = self.get_status().await?;
            if (status & ((1 << mnemonic::TX_DS) | (1 << mnemonic::MAX_RT))) != 0 {
                self.power_up_rx().await?;
                return Ok(false);
            }

            return Ok(true);
        }
        Ok(false)
    }

    pub async fn data_ready(&mut self) -> Result<bool, Error> {
        let status = self.get_status().await?;
        if (status & (1 << mnemonic::RX_DR)) != 0 {
            return Ok(true);
        }
        let fifo_empty = self.rx_fifo_empty().await?;
        Ok(!fifo_empty)
    }

    async fn rx_fifo_empty(&mut self) -> Result<bool, Error> {
        let fifo_status = self.read_register(memory::FIFO_STATUS).await?;
        if fifo_status & (1 << mnemonic::RX_EMPTY) != 0 {
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn receive(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        self.write(&[command::R_RX_PAYLOAD]).await?;
        self.read(buf).await?;
        self.config_register(memory::STATUS, &(1 << mnemonic::RX_DR)).await?;
        Ok(())
    }
}

impl<SPI, CSN, CE, D> ErrorType for Nrf24l01<SPI, CSN, CE, D>
where
    SPI: spi::SpiBus,
    CSN: OutputPin,
    CE: OutputPin,
    D: delay::DelayNs
{
    type Error = Error;
}

impl<SPI, CSN, CE, D> SpiDevice for Nrf24l01<SPI, CSN, CE, D>
where
    SPI: spi::SpiBus,
    CSN: OutputPin,
    CE: OutputPin,
    D: delay::DelayNs
{    
    async fn transaction(
        &mut self,
        operations: &mut [Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        // Idk how to lock the bus
        self.csn.set_low().map_err(|_| Error::Gpio)?;
        use Operation::*;
        
        // TODO! redo this garbage string in spi error handling
        for operation in operations {
            match operation {
                Read(r) => self.spi.read(r).await.map_err(|_| Error::Spi("read error"))?,
                Write(w) => self.spi.write(w).await.map_err(|_| Error::Spi("write error"))?,
                Transfer(r, w) => self.spi.transfer(r, w).await.map_err(|_| Error::Spi("transfer error"))?,
                TransferInPlace(r) => self.spi.transfer_in_place(r).await.map_err(|_| Error::Spi("transfer in place error"))?,
                DelayNs(t) => self.timer.delay_ns(*t).await
            }
        }

        self.spi.flush().await.map_err(|_| Error::Spi("flush error"))?;
        self.csn.set_high().map_err(|_| Error::Gpio)?;
        Ok(())
    }
}