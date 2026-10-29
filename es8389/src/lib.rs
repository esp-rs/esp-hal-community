//! `no_std` async driver for the Everest Semiconductor ES8389 audio codec.
//!
//! The codec runs as an I2S slave: the controller drives BCLK and LRCK, and optionally MCLK. The
//! register sequences follow Espressif's `esp_codec_dev` ES8389 driver.
//!
//! ```no_run
//! use embedded_hal_async::i2c::I2c;
//! use es8389::{Clock, Config, Error, Es8389, Format, MicGain, WordLength};
//!
//! async fn start<I: I2c>(i2c: I) -> Result<(), Error<I::Error>> {
//!     let mut codec = Es8389::new(i2c, es8389::DEFAULT_ADDRESS);
//!     // 16-bit samples in 32-bit slots: BCLK = 2 x 32 x 48 kHz.
//!     codec
//!         .init(Config {
//!             clock: Clock::Bclk(3_072_000),
//!             sample_rate: 48_000,
//!             format: Format::I2s,
//!             word_length: WordLength::Bits16,
//!             dac_reference: false,
//!         })
//!         .await?;
//!     codec.set_mic_gain(MicGain::Db30_5).await?;
//!     Ok(())
//! }
//! ```

#![no_std]
#![warn(missing_docs)]

mod coefficients;

use core::fmt;

use embedded_hal_async::{delay::DelayNs, i2c::I2c};

use crate::coefficients::{COEFFICIENTS, REGISTERS};

/// The 7-bit I2C address that `esp_codec_dev` uses (`ES8389_CODEC_DEFAULT_ADDR` is the 8-bit form,
/// 0x20).
pub const DEFAULT_ADDRESS: u8 = 0x10;

/// DAC volume register value for 0 dB.
pub const VOLUME_0DB: u8 = 0xBF;

/// The clock the codec derives its internal clocks from, with its frequency in Hz.
///
/// The frequency must be one of these multiples of the sample rate:
///
/// | Sample rate | Clock frequency / sample rate |
/// |---|---|
/// | 8 kHz | 32, 36, 48, 50, 64, 72, 96, 128, 192, 256, 288, 384, 512, 600, 768, 1024, 1500, 1536, 1625, 2048, 2304, 2400, 3000, 3072, 3250 |
/// | 16 kHz | 32, 36, 48, 50, 64, 72, 96, 128, 144, 192, 256, 300, 384, 512, 750, 768, 1024, 1152, 1200, 1500, 1536, 1625 |
/// | 24 kHz | 800 |
/// | 32 kHz | 600 |
/// | 44.1 kHz | 32, 64, 128, 256, 512 |
/// | 48 kHz | 32, 48, 50, 64, 100, 125, 128, 200, 250, 256, 384, 400, 500, 512, 800 |
/// | 88.2 kHz | 128 |
/// | 96 kHz | 64, 256 |
/// | 192 kHz | 128 |
///
/// With BCLK as clock, the multiple is the number of bits per frame, for example 64 for two 32-bit
/// slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clock {
    /// The MCLK pin.
    Mclk(u32),
    /// The BCLK pin, so the board needs no MCLK.
    Bclk(u32),
}

impl Clock {
    fn frequency(self) -> u32 {
        match self {
            Self::Mclk(hz) | Self::Bclk(hz) => hz,
        }
    }
}

/// Serial audio data format of the ADC and the DAC.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// Philips I2S: the MSB follows the LRCK edge by one BCLK.
    I2s,
    /// The MSB starts at the LRCK edge.
    LeftJustified,
    /// DSP/PCM mode A: the MSB follows the LRCK pulse by one BCLK.
    DspA,
    /// DSP/PCM mode B: the MSB starts with the LRCK pulse.
    DspB,
}

