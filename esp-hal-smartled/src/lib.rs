//! Allows for the use of an RMT output channel on the ESP32 family to easily drive smart RGB LEDs. This is a driver for the [smart-leds](https://crates.io/crates/smart-leds) framework and allows using the utility functions from this crate as well as higher-level libraries based on smart-leds.
//!
//! Different from [ws2812-esp32-rmt-driver](https://crates.io/crates/ws2812-esp32-rmt-driver), which is based on the unofficial `esp-idf` SDK, this crate is based on the official no-std [esp-hal](https://github.com/esp-rs/esp-hal).
//!
//! This driver uses the blocking RMT API, which is not suitable for use in async code. The [`SmartLedsWrite`] trait is implemented for [`SmartLedsAdapter`] only if a [`Blocking`] RMT channel is passed.
//!
//! ## Example
//!
//! ```rust,ignore
//! let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();
//!
//! let mut led = SmartLedsAdapter::<{ buffer_size(1) }, _, color_order::Rgb, Ws2812Timing>::new(
//!     rmt.channel0, peripherals.GPIO2
//! );
//!
//! led.write(brightness([RED], 10)).unwrap();
//! ```
//!
//! ## Usage overview
//!
//! The [`SmartLedsAdapter`] struct implements [`SmartLedsWrite`]
//! and can be used to send color data to connected LEDs.
//! To initialize a [`SmartLedsAdapter`], use [`SmartLedsAdapter::new`],
//! which takes an RMT channel and a [`PeripheralOutput`].
//! If you want to reuse the channel afterwards, you can use [`esp_hal::rmt::ChannelCreator::reborrow`] to create a shorter-lived derived channel.
//! [`SmartLedsAdapter`] is configured at compile-time to support a variety of LED configurations. See the documentation for [`SmartLedsAdapter`] for more info.
//!
//! ## Features
//!
//! None of the features provided by this crate are for external use, they are only used for testing and examples.
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/46717278")]
#![deny(missing_docs)]
#![no_std]

use core::{fmt::Debug, marker::PhantomData, slice::IterMut};

pub use color_order::ColorOrder;
use esp_hal::{
    Async, Blocking, DriverMode,
    clock::Clocks,
    gpio::{Level, interconnect::PeripheralOutput},
    rmt::{Channel, Error as RmtError, PulseCode, Tx, TxChannelConfig, TxChannelCreator},
};
use smart_leds_trait::{RGB8, SmartLedsWrite, SmartLedsWriteAsync};

/// Common trait for all different smart LED dependent timings.
///
/// All common smart LEDs are controlled by sending PWM-like pulses, in two different configurations for high and low.
/// The required timings (and tolerances) can be found in the relevant datasheets.
///
/// Provided timings: [`Sk68xxTiming`], [`Ws2812bTiming`], [`Ws2811Timing`], [`Ws2812Timing`]
// Implementations of this should be vacant enums so they can’t be constructed.
pub trait Timing {
    /// Low time for zero pulse, in nanoseconds.
    const TIME_0_LOW: u16;
    /// High time for zero pulse, in nanoseconds.
    const TIME_0_HIGH: u16;
    /// Low time for one pulse, in nanoseconds.
    const TIME_1_LOW: u16;
    /// High time for one pulse, in nanoseconds.
    const TIME_1_HIGH: u16;
}

const SK68XX_CODE_PERIOD: u16 = 1200;
/// Timing for the SK68 collection of LEDs.
/// Note: it is not verified that this is correct, the datasheet for SK6812 says otherwise.
/// These values have been carried over from an earlier version.
pub enum Sk68xxTiming {}
impl Timing for Sk68xxTiming {
    const TIME_0_HIGH: u16 = 320;
    const TIME_0_LOW: u16 = SK68XX_CODE_PERIOD - Self::TIME_0_HIGH;
    const TIME_1_HIGH: u16 = 640;
    const TIME_1_LOW: u16 = SK68XX_CODE_PERIOD - Self::TIME_1_HIGH;
}

/// Timing for the WS2812B LEDs.
pub enum Ws2812bTiming {}
impl Timing for Ws2812bTiming {
    const TIME_0_HIGH: u16 = 400;
    const TIME_0_LOW: u16 = 800;
    const TIME_1_HIGH: u16 = 850;
    const TIME_1_LOW: u16 = 450;
}

/// Timing for the WS2812 LEDs.
pub enum Ws2812Timing {}
impl Timing for Ws2812Timing {
    const TIME_0_HIGH: u16 = 350;
    const TIME_0_LOW: u16 = 700;
    const TIME_1_HIGH: u16 = 800;
    const TIME_1_LOW: u16 = 600;
}

