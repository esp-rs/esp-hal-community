//! # Buzzer
//!
//! ## Overview
//!
//! This driver provides an abstraction over LEDC to drive a piezo-electric
//! buzzer through a user-friendly API.
//!
//! The [buzzer example](https://github.com/esp-rs/esp-hal-community/blob/main/esp-hal-buzzer/examples/buzzer.rs)
//! contains pre-programmed songs to play through the buzzer.
//!
//! ## Example
//!
//! ```rust,ignore
//! let peripherals = esp_hal::init(esp_hal::Config::default());
//!
//! let mut ledc = Ledc::new(peripherals.LEDC);
//! ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
//!
//! let mut buzzer = Buzzer::new(
//!     &ledc,
//!     timer::Number::Timer0,
//!     channel::Number::Channel1,
//!     peripherals.GPIO4,
//! );
//!
//! // Play a 1000Hz frequency
//! buzzer.play(1000).unwrap()
//! ```
//!
//! ## Feature Flags
#![doc = document_features::document_features!()]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/46717278")]
#![deny(missing_docs)]
#![no_std]

use core::fmt::Debug;

use esp_hal::{
    clock::Clocks,
    delay::Delay,
    gpio::{AnyPin, DriveMode, Level, Output, OutputConfig, OutputPin},
    ledc::{
        Ledc, LowSpeed,
        channel::{self, Channel, ChannelIFace},
        timer::{self, Timer, TimerIFace},
    },
    time::Rate,
};

pub mod notes;

/// Errors from Buzzer
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Errors from [channel::Error]
    Channel(channel::Error),

    /// Errors from [timer::Error]
    Timer(timer::Error),

    /// Error when the volume pin isn't set and we try to use it
    VolumeNotSet,

    /// When the volume level is out of range. Either too low or too high.
    VolumeOutOfRange,

    /// Sequence and timings slice aren't of the same length
    LengthMismatch,
}

/// Converts [channel::Error] into [self::Error]
impl From<channel::Error> for Error {
    fn from(error: channel::Error) -> Self {
        Error::Channel(error)
    }
}

/// Converts [timer::Error] into [self::Error]
impl From<timer::Error> for Error {
    fn from(error: timer::Error) -> Self {
        Error::Timer(error)
    }
}

/// Represents a tone value to play through the buzzer
pub struct ToneValue {
    /// Frequency of the tone in Hz
    /// *Use 0 for a silent tone*
    pub frequency: u32,

    /// Duration for the frequency in ms
    pub duration: u32,
}

/// Represents different volume strategies for the buzzer.
///
/// - [VolumeType::OnOff] is a simple on or off volume. It's similar as using
///   `.mute()` except that the volume control is on a second pin independent of
///   the buzzer.
///
/// - [VolumeType::Duty] uses the duty as the volume control. It acts like a PWM
///   by switching the power on and off. This may require extra logic gates in
///   the circuit.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum VolumeType {
    /// An On / Off based volume
    OnOff,

    /// A duty based volume where 0% is the lowest and 100% the highest.
    Duty,
}

/// Volume configuration for the buzzer
struct Volume<'d> {
    /// Output pin for the volume
    volume_pin: AnyPin<'d>,

    /// Type of the volume
    volume_type: VolumeType,

    /// Volume level
    ///
    /// For [VolumeType::OnOff], should be 0 for Off, or 1 or more for On
    /// For [VolumeType::Duty], should be between 0 and 100.
    level: u8,
}

/// A buzzer instance driven by Ledc
pub struct Buzzer<'a> {
    timer: Timer<'a, LowSpeed>,
    channel_number: channel::Number,
    output_pin: AnyPin<'a>,
    delay: Delay,
    volume: Option<Volume<'a>>,
}

impl<'a> Buzzer<'a> {
    /// Create a new buzzer for the given pin
    pub fn new(
        ledc: &'a Ledc,
        timer_number: timer::Number,
        channel_number: channel::Number,
        output_pin: impl OutputPin + 'a,
    ) -> Self {
        let timer = ledc.timer(timer_number);
        Self {
            timer,
            channel_number,
            output_pin: output_pin.degrade(),
            delay: Delay::new(),
            volume: None::<Volume>,
        }
    }