/// Bits per sample of the ADC and the DAC.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordLength {
    /// 16 bits per sample.
    Bits16,
    /// 18 bits per sample.
    Bits18,
    /// 20 bits per sample.
    Bits20,
    /// 24 bits per sample.
    Bits24,
    /// 32 bits per sample.
    Bits32,
}

/// Microphone PGA gain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MicGain {
    /// 0 dB.
    Db0 = 0b0000,
    /// 3.5 dB.
    Db3_5 = 0b0001,
    /// 6.5 dB.
    Db6_5 = 0b0010,
    /// 9.5 dB.
    Db9_5 = 0b0011,
    /// 12.5 dB.
    Db12_5 = 0b0110,
    /// 15.5 dB.
    Db15_5 = 0b0111,
    /// 18.5 dB.
    Db18_5 = 0b1000,
    /// 21.5 dB.
    Db21_5 = 0b1001,
    /// 24.5 dB.
    Db24_5 = 0b1010,
    /// 27.5 dB.
    Db27_5 = 0b1011,
    /// 30.5 dB.
    Db30_5 = 0b1100,
    /// 33.5 dB.
    Db33_5 = 0b1101,
    /// 36.5 dB.
    Db36_5 = 0b1110,
}

/// Codec configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    /// The clock the codec derives its internal clocks from.
    pub clock: Clock,
    /// LRCK frequency in Hz.
    pub sample_rate: u32,
    /// Serial audio data format.
    pub format: Format,
    /// Bits per sample.
    pub word_length: WordLength,
    /// Replaces the right ADC channel with the DAC output, as reference for echo cancellation.
    pub dac_reference: bool,
}

/// Driver error, generic over the I2C error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error<E> {
    /// An I2C transfer failed.
    I2c(E),
    /// The clock frequency is not a supported multiple of the sample rate, see [`Clock`].
    UnsupportedClock {
        /// The clock frequency in Hz.
        clock_hz: u32,
        /// The sample rate in Hz.
        sample_rate: u32,
    },
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Self::I2c(error)
    }
}

impl<E: fmt::Debug> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I2c(error) => write!(f, "I2C error: {error:?}"),
            Self::UnsupportedClock {
                clock_hz,
                sample_rate,
            } => write!(
                f,
                "no clock coefficients for a {clock_hz} Hz clock at {sample_rate} Hz"
            ),
        }
    }
}

impl<E: fmt::Debug> core::error::Error for Error<E> {}

/// The register writes of `es8389_open`, with differential microphone inputs, 0 dB mic gain, 0 dB
/// DAC volume and 16-bit Philips I2S.
const OPEN: &[(u8, u8)] = &[
    (0xF3, 0x00),
    (0x00, 0x7E),
    (0xF3, 0x38),
    (0x24, 0x64),
    (0x25, 0x04),
    (0x45, 0x03),
    (0x60, 0x2A),
    (0x61, 0xC9),
    (0x62, 0x4F),
    (0x63, 0x06),
    (0x6B, 0x00),
    (0x6D, 0x16),
    (0x6E, 0xAA),
    (0x6F, 0x66),
    (0x70, 0x99),
    (0x23, 0x00),
    (0x72, MIC_INPUT_DIFFERENTIAL),
    (0x73, MIC_INPUT_DIFFERENTIAL),
    (0x10, 0xC4),
    (0x01, 0x08),
    (0xF1, 0x00),
    (0x12, 0x01),
    (0x13, 0x01),
    (0x14, 0x01),
    (0x15, 0x01),
    (0x16, 0x35),
    (0x17, 0x09),
    (0x18, 0x91),
    (0x19, 0x28),
    (0x1A, 0x01),
    (0x1B, 0x01),
    (0x1C, 0x11),
    (0x2A, 0x00),
    (0x20, 0x60),
    (0x40, 0x60),
    (0xF0, 0x01),
    (0x02, 0x00),
    (0x04, 0x00),
    (0x05, 0x10),
    (0x06, 0x00),
    (0x07, 0xC0),
    (0x08, 0x00),
    (0x09, 0xC0),
    (0x0A, 0x80),
    (0x0B, 0x04),
    (0x0C, 0x01),
    (0x0D, 0x00),
    (0x0F, 0x10),
    (0x21, 0x1F),
    (0x22, 0x7F),
    (0x2F, 0xC0),
    (0x30, 0xF4),
    (0x31, 0x00),
    (0x44, 0x00),
    (0x41, 0x7F),
    (0x42, 0x7F),
    (0x43, 0x10),
    (0x49, 0x0F),
    (0x4C, 0xC0),
    (0x00, 0x00),
    (0x03, 0xC1),
    (0x00, 0x01),
    (0x4D, 0x00),
    (0x26, 0xBF),
    (0x27, 0xBF),
    (0x28, 0xBF),
    (0x46, VOLUME_0DB),
    (0x47, VOLUME_0DB),
    (0x48, 0xBE),
];