/// Timing for the WS2811 driver ICs, low-speed mode.
pub enum Ws2811LowSpeedTiming {}
impl Timing for Ws2811LowSpeedTiming {
    const TIME_0_HIGH: u16 = 500;
    const TIME_0_LOW: u16 = 2000;
    const TIME_1_HIGH: u16 = 1200;
    const TIME_1_LOW: u16 = 1300;
}

/// Timing for the WS2811 driver ICs, high-speed mode.
pub enum Ws2811Timing {}
impl Timing for Ws2811Timing {
    const TIME_0_HIGH: u16 = Ws2811LowSpeedTiming::TIME_0_HIGH / 2;
    const TIME_0_LOW: u16 = Ws2811LowSpeedTiming::TIME_0_LOW / 2;
    const TIME_1_HIGH: u16 = Ws2811LowSpeedTiming::TIME_1_HIGH / 2;
    const TIME_1_LOW: u16 = Ws2811LowSpeedTiming::TIME_1_LOW / 2;
}

/// All types of errors that can happen during the conversion and transmission
/// of LED commands.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum AdapterError {
    /// Raised in the event that the RMT buffer is not large enough.
    ///
    /// This almost always points to an issue with the `BUFFER_SIZE` parameter of [`SmartLedsAdapter`]. You should create this parameter using [`buffer_size`], passing in the desired number of LEDs that will be controlled.
    BufferSizeExceeded,
    /// Raised if something goes wrong in the transmission. This contains the inner HAL error ([`RmtError`]).
    TransmissionError(RmtError),
}

impl From<RmtError> for AdapterError {
    fn from(value: RmtError) -> Self {
        Self::TransmissionError(value)
    }
}

/// Calculate the required buffer size for a certain number of LEDs. This should be used to create the `BUFFER_SIZE` parameter of [`SmartLedsAdapter`].
///
/// Attempting to use more LEDs that the buffer is configured for will result in
/// an [`AdapterError::BufferSizeExceeded`] error.
pub const fn buffer_size(led_count: usize) -> usize {
    // The size we're assigning here is calculated as following
    //  (
    //   Nr. of LEDs
    //   * channels (r,g,b -> 3)
    //   * pulses per channel 8)
    //  ) + 1 additional pulse for the end delimiter
    led_count * 24 + 1
}

/// Common [`ColorOrder`] implementations.
pub mod color_order {
    use smart_leds_trait::RGB8;

    /// Specific channel to request from [`ColorOrder`].
    #[derive(Copy, Clone, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[repr(u8)]
    pub enum Channel {
        /// First channel.
        First = 0,
        /// Second channel.
        Second = 1,
        /// Third channel.
        Third = 2,
    }

    /// Order of colors in the physical LEDs.
    /// Some common color orders are:
    /// - [`Rgb`] for WS2811
    /// - [`Grb`] for SK86XX and WS2812(B)
    // Implementations of this should be vacant enums so they can’t be constructed.
    pub trait ColorOrder {
        /// Retrieve the output value for the provided channel.
        /// For instance, if color order is RGB, then the red value will be returned for channel 0,
        /// the green value for channel 1 and the blue value for channel 2.
        fn get_channel_data(color: RGB8, channel: Channel) -> u8;
    }

    macro_rules! color_order {
        ($name:ident => $first:ident, $second:ident, $third:ident) => {
            #[doc = concat!("[`ColorOrder`] ", stringify!($name), ".")]
            pub enum $name {}
            impl ColorOrder for $name {
                fn get_channel_data(color: RGB8, channel: Channel) -> u8 {
                    match channel {
                        Channel::First => color.$first,
                        Channel::Second => color.$second,
                        Channel::Third => color.$third,
                    }
                }
            }
        };
    }

    color_order!(Rgb => r, g, b);
    color_order!(Rbg => r, b, g);
    color_order!(Grb => g, r, b);
    color_order!(Gbr => g, b, r);
    color_order!(Brg => b, r, g);
    color_order!(Bgr => b, g, r);
}

