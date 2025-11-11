//! [`embedded_graphics`](`embedded_graphics_core`) driver implementation using the smart LED driver.
//! This allows you to use addressable LEDs in some matrix arrangement as a 2D display with the [`embedded_graphics`](`embedded_graphics_core`) ecosystem.
//!

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
///   Choose `BUFFER_SIZE` with [`buffer_size_2d`].
///   The buffer size can be bigger than the number of pixels (`W`*`H`).
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
    /// Create a new 2D display driver with the given output pin and RMT channel.
    /// 
    /// See [`RmtSmartLeds::new`] for further information.
    pub fn new<Ch, P>(channel: Ch, pin: P) -> Result<Self, crate::RmtError>
    where
        Ch: TxChannelCreator<'d, Blocking>,
        P: PeripheralOutput<'d>,
    {
        Ok(Self {
            driver: RmtSmartLeds::new(channel, pin)?,
        })
    }
}

/// Calculate the appropriate `BUFFER_SIZE` for
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
