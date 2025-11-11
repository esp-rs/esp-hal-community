//! [`embedded_graphics`](`embedded_graphics_core`) driver implementation using the smart LED driver.
//! This allows you to use addressable LEDs in some matrix arrangement as a 2D display with the [`embedded_graphics`](`embedded_graphics_core`) ecosystem.
//!
//! For usage details, see [`RmtSmartLedsGraphics`].

use embedded_graphics_core::{Pixel, pixelcolor::Rgb888, prelude::*, primitives::Rectangle};

use super::*;

/// [`embedded_graphics`](`embedded_graphics_core`) display driver based on the ESP32 “remote control” (RMT) peripheral.
/// Use this to build 2D displays using smart LEDs.
/// You need to enable the `embedded-graphics` feature of the crate to make this struct available.
///
/// # Generic arguments
///
/// For runtime efficiency, the driver is almost entirely configured using generics.
/// This allows the compiler to generate a highly optimized implementation.
/// The available generics are:
/// - Most generics of [`RmtSmartLeds`]: `BUFFER_SIZE`, `Color`, `Order`, `Timing`.
///   Choose `BUFFER_SIZE` with [`buffer_size_2d`], passing in width and height.
///   The buffer size can however be bigger than the number of pixels (`W`*`H`).
///   The `Mode` has to be blocking due to embedded-graphics restrictions.
/// - `W`: Width of the 2D panel.
/// - `H`: Height of the 2D panel.
/// - `SNAKING`: Whether the panel is laid out in snaking order, where each odd line is reversed (right-to-left),
///   or normally, where each line is laid out left-to-right.
pub struct RmtSmartLedsGraphics<
    'd,
    C,
    Order,
    Timing,
    const BUFFER_SIZE: usize,
    const W: usize,
    const H: usize,
    const SNAKING: bool = false,
> where
    C: Color,
    Order: ColorOrder<C>,
    Timing: crate::Timing,
{
    // FIXME: BUFFER_SIZE type should really just be `{ W * H * ( size per pixel ) }` here once someone at Rust has the fucking dignity to stabilize generic-const-exprs already.
    driver: RmtSmartLeds<'d, BUFFER_SIZE, Blocking, C, Order, Timing>,
}

impl<
    'd,
    C,
    Order,
    Timing,
    const BUFFER_SIZE: usize,
    const W: usize,
    const H: usize,
    const SNAKING: bool,
> RmtSmartLedsGraphics<'d, C, Order, Timing, BUFFER_SIZE, W, H, SNAKING>
where
    C: Color,
    Order: ColorOrder<C>,
    Timing: crate::Timing,
{
    /// Number of LEDs in this panel.
    pub const LED_COUNT: usize = W * H;

    /// Create a new 2D display driver with the given output pin and RMT channel.
    ///
    /// See [`RmtSmartLeds::new`] for further information.
    pub fn new<Ch, P>(channel: Ch, pin: P) -> Result<Self, crate::RmtError>
    where
        Ch: TxChannelCreator<'d, Blocking>,
        P: PeripheralOutput<'d>,
    {
        Self::new_with_memsize(channel, pin, 1)
    }

    /// Create a new 2D display driver with the given output pin and RMT channel.
    /// Additionally, configure the provided number of DMA memory channels.
    ///
    /// See [`RmtSmartLeds::new_with_memsize`] for further information.
    pub fn new_with_memsize<Ch, P>(
        channel: Ch,
        pin: P,
        memsize: u8,
    ) -> Result<Self, crate::RmtError>
    where
        Ch: TxChannelCreator<'d, Blocking>,
        P: PeripheralOutput<'d>,
    {
        Ok(Self {
            driver: RmtSmartLeds::new_with_memsize(channel, pin, memsize)?,
        })
    }

    /// Convert cartesian coordinates to linear strip index.
    fn coordinate_to_index(x: usize, y: usize) -> usize {
        // every odd row, x is reversed
        let x = if SNAKING && y.is_multiple_of(2) {
            W - x - 1
        } else {
            x
        };
        x + y * W
    }

    /// Update the matrix by transmitting it over the RMT peripheral.
    pub fn flush(&mut self) -> Result<(), AdapterError> {
        self.driver.flush()
    }
}

impl<
    'd,
    C,
    Order,
    Timing,
    const BUFFER_SIZE: usize,
    const W: usize,
    const H: usize,
    const SNAKING: bool,
> Dimensions for RmtSmartLedsGraphics<'d, C, Order, Timing, BUFFER_SIZE, W, H, SNAKING>
where
    C: Color,
    Order: ColorOrder<C>,
    Timing: crate::Timing,
{
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(Point::zero(), Size::new(W as u32, H as u32))
    }
}

/// Convert from embedded-graphics 8-bit RGB to smart-leds 8-bit RGB.
fn rgb888_to_rgb8(v: Rgb888) -> RGB8 {
    RGB {
        r: v.r(),
        g: v.g(),
        b: v.b(),
    }
}

// TODO: similar implementation for grayscale, which could then be used for a white-only (1-channel) strip
impl<
    'd,
    Order,
    Timing,
    const BUFFER_SIZE: usize,
    const W: usize,
    const H: usize,
    const SNAKING: bool,
> DrawTarget for RmtSmartLedsGraphics<'d, RGB8, Order, Timing, BUFFER_SIZE, W, H, SNAKING>
where
    Order: ColorOrder<RGB8>,
    Timing: crate::Timing,
{
    // not exactly our own color type (from `rgb`), but fully compatible with it
    type Color = Rgb888;
    type Error = crate::AdapterError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics_core::Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            // ignore out-of-range pixels
            let Ok(x) = coord.x.try_into() else {
                continue;
            };
            let Ok(y) = coord.y.try_into() else {
                continue;
            };
            if x >= W || y >= H {
                continue;
            }

            let index = Self::coordinate_to_index(x, y);
            self.driver.write_pixel_data(index, rgb888_to_rgb8(color))?;
        }
        Ok(())
    }
}

/// Calculate the appropriate `BUFFER_SIZE` for a [`RmtSmartLedsGraphics`] of a certain width and height.
pub const fn buffer_size_2d<C: Color>(width: usize, height: usize) -> usize {
    crate::buffer_size::<C>(width * height)
}

/// [`RmtSmartLedsGraphics`] with snaking LED strip layout.
///
/// “Snaking” means that every second line of LEDs is reversed compared to the previous.
/// When using LED strips to create matrices, this is a common configuration, as it greatly simplifies the wiring.
/// As usual, the first row is assumed to be oriented normally, i.e. left-to-right.
pub type SnakingRmtSmartLedsGraphics<
    'd,
    C,
    Order,
    Timing,
    const BUFFER_SIZE: usize,
    const W: usize,
    const H: usize,
> = RmtSmartLedsGraphics<'d, C, Order, Timing, BUFFER_SIZE, W, H, true>;