/// [`SmartLedsWrite`] driver implementation using the ESP32’s “remote control” (RMT) peripheral for hardware-offloaded, fast control of smart LEDs.
///
/// For usage examples and a general overview see [the crate documentation](`crate`).
///
/// This type supports many configurations of color order, LED timings, and LED count. For this reason, there are three main type parameters you have to choose:
/// - The buffer size. This determines how many RMT pulses can be sent by this driver, and allows it to function entirely without heap allocation. It is strongly recommended to use the [`buffer_size`] function with the desired number of LEDs to choose a correct buffer size, otherwise [`SmartLedsWrite::write`] will return [`AdapterError::BufferSizeExceeded`].
/// - The [`ColorOrder`]. This determines what order the LED expects the color values in. Almost all LEDs use [`color_order::Rgb`] or [`color_order::Grb`].
/// - The [`Timing`]. This determines the smart LED type in use; what kind of signal it expects. Several implementations for common LED types like WS2812 are provided. Note that many WS2812-like LEDs are at least almost compatible in their timing, even though the datasheets specify different amounts, the other LEDs’ values are within the tolerance range, and even exceeding these, many LEDs continue to work beyond their specified timing range. It is however recommended to use the corresponding LED type, or implement your own when needed.
///
/// When the driver move is [`Blocking`], this type implements the blocking [`SmartLedsWrite`] interface. An async interface for [`esp_hal::Async`] may be added in the future. (You usually don’t need to choose this manually, Rust can deduce it from the passed-in RMT channel.)
pub struct SmartLedsAdapter<'d, const BUFFER_SIZE: usize, Mode, Order, Timing>
where
    Mode: DriverMode,
    Order: ColorOrder,
    Timing: crate::Timing,
{
    channel: Option<Channel<'d, Mode, Tx>>,
    rmt_buffer: [PulseCode; BUFFER_SIZE],
    pulses: (PulseCode, PulseCode),
    _order: PhantomData<Order>,
    _timing: PhantomData<Timing>,
}

impl<'d, const BUFFER_SIZE: usize, Mode, Order, Timing>
    SmartLedsAdapter<'d, BUFFER_SIZE, Mode, Order, Timing>
where
    Mode: DriverMode,
    Order: ColorOrder,
    Timing: crate::Timing,
{
    /// Creates a new [`SmartLedsAdapter`] that drives the provided output using the given RMT channel.
    ///
    /// Note that calling this function usually requires you to specify the desired buffer size, [`ColorOrder`] and [`Timing`]. See the struct documentation for details.
    ///
    /// If you want to reuse the channel afterwards, you can use [`esp_hal::rmt::ChannelCreator::reborrow`] to create a shorter-lived derived channel.
    ///
    /// # Errors
    ///
    /// If any configuration issue with the RMT [`Channel`] occurs, the error will be returned.
    pub fn new<C, P>(channel: C, pin: P) -> Result<Self, RmtError>
    where
        C: TxChannelCreator<'d, Mode>,
        P: PeripheralOutput<'d>,
    {
        Self::new_with_memsize(channel, pin, 1)
    }
    /// Creates a new [`SmartLedsAdapter`] that drives the provided output using the given RMT channel.
    ///
    /// Note that calling this function usually requires you to specify the desired buffer size, [`ColorOrder`] and [`Timing`]. See the struct documentation for details.
    ///
    /// If you want to reuse the channel afterwards, you can use [`esp_hal::rmt::ChannelCreator::reborrow`] to create a shorter-lived derived channel.
    ///
    /// The `memsize` parameter determines how many RMT blocks this adapter will use.
    /// If you use any value other than 1, other RMT channels will not be available, as their memory blocks will be used up by this driver.
    /// However, this can allow you to control many more LEDs without issues.
    ///
    /// # Errors
    ///
    /// If any configuration issue with the RMT [`Channel`] occurs, the error will be returned.
    pub fn new_with_memsize<C, P>(channel: C, pin: P, memsize: u8) -> Result<Self, RmtError>
    where
        C: TxChannelCreator<'d, Mode>,
        P: PeripheralOutput<'d>,
    {
        let config = TxChannelConfig::default()
            .with_clk_divider(1)
            .with_idle_output_level(Level::Low)
            .with_memsize(memsize)
            .with_carrier_modulation(false)
            .with_idle_output(true);

        let channel = channel.configure_tx(pin, config)?;

        // Assume the RMT peripheral is set up to use the APB clock
        let clocks = Clocks::get();
        // convert to the MHz value to simplify nanosecond calculations
        let src_clock = clocks.apb_clock.as_hz() / 1_000_000;

        Ok(Self {
            channel: Some(channel),
            rmt_buffer: [PulseCode::end_marker(); _],
            pulses: (
                PulseCode::new(
                    Level::High,
                    ((Timing::TIME_0_HIGH as u32 * src_clock) / 1000) as u16,
                    Level::Low,
                    ((Timing::TIME_0_LOW as u32 * src_clock) / 1000) as u16,
                ),
                PulseCode::new(
                    Level::High,
                    ((Timing::TIME_1_HIGH as u32 * src_clock) / 1000) as u16,
                    Level::Low,
                    ((Timing::TIME_1_LOW as u32 * src_clock) / 1000) as u16,
                ),
            ),
            _order: PhantomData,
            _timing: PhantomData,
        })
    }

    fn convert_rgb_to_pulse(
        value: RGB8,
        mut_iter: &mut IterMut<PulseCode>,
        pulses: (PulseCode, PulseCode),
    ) -> Result<(), AdapterError> {
        use crate::color_order::Channel;

        Self::convert_rgb_channel_to_pulses(
            Order::get_channel_data(value, Channel::First),
            mut_iter,
            pulses,
        )?;
        Self::convert_rgb_channel_to_pulses(
            Order::get_channel_data(value, Channel::Second),
            mut_iter,
            pulses,
        )?;
        Self::convert_rgb_channel_to_pulses(
            Order::get_channel_data(value, Channel::Third),
            mut_iter,
            pulses,
        )?;

        Ok(())
    }

    fn convert_rgb_channel_to_pulses(
        channel_value: u8,
        mut_iter: &mut IterMut<PulseCode>,
        pulses: (PulseCode, PulseCode),
    ) -> Result<(), AdapterError> {
        for position in [128, 64, 32, 16, 8, 4, 2, 1] {
            *mut_iter.next().ok_or(AdapterError::BufferSizeExceeded)? =
                match channel_value & position {
                    0 => pulses.0,
                    _ => pulses.1,
                }
        }

        Ok(())
    }

    /// Create and store RMT data from the color information provided.
    fn create_rmt_data(
        &mut self,
        iterator: impl IntoIterator<Item = impl Into<RGB8>>,
    ) -> Result<(), AdapterError> {
        // We always start from the beginning of the buffer
        let mut seq_iter = self.rmt_buffer.iter_mut();

        // Add all converted iterator items to the buffer.
        // This will result in an `BufferSizeExceeded` error in case
        // the iterator provides more elements than the buffer can take.
        for item in iterator {
            Self::convert_rgb_to_pulse(item.into(), &mut seq_iter, self.pulses)?;
        }

        // Finally, add an end element.
        *seq_iter.next().ok_or(AdapterError::BufferSizeExceeded)? = PulseCode::end_marker();

        Ok(())
    }
}