    /// Add a volume control for the buzzer.
    pub fn with_volume(mut self, volume_pin: impl OutputPin + 'a, volume_type: VolumeType) -> Self {
        self.volume = Some(Volume {
            volume_pin: volume_pin.degrade(),
            volume_type,
            level: 50,
        });

        self
    }

    /// Set the volume of the buzzer
    ///
    /// For [VolumeType::Duty], the level should be between 0 and 100.
    /// For [VolumeType::OnOff], it will only be mute on 0 and playing on 1 or
    /// more
    pub fn set_volume(&mut self, level: u8) -> Result<(), Error> {
        if let Some(ref mut volume) = self.volume {
            match volume.volume_type {
                VolumeType::OnOff => {
                    // Only turn off when level is set to 0, else set to high
                    Output::new(
                        unsafe { volume.volume_pin.clone_unchecked() },
                        if level != 0 { Level::High } else { Level::Low },
                        OutputConfig::default(),
                    );
                    Ok(())
                }
                VolumeType::Duty => {
                    match level {
                        0..=99 => {
                            volume.level = level;

                            // Put a dummy config in the timer if it's not already configured
                            if !self.timer.is_configured() {
                                self.timer.configure(timer::config::Config {
                                    duty: timer::config::Duty::Duty11Bit,
                                    clock_source: timer::LSClockSource::APBClk,
                                    frequency: Rate::from_hz(20_000),
                                })?;
                            }

                            let mut channel = Channel::new(self.channel_number, unsafe {
                                volume.volume_pin.clone_unchecked()
                            });
                            channel
                                .configure(channel::config::Config {
                                    timer: &self.timer,
                                    duty_pct: level,
                                    drive_mode: DriveMode::PushPull,
                                })
                                .map_err(|e| e.into())
                        }
                        100 => {
                            // If level is 100, we just keep the pin high
                            Output::new(
                                unsafe { volume.volume_pin.clone_unchecked() },
                                Level::High,
                                OutputConfig::default(),
                            );
                            Ok(())
                        }
                        _ => Err(Error::VolumeOutOfRange),
                    }
                }
            }
        } else {
            Err(Error::VolumeNotSet)
        }
    }

    /// Mute the buzzer
    ///
    /// The muting is done by simply setting the duty to 0
    pub fn mute(&self) {
        // Timer is not configured, we do an early return since no sound is playing anyway.
        if !self.timer.is_configured() {
            return;
        }
        let mut channel = Channel::new(self.channel_number, unsafe {
            self.output_pin.clone_unchecked()
        });
        // Safety:
        // - Error::Duty cannot happen, we hardcode 0 which is valid
        // - Error::Channel cannot happen, channel is configured below
        // - Error::Timer cannot happen, it's an early return no-op above
        channel
            .configure(channel::config::Config {
                timer: &self.timer,
                duty_pct: 0,
                drive_mode: DriveMode::PushPull,
            })
            .unwrap()
    }

    /// Play a frequency through the buzzer
    pub fn play(&mut self, frequency: u32) -> Result<(), Error> {
        // Mute if frequency is 0Hz
        if frequency == 0 {
            self.mute();
            return Ok(());
        }

        // Max duty resolution for a frequency:
        // Integer(log2(LEDC_APB_CKL / frequency))
        let mut result = 0;
        let mut value = Clocks::get().apb_clock / Rate::from_hz(frequency);

        // Limit duty resolution to 14 bits
        while value > 1 && result < 14 {
            value >>= 1;
            result += 1;
        }

        self.timer.configure(timer::config::Config {
            // Safety: This should never fail because resolution is limited to 14 bits
            duty: timer::config::Duty::try_from(result).unwrap(),
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_hz(frequency),
        })?;

        let mut channel = Channel::new(self.channel_number, unsafe {
            self.output_pin.clone_unchecked()
        });
        channel.configure(channel::config::Config {
            timer: &self.timer,
            // Use volume as duty if set since we use the same channel.
            duty_pct: self.volume.as_ref().map_or(50, |v| v.level),
            drive_mode: DriveMode::PushPull,
        })?;

        Ok(())
    }