/// PGA input selection bits 6:4 of registers 0x72 and 0x73: MIC1P-MIC1N for the left channel and
/// MIC2N-MIC2P for the right channel.
const MIC_INPUT_DIFFERENTIAL: u8 = 0x10;

/// `es8389_set_bias_on` after the read-modify-write of register 0x69.
const BIAS_ON: &[(u8, u8)] = &[
    (0x61, 0xD9),
    (0x64, 0x8F),
    (0x10, 0xE4),
    (0x00, 0x01),
    (0x03, 0xC3),
    (0x24, 0x6A),
    (0x25, 0x0A),
];

/// An ES8389 on an I2C bus.
pub struct Es8389<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C: I2c> Es8389<I2C> {
    /// Creates the driver without accessing the codec. `address` is the 7-bit I2C address, usually
    /// [`DEFAULT_ADDRESS`].
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Returns the I2C bus.
    pub fn release(self) -> I2C {
        self.i2c
    }

    /// Reads the chip ID registers 0xFD and 0xFE.
    pub async fn chip_id(&mut self) -> Result<[u8; 2], I2C::Error> {
        let mut id = [0; 2];
        self.i2c.write_read(self.address, &[0xFD], &mut id).await?;
        Ok(id)
    }

    /// Configures and enables the codec, with 0 dB mic gain and 0 dB DAC volume.
    ///
    /// The codec drops into analog standby without its clock, so the clock must already run.
    pub async fn init(&mut self, config: Config) -> Result<(), Error<I2C::Error>> {
        let clock_hz = config.clock.frequency();
        let Some((_, _, coefficients)) = COEFFICIENTS
            .iter()
            .find(|row| row.0 == clock_hz && row.1 == config.sample_rate)
        else {
            return Err(Error::UnsupportedClock {
                clock_hz,
                sample_rate: config.sample_rate,
            });
        };

        self.write_all(OPEN).await?;
        let clock_source = match config.clock {
            Clock::Mclk(_) => 0x00,
            Clock::Bclk(_) => 0x40,
        };
        self.write(0x02, clock_source).await?;
        if config.dac_reference {
            self.update(0x23, 0x80, 0x80).await?;
        }
        self.write(0xF0, 0x12 | u8::from(config.dac_reference) << 3)
            .await?;

        for (&(register, mask), &value) in REGISTERS.iter().zip(coefficients) {
            self.update(register, mask, value).await?;
        }

        let word_length = match config.word_length {
            WordLength::Bits16 => 0x60,
            WordLength::Bits18 => 0x40,
            WordLength::Bits20 => 0x20,
            WordLength::Bits24 => 0x00,
            WordLength::Bits32 => 0x80,
        };
        let (format, lrck_format) = match config.format {
            Format::I2s => (0x00, 0x00),
            Format::LeftJustified => (0x04, 0x40),
            Format::DspA => (0x08, 0x80),
            Format::DspB => (0x18, 0xA0),
        };
        self.update(0x20, 0xFC, word_length | format).await?;
        self.update(0x40, 0xFC, word_length | format).await?;
        self.update(0x0C, 0xE0, lrck_format).await?;

        Ok(self.enable().await?)
    }

