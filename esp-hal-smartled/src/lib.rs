//! Allows for the use of an RMT output channel on the ESP32 family to easily drive smart RGB LEDs. This is a driver for the [smart-leds](https://crates.io/crates/smart-leds) framework and allows using the utility functions from this crate as well as higher-level libraries based on smart-leds.
//!
//! Different from [ws2812-esp32-rmt-driver](https://crates.io/crates/ws2812-esp32-rmt-driver), which is based on the unofficial `esp-idf` SDK, this crate is based on the official no-std [esp-hal](https://github.com/esp-rs/esp-hal).
//!
//! This driver uses the blocking RMT API, which is not suitable for use in async code. The [`SmartLedsWrite`] trait is implemented for [`RmtSmartLeds`] only if a [`Blocking`] RMT channel is passed.
//!
//! ## Example
//!
//! ```rust,ignore
//! let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();
//!
//! let mut led = RmtSmartLeds::<{ buffer_size::<RGB8>(1) }, _, RGB8, color_order::Rgb, Ws2812Timing>::new(
//!     rmt.channel0, peripherals.GPIO2
//! );
//!
//! led.write(brightness([RED], 10)).unwrap();
//! ```
//!
//! ## Usage overview
//!
//! The [`RmtSmartLeds`] struct implements [`SmartLedsWrite`]
//! and can be used to send color data to connected LEDs.
//! To initialize a [`RmtSmartLeds`], use [`RmtSmartLeds::new`],
//! which takes an RMT channel and a [`PeripheralOutput`].
//! If you want to reuse the channel afterwards, you can use [`esp_hal::rmt::ChannelCreator::reborrow`] to create a shorter-lived derived channel.
//! [`RmtSmartLeds`] is configured at compile-time to support a variety of LED configurations. See the documentation for [`RmtSmartLeds`] for more info.
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
use num_traits::Unsigned;
use smart_leds_trait::{
    CctWhite, RGB, RGB8, RGBCCT, RGBW, SmartLedsWrite, SmartLedsWriteAsync, White,
};

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
    /// This almost always points to an issue with the `BUFFER_SIZE` parameter of [`RmtSmartLeds`].
    /// You should create this parameter using [`buffer_size`], passing in the desired number of LEDs that will be controlled.
    BufferSizeExceeded,
    /// Raised if something goes wrong in the transmission. This contains the inner HAL error ([`RmtError`]).
    TransmissionError(RmtError),
}

impl From<RmtError> for AdapterError {
    fn from(value: RmtError) -> Self {
        Self::TransmissionError(value)
    }
}

/// Utility trait that retrieves metadata about all [`smart_leds`] color types.
pub trait Color {
    /// The maximum channel number this color supports.
    ///
    /// - For RGB (or any permutation thereof), this is 3.
    /// - For RGBW, this is 4.
    /// - For RGBCCT, this is 5.
    /// - For CCT, this is 2.
    ///
    /// Note that this channel count is used by users of [`ColorOrder`] to limit the channel number that’s passed into [`ColorOrder::get_channel_data`].
    const CHANNELS: u8;

    /// Type of a single channel of this color. Usually [`u8`], but [`u16`] is also used for some LEDs.
    type ChannelType: Unsigned + Into<usize>;
}

impl<T> Color for RGB<T>
where
    T: Unsigned + Into<usize>,
{
    const CHANNELS: u8 = 3;
    type ChannelType = T;
}

impl<T> Color for RGBW<T>
where
    T: Unsigned + Into<usize>,
{
    const CHANNELS: u8 = 4;
    type ChannelType = T;
}

impl<T> Color for RGBCCT<T>
where
    T: Unsigned + Into<usize>,
{
    const CHANNELS: u8 = 5;
    type ChannelType = T;
}

impl<T> Color for White<T>
where
    T: Unsigned + Into<usize>,
{
    const CHANNELS: u8 = 1;
    type ChannelType = T;
}

impl<T> Color for CctWhite<T>
where
    T: Unsigned + Into<usize>,
{
    const CHANNELS: u8 = 2;
    type ChannelType = T;
}

/// Calculate the required buffer size for a certain number of LEDs.
/// This should be used to create the `BUFFER_SIZE` parameter of [`RmtSmartLeds`].
///
/// Attempting to use more LEDs that the buffer is configured for will result in
/// an [`AdapterError::BufferSizeExceeded`] error.
///
/// You need to specify the correct color and channel type
// TODO: As soon as generic expressions are more stabilized, we should be able to do this calculation entirely internally in `RmtSmartLeds`. For now, users have to be careful.
pub const fn buffer_size<C: Color>(led_count: usize) -> usize
where
{
    // The size we're assigning here is calculated as following
    //  (
    //   Nr. of LEDs
    //   * channels
    //   * pulses per channel (=bitcount)
    //  ) + 1 additional pulse for the end delimiter
    led_count * (size_of::<C::ChannelType>() * 8) * C::CHANNELS as usize + 1
}