impl<'d, const BUFFER_SIZE: usize, Order, Timing> SmartLedsWrite
    for SmartLedsAdapter<'d, BUFFER_SIZE, Blocking, Order, Timing>
where
    Order: ColorOrder,
    Timing: crate::Timing,
{
    type Error = AdapterError;
    type Color = RGB8;

    /// Convert all RGB8 items of the iterator to the RMT format and
    /// add them to internal buffer, then start a singular RMT operation
    /// based on that buffer.
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.create_rmt_data(iterator)?;

        // Perform the actual RMT operation. We use the u32 values here right away.
        let channel = self.channel.take().unwrap();
        // TODO: If the transmit fails, we’re in an unsafe state and future calls to write() will panic.
        // This is currently unavoidable since transmit consumes the channel on error.
        // This is a known design flaw in the current RMT API and will be fixed soon.
        // We should adjust our usage accordingly as soon as possible.
        match channel.transmit(&self.rmt_buffer)?.wait() {
            Ok(chan) => {
                self.channel = Some(chan);
                Ok(())
            }
            Err((e, chan)) => {
                self.channel = Some(chan);
                Err(AdapterError::TransmissionError(e))
            }
        }
    }
}

impl<'d, const BUFFER_SIZE: usize, Order, Timing> SmartLedsWriteAsync
    for SmartLedsAdapter<'d, BUFFER_SIZE, Async, Order, Timing>
where
    Order: ColorOrder,
    Timing: crate::Timing,
{
    type Error = AdapterError;
    type Color = RGB8;

    /// Convert all RGB8 items of the iterator to the RMT format and
    /// add them to internal buffer, then start a singular RMT operation
    /// based on that buffer.
    async fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.create_rmt_data(iterator)?;

        // Perform the actual RMT operation. We use the u32 values here right away.
        self.channel
            .as_mut()
            .unwrap()
            .transmit(&self.rmt_buffer)
            .await?;
        Ok(())
    }
}