    /// Sets the PGA gain of both microphones.
    pub async fn set_mic_gain(&mut self, gain: MicGain) -> Result<(), I2C::Error> {
        let value = MIC_INPUT_DIFFERENTIAL | gain as u8;
        self.write_all(&[(0x72, value), (0x73, value)]).await
    }

    /// Sets the volume of both DAC channels in 0.5 dB steps, from -95.5 dB (0x00) over 0 dB
    /// ([`VOLUME_0DB`]) to +32 dB (0xFF).
    pub async fn set_dac_volume(&mut self, volume: u8) -> Result<(), I2C::Error> {
        self.write_all(&[(0x46, volume), (0x47, volume)]).await
    }

    /// Mutes or unmutes both ADC channels, the recording.
    pub async fn set_adc_mute(&mut self, mute: bool) -> Result<(), I2C::Error> {
        self.update(0x20, 0x03, if mute { 0x03 } else { 0x00 })
            .await
    }

    /// Mutes or unmutes both DAC channels, the playback.
    pub async fn set_dac_mute(&mut self, mute: bool) -> Result<(), I2C::Error> {
        self.update(0x40, 0x03, if mute { 0x03 } else { 0x00 })
            .await
    }

    /// Powers the analog part up and unmutes the ADC.
    pub async fn enable(&mut self) -> Result<(), I2C::Error> {
        self.write(0x4D, 0x00).await?;
        self.update(0x69, 0x20, 0x20).await?;
        self.write_all(BIAS_ON).await?;
        self.set_adc_mute(false).await
    }

    /// Mutes the ADC and puts the analog part into standby, until [`Self::enable`].
    ///
    /// Leaves the DAC unmuted.
    pub async fn disable(&mut self, delay: &mut impl DelayNs) -> Result<(), I2C::Error> {
        self.set_adc_mute(true).await?;
        self.standby(
            delay,
            &[(0x61, 0x59), (0x64, 0x00), (0x03, 0x00), (0x00, 0x7E)],
        )
        .await
    }

    /// Mutes the ADC and powers the codec down, until [`Self::init`].
    pub async fn power_down(&mut self, delay: &mut impl DelayNs) -> Result<(), I2C::Error> {
        self.set_adc_mute(true).await?;
        self.standby(
            delay,
            &[(0x61, 0x59), (0x64, 0x00), (0x63, 0x00), (0x00, 0x7E)],
        )
        .await?;
        self.write(0x01, 0x28).await?;
        self.update(0x69, 0x20, 0x00).await?;
        self.write_all(&[(0x60, 0x00), (0x00, 0x00), (0x10, 0xCC)])
            .await?;
        delay.delay_ms(500).await;
        self.write_all(&[(0x10, 0x00), (0x61, 0x08), (0xF3, 0xC1), (0xF2, 0x00)])
            .await
    }

    /// `es8389_set_bias_standby`, with the analog writes that differ between standby and
    /// power-down.
    async fn standby(
        &mut self,
        delay: &mut impl DelayNs,
        analog: &[(u8, u8)],
    ) -> Result<(), I2C::Error> {
        self.set_dac_mute(true).await?;
        self.write(0x10, 0xD4).await?;
        delay.delay_ms(70).await;
        self.write_all(analog).await?;
        self.set_dac_mute(false).await
    }

    async fn update(&mut self, register: u8, mask: u8, value: u8) -> Result<(), I2C::Error> {
        if mask == 0xFF {
            return self.write(register, value).await;
        }
        let mut current = [0];
        self.i2c
            .write_read(self.address, &[register], &mut current)
            .await?;
        self.write(register, current[0] & !mask | value & mask)
            .await
    }

    async fn write_all(&mut self, registers: &[(u8, u8)]) -> Result<(), I2C::Error> {
        for &(register, value) in registers {
            self.write(register, value).await?;
        }
        Ok(())
    }