/// Common [`ColorOrder`] implementations.
pub mod color_order {
    use num_traits::Unsigned;
    use smart_leds_trait::{RGB, RGBW};

    use crate::Color;

    /// Order of colors in the physical LEDs.
    /// The most common color orders for RGB LEDs are [`Rgb`] (most integrated controllers like WS2812) and [`Grb`].
    /// Note that discrete ICs have generic channels and are often wired up arbitrarily, so you will have to check which order is correct for your hardware.
    // Implementations of this should be vacant enums so they can’t be constructed.
    // This should also be a constant trait once that becomes a stable Rust feature.
    pub trait ColorOrder<C: Color> {
        /// Retrieve the output value for the provided channel.
        /// For instance, if color order is RGB, then the red value will be returned for channel 0,
        /// the green value for channel 1 and the blue value for channel 2.
        ///
        /// The maximum channel number users are allowed to pass in is [`ChannelCount::CHANNELS`] (on `Color`) minus one.
        /// If this restriction is not upheld, the implementation may panic.
        fn get_channel_data(color: &C, channel: u8) -> C::ChannelType;
    }

    macro_rules! color_order_rgb {
        ($name:ident => $first:ident, $second:ident, $third:ident) => {
            #[doc = concat!("[`ColorOrder`] ", stringify!($name), ".")]
            pub enum $name {}
            impl<T> ColorOrder<RGB<T>> for $name
            where
                T: Copy + Unsigned + Into<usize>,
            {
                fn get_channel_data(color: &RGB<T>, channel: u8) -> T {
                    match channel {
                        0 => color.$first,
                        1 => color.$second,
                        2 => color.$third,
                        _ => unreachable!(),
                    }
                }
            }
        };
    }

    color_order_rgb!(Rgb => r, g, b);
    color_order_rgb!(Rbg => r, b, g);
    color_order_rgb!(Grb => g, r, b);
    color_order_rgb!(Gbr => g, b, r);
    color_order_rgb!(Brg => b, r, g);
    color_order_rgb!(Bgr => b, g, r);

    /// [`ColorOrder`] RGBW.
    pub enum Rgbw {}
    impl<T> ColorOrder<RGBW<T>> for Rgbw
    where
        T: Copy + Unsigned + Into<usize>,
    {
        fn get_channel_data(color: &RGBW<T>, channel: u8) -> T {
            match channel {
                0 => color.r,
                1 => color.g,
                2 => color.b,
                3 => color.a.0,
                _ => unreachable!(),
            }
        }
    }
}

/// [`SmartLedsWrite`] driver implementation using the ESP32’s “remote control” (RMT) peripheral for hardware-offloaded, fast control of smart LEDs.
///
/// For usage examples and a general overview see [the crate documentation](`crate`).
///
/// This type supports many configurations of color order, LED timings, and LED count. For this reason, there are three main type parameters you have to choose:
/// - The buffer size. This determines how many RMT pulses can be sent by this driver, and allows it to function entirely without heap allocation. It is strongly recommended to use the [`buffer_size`] function with the desired number of LEDs to choose a correct buffer size, otherwise [`SmartLedsWrite::write`] will return [`AdapterError::BufferSizeExceeded`].
/// - The `Color`.
///   This determines the color model and number of channels to be sent.
///   `Color` must implement [`ChannelCount`], which is already the case for all [`smart_leds`] colors.
/// - The [`ColorOrder`].
///   This determines what order the LED expects the color values in.
/// - The [`Timing`].
///   This determines the smart LED type in use; what kind of signal it expects.
///   Several implementations for common LED types like WS2812 are provided.
///   Note that many WS2812-like LEDs are at least almost compatible in their timing, even though the datasheets specify different amounts, the other LEDs’ values are within the tolerance range, and even exceeding these, many LEDs continue to work beyond their specified timing range.
///   It is however recommended to use the corresponding LED type, or implement your own when needed.
///
/// When the driver move is [`Blocking`], this type implements the blocking [`SmartLedsWrite`] interface. An async interface for [`esp_hal::Async`] may be added in the future. (You usually don’t need to choose this manually, Rust can deduce it from the passed-in RMT channel.)
pub struct RmtSmartLeds<'d, const BUFFER_SIZE: usize, Mode, C, Order, Timing>
where
    Mode: DriverMode,
    C: Color,
    Order: ColorOrder<C>,
    Timing: crate::Timing,
{
    channel: Option<Channel<'d, Mode, Tx>>,
    rmt_buffer: [PulseCode; BUFFER_SIZE],
    pulses: (PulseCode, PulseCode),
    _order: PhantomData<Order>,
    _timing: PhantomData<Timing>,
    _color: PhantomData<C>,
}

/// A [`RmtSmartLeds`] specifically for 8-bit RGB colors, which is what most smart LEDs use.
pub type Rgb8RmtSmartLeds<'d, const BUFFER_SIZE: usize, Mode, Order, Timing> =
    RmtSmartLeds<'d, BUFFER_SIZE, Mode, RGB8, Order, Timing>;