    /// Play a sound sequence through the buzzer
    ///
    /// Uses a pair of frequencies and timings to play a sound sequence.
    ///
    /// # Arguments
    /// * `sequence` - A list of frequencies to play through the buzzer
    /// * `timings` - A list of timings in ms for each frequencies
    ///
    /// # Examples
    /// Play a single beep at 300Hz for 1 second
    /// ```
    /// buzzer.play_tones([300], [1000]);
    /// ```
    ///
    /// Play a sequence of 3 beeps with a break inbetween
    /// ```
    /// buzzer.play_tones([200, 0, 200, 0, 200], [200, 50, 200, 50, 200]);
    /// ```
    ///
    /// Play a sequence of 3 beeps with the same duration
    /// ```
    /// buzzer.play_tones([100, 200, 300], [100; 3]);
    /// ```
    ///
    /// # Errors
    /// This function returns an [Error] in case of an error.
    /// An error can occur when an invalid value is used as a tone
    pub fn play_tones<const T: usize>(
        &mut self,
        sequence: [u32; T],
        timings: [u32; T],
    ) -> Result<(), Error> {
        // Iterate for each frequency / timing pair
        for (frequency, timing) in sequence.iter().zip(timings.iter()) {
            self.play(*frequency)?;
            self.delay.delay_millis(*timing);
            self.mute();
        }
        // Mute at the end of the sequence
        self.mute();
        Ok(())
    }

    /// Play a sound sequence through the buzzer
    ///
    /// Uses a pair of frequency and duration slices to play a sound sequence.
    /// Both slices must be of the same length, where each pair of `(frequency, duration)` defines one tone.
    ///
    /// # Arguments
    /// * `sequence` - A slice of frequencies to play through the buzzer
    /// * `timings` - A slice of durations in milliseconds for each frequency
    ///
    /// # Examples
    /// Play a single beep at 300Hz for 1 second
    /// ```
    /// buzzer.play_tones_from_slice(&[300], &[1000]);
    /// ```
    ///
    /// Play a sequence of 3 beeps with a break in between
    /// ```
    /// buzzer.play_tones_from_slice(&[200, 0, 200, 0, 200], &[200, 50, 200, 50, 200]);
    /// ```
    ///
    /// Play a sequence of 3 beeps with the same duration
    /// ```
    /// buzzer.play_tones_from_slice(&[100, 200, 300], &[100; 3]);
    /// ```
    ///
    /// # Errors
    /// This function returns an [Error] in the following cases:
    /// - If the `sequence` and `timings` slices have different lengths ([Error::LengthMismatch])
    /// - If playing a frequency results in an error
    pub fn play_tones_from_slice(
        &mut self,
        sequence: &[u32],
        timings: &[u32],
    ) -> Result<(), Error> {
        if sequence.len() != timings.len() {
            return Err(Error::LengthMismatch);
        }

        // Iterate for each frequency / timing pair
        for (frequency, timing) in sequence.iter().zip(timings.iter()) {
            self.play(*frequency)?;
            self.delay.delay_millis(*timing);
            self.mute();
        }
        // Mute at the end of the sequence
        self.mute();
        Ok(())
    }

    /// Play a tone sequence through the buzzer
    ///
    /// Uses a pair of frequencies and timings to play a sound sequence.
    ///
    /// # Arguments
    /// * `tones` - A list of type [ToneValue] to play through the buzzer
    ///
    /// # Examples
    /// Play a tone sequence
    /// ```
    /// let song = [
    ///     ToneValue {
    ///         frequency: 100,
    ///         duration: 100,
    ///     },
    ///     ToneValue {
    ///         frequency: 200,
    ///         duration: 100,
    ///     },
    ///     ToneValue {
    ///         frequency: 300,
    ///         duration: 100,
    ///     },
    /// ];
    /// buzzer.play_song(&song);
    /// ```
    ///
    /// # Errors
    /// This function returns an [Error] in case of an error.
    /// An error can occur when an invalid value is used as a tone
    pub fn play_song(&mut self, tones: &[ToneValue]) -> Result<(), Error> {
        for tone in tones {
            self.play(tone.frequency)?;
            self.delay.delay_millis(tone.duration);
            self.mute();
        }
        // Mute at the end of the sequence
        self.mute();
        Ok(())
    }
}