    async fn write(&mut self, register: u8, value: u8) -> Result<(), I2C::Error> {
        self.i2c.write(self.address, &[register, value]).await
    }
}

#[cfg(test)]
mod tests {
    use core::{
        convert::Infallible,
        pin::pin,
        task::{Context, Poll, Waker},
    };

    use embedded_hal_async::i2c::{ErrorType, Operation};

    use super::*;

    /// A register file in place of the codec.
    struct Registers {
        values: [u8; 256],
        writes: usize,
    }

    impl ErrorType for Registers {
        type Error = Infallible;
    }

    impl I2c for Registers {
        async fn transaction(
            &mut self,
            address: u8,
            operations: &mut [Operation<'_>],
        ) -> Result<(), Infallible> {
            assert_eq!(address, DEFAULT_ADDRESS);
            let mut register = 0;
            for operation in operations {
                match operation {
                    Operation::Write([selected]) => register = usize::from(*selected),
                    Operation::Write([selected, value]) => {
                        self.values[usize::from(*selected)] = *value;
                        self.writes += 1;
                    }
                    Operation::Read(buffer) => {
                        buffer.copy_from_slice(&self.values[register..][..buffer.len()])
                    }
                    Operation::Write(bytes) => panic!("unexpected write {bytes:02x?}"),
                }
            }
            Ok(())
        }
    }

    fn block_on<F: Future>(future: F) -> F::Output {
        match pin!(future).poll(&mut Context::from_waker(Waker::noop())) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("the register file never blocks"),
        }
    }

    fn init(config: Config) -> (Result<(), Error<Infallible>>, Registers) {
        let mut codec = Es8389::new(
            Registers {
                values: [0; 256],
                writes: 0,
            },
            DEFAULT_ADDRESS,
        );
        let result = block_on(codec.init(config));
        (result, codec.release())
    }

    const KORVO: Config = Config {
        clock: Clock::Bclk(3_072_000),
        sample_rate: 48_000,
        format: Format::I2s,
        word_length: WordLength::Bits16,
        dac_reference: false,
    };

    #[test]
    fn init_with_bclk() {
        let (result, registers) = init(KORVO);
        assert_eq!(result, Ok(()));
        let values = registers.values;
        // Clock from BCLK, the coefficients of the 3.072 MHz / 48 kHz row, 16-bit Philips I2S.
        assert_eq!(values[0x02], 0x40);
        assert_eq!(values[0x05], 0x51);
        assert_eq!(values[0x20], 0x60);
        assert_eq!(values[0x40], 0x60);
        assert_eq!(values[0x0C], 0x01);
        // Enabled.
        assert_eq!(values[0x00], 0x01);
        assert_eq!(values[0x69] & 0x20, 0x20);
    }

    #[test]
    fn init_with_format() {
        let (result, registers) = init(Config {
            clock: Clock::Mclk(12_288_000),
            format: Format::LeftJustified,
            word_length: WordLength::Bits24,
            dac_reference: true,
            ..KORVO
        });
        assert_eq!(result, Ok(()));
        let values = registers.values;
        assert_eq!(values[0x02], 0x00);
        assert_eq!(values[0x20], 0x04);
        assert_eq!(values[0x40], 0x04);
        assert_eq!(values[0x0C], 0x41);
        assert_eq!(values[0x23], 0x80);
        assert_eq!(values[0xF0] & 0x08, 0x08);
    }

    #[test]
    fn init_rejects_an_unsupported_clock_before_writing() {
        let (result, registers) = init(Config {
            clock: Clock::Bclk(1_000_000),
            ..KORVO
        });
        assert_eq!(
            result,
            Err(Error::UnsupportedClock {
                clock_hz: 1_000_000,
                sample_rate: 48_000
            })
        );
        assert_eq!(registers.writes, 0);
    }
}