impl<'d, const BUFFER_SIZE: usize, Mode, C, Order, Timing>
    RmtSmartLeds<'d, BUFFER_SIZE, Mode, C, Order, Timing>
where
    Mode: DriverMode,
    C: Color,
    Order: ColorOrder<C>,
    Timing: crate::Timing,
{
    /// Creates a new [`RmtSmartLeds`] that drives the provided output using the given RMT channel.
    ///
    /// Note that calling this function usually requires you to specify the desired buffer size, [`ColorOrder`] and [`Timing`]. See the struct documentation for details.
    ///
    /// If you want to reuse the channel afterwards, you can use [`esp_hal::rmt::ChannelCreator::reborrow`] to create a shorter-lived derived channel.
    ///
    /// # Errors
    ///
    /// If any configuration issue with the RMT [`Channel`] occurs, the error will be returned.
    pub fn new<Ch, P>(channel: Ch, pin: P) -> Result<Self, RmtError>
    where
        Ch: TxChannelCreator<'d, Mode>,
        P: PeripheralOutput<'d>,
    {
        Self::new_with_memsize(channel, pin, 1)
    }
    /// Creates a new [`RmtSmartLeds`] that drives the provided output using the given RMT channel.
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
    pub fn new_with_memsize<Ch, P>(channel: Ch, pin: P, memsize: u8) -> Result<Self, RmtError>
    where
        Ch: TxChannelCreator<'d, Mode>,
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
            _color: PhantomData,
        })
    }

    /// Create and store RMT data from the color information provided.
    fn create_rmt_data(
        &mut self,
        iterator: impl IntoIterator<Item = impl Into<C>>,
    ) -> Result<(), AdapterError> {
        // We always start from the beginning of the buffer
        let mut seq_iter = self.rmt_buffer.iter_mut();

        // Add all converted iterator items to the buffer.
        // This will result in an `BufferSizeExceeded` error in case
        // the iterator provides more elements than the buffer can take.
        for item in iterator {
            convert_colors_to_pulse::<_, Order>(&item.into(), &mut seq_iter, self.pulses)?;
        }

        // Finally, add an end element.
        *seq_iter.next().ok_or(AdapterError::BufferSizeExceeded)? = PulseCode::end_marker();

        Ok(())
    }
}

impl<'d, const BUFFER_SIZE: usize, C, Order, Timing> SmartLedsWrite
    for RmtSmartLeds<'d, BUFFER_SIZE, Blocking, C, Order, Timing>
where
    C: Color,
    Order: ColorOrder<C>,
    Timing: crate::Timing,
{
    type Error = AdapterError;
    type Color = C;

    /// Convert all Color items of the iterator to the RMT format and
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

impl<'d, const BUFFER_SIZE: usize, C, Order, Timing> SmartLedsWriteAsync
    for RmtSmartLeds<'d, BUFFER_SIZE, Async, C, Order, Timing>
where
    C: Color,
    Order: ColorOrder<C>,
    Timing: crate::Timing,
{
    type Error = AdapterError;
    type Color = C;

    /// Convert all Color items of the iterator to the RMT format and
    /// add them to internal buffer, then start a singular RMT operation
    /// based on that buffer.
    fn write<T, I>(&mut self, iterator: T) -> impl Future<Output = Result<(), Self::Error>>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        // we split the future into a creation part and a sending part
        // so we can prepare multiple futures and send/await then all at the same time
        let res = self.create_rmt_data(iterator);

        async move {
            res?;
            // Perform the actual RMT operation. We use the u32 values here right away.
            self.channel
                .as_mut()
                .unwrap()
                .transmit(&self.rmt_buffer)
                .await?;
            Ok(())
        }
    }
}

fn convert_colors_to_pulse<C, Order>(
    value: &C,
    mut_iter: &mut IterMut<PulseCode>,
    pulses: (PulseCode, PulseCode),
) -> Result<(), AdapterError>
where
    C: Color,
    Order: ColorOrder<C>,
{
    for channel in 0..C::CHANNELS {
        convert_channel_to_pulses(Order::get_channel_data(&value, channel), mut_iter, pulses)?;
    }

    Ok(())
}

fn convert_channel_to_pulses<N>(
    channel_value: N,
    mut_iter: &mut IterMut<PulseCode>,
    pulses: (PulseCode, PulseCode),
) -> Result<(), AdapterError>
where
    N: Unsigned + Into<usize>,
{
    let channel_value: usize = channel_value.into();
    for index in (0..size_of::<N>() * 8).rev() {
        let position = 1 << index;
        *mut_iter.next().ok_or(AdapterError::BufferSizeExceeded)? = match channel_value & position {
            0 => pulses.0,
            _ => pulses.1,
        }
    }

    Ok(())
}
