//! JSY-MK-194 is hardware to read power of line.
//! Please see [official website](https://jsy-tek.com/products/ac-electric-energy-metering-module/single-phase-2-way-power-metering-module-modbus-ttl-electric-energy-metering-pcba).
//!
//! This crate was inspered by [jsy-mk-194.cpp](https://github.com/clankgeek/JSY-MK-194/blob/main/src/jsy-mk-194.cpp) library.
//!
//! JSY-MK-194 return  data in 61 bytes array:
//!
//! | Order of data | Data               | byte index     |
//! |---------------|--------------------|----------------|
//! |             1 | header of response | 0, 1, 2        |
//! |             2 | voltage1           | 3, 4, 5, 6     |
//! |             3 | current1           | 7, 8, 9, 10    |
//! |             4 | power1             | 11, 12, 13, 14 |
//! |             5 | positive kwh1      | 15, 16, 17, 18 |
//! |             6 | power factor1      | 19, 20, 21, 22 |
//! |             7 | negative kwh1      | 23, 24, 25, 26 |
//! |             8 | negative1          | 27             |
//! |               | negative2          | 28             |
//! |               | not used           | 29, 30         |
//! |             9 | frequency          | 31, 32, 33, 34 |
//! |            10 | voltage2           | 35, 36, 37, 38 |
//! |            11 | current2           | 39, 40, 41, 42 |
//! |            12 | power2             | 43, 44, 45, 46 |
//! |            13 | positive kwh2      | 47, 48, 49, 50 |
//! |            14 | power factor2      | 51, 52, 53, 54 |
//! |            15 | negative kwh2      | 55, 56, 57, 58 |
//! |            16 | crc                | 59, 60         |
//!
use embedded_hal::blocking::delay::DelayMs;

pub mod error;
#[cfg(test)]
mod tests;

// Maximum message to read
const SEGMENT_READ: usize = 64;
/// Maximum size of message to send
const SEGMENT_WRITE: usize = 8;
/// Size of message to read
const READ_DATA_SIZE: usize = 61;
/// Size of message to write change bitrate
const SEGMENT_WRITE_CHANGE_BIT_RATE: usize = 11;

/// Channel 1 offset
const CHANNEL_1_OFFSET: usize = 3;
/// Channel 1 offset
const CHANNEL_2_OFFSET: usize = 35;

/// Voltage position in data
const VOLTAGE: usize = 0;
/// Current position in data
const CURRENT: usize = 4;
/// Power
const POWER: usize = 8;
/// Positive energy
const POSITIVE_ENERGY: usize = 12;
/// Factor
const FACTOR: usize = 16;
/// Negative energy
const NEGATIVE_ENERGY: usize = 20;

/// Frequency position in data
const FREQUENCY: usize = 31;
/// Power sign 1
const POWER_SIGN_1: usize = 27;
/// Power sign 2
const POWER_SIGN_2: usize = 28;

type CrcCheck = fn(&[u8]) -> bool;

// From https://ctlsys.com/support/how_to_compute_the_modbus_rtu_message_crc/
fn is_crc_ok(buf: &[u8]) -> bool {
    let mut crc: u16 = 0xFFFF;
    let low = buf.len() - 2;
    let hi = buf.len() - 1;
    let buf_crc: u16 = (buf[hi] as u16) * 256 + (buf[low] as u16);

    for current_byte in buf.iter().take(buf.len() - 2) {
        crc ^= *current_byte as u16; // XOR byte into least sig. byte of crc

        for _ in (0..8).rev() {
            // Loop over each bit
            if (crc & 0x0001) != 0 {
                // If the LSB is set
                crc >>= 1; // Shift right and XOR 0xA001
                crc ^= 0xA001;
            } else {
                // Else LSB is not set
                crc >>= 1; // Just shift right
            }
        }
    }
    // Note, this number has low and high bytes swapped, so use it accordingly (or swap bytes)
    crc == buf_crc
}

fn crc_always_ok(_buf: &[u8]) -> bool {
    true
}

/// Convert a 32 bits value returned in 4 bytes, to a 32 bit
#[inline(always)]
fn conv8to32(hi_byte: u8, mid_byte_2: u8, mid_byte_1: u8, lo_byte: u8) -> u32 {
    lo_byte as u32
        + ((mid_byte_1 as u32) << 8)
        + ((mid_byte_2 as u32) << 16)
        + ((hi_byte as u32) << 24)
}

/// Get data number X (see crate doc)
fn get_data(segment_read: &[u8; SEGMENT_READ], n: usize) -> u32 {
    conv8to32(
        segment_read[n],
        segment_read[n + 1],
        segment_read[n + 2],
        segment_read[n + 3],
    )
}

/// Get power with right sign.
#[inline(always)]
fn power(segment_read: &[u8; SEGMENT_READ], power: usize, sign: usize) -> f32 {
    let p = (get_data(segment_read, power) as f32) * 0.0001;

    if segment_read[sign] == 1 && p > 0.0 {
        return -p;
    }

    p
}

/// Value to change bitrate
pub enum ChangeBitrate {
    B4800,
    B9600,
    B19200,
    B38400,
}

/// Uart trait that must be impremented for specific hardware
pub trait Uart {
    /// Read multiple bytes into a slice
    fn read(&mut self, buf: &mut [u8], timeout: u32) -> Result<usize, error::UartError>;

    /// Write multiple bytes from a slice
    fn write(&mut self, bytes: &[u8]) -> Result<usize, error::UartError>;
}

/// Channel struct to get information. JSY MK 194 has 2 channels
pub struct Channel {
    data_offset: usize,
    power_sign: usize,
    voltage: f32,
    current: f32,
    positive_energy: f32,
    negative_energy: f32,
    power: f32,
    factor: f32,
}

impl Channel {
    pub fn new(data_offset: usize, power_sign: usize) -> Self {
        Self {
            data_offset,
            power_sign,
            voltage: 0.0,
            current: 0.0,
            positive_energy: 0.0,
            negative_energy: 0.0,
            power: 0.0,
            factor: 0.0,
        }
    }

    /// Return the voltage of first channel in volt.
    pub fn voltage(&self) -> f32 {
        self.voltage
    }

    /// Return current in A of channel.
    pub fn current(&self) -> f32 {
        self.current
    }

    /// Return positive energy in kW/h of channel.
    pub fn positive_energy(&self) -> f32 {
        self.positive_energy
    }

    /// Return negative energy in kW/h of channel.
    pub fn negative_energy(&self) -> f32 {
        self.negative_energy
    }

    /// Return the power of channel in watt.
    pub fn power(&self) -> f32 {
        self.power
    }

    /// Return the power of channel in watt.
    pub fn factor(&self) -> f32 {
        self.factor
    }

    /// Update all data
    fn update(&mut self, segment_read: &[u8; SEGMENT_READ]) {
        self.voltage = (get_data(segment_read, self.data_offset + VOLTAGE) as f32) * 0.0001;
        self.current = (get_data(segment_read, self.data_offset + CURRENT) as f32) * 0.0001;
        self.positive_energy =
            (get_data(segment_read, self.data_offset + POSITIVE_ENERGY) as f32) * 0.0001;
        self.negative_energy =
            (get_data(segment_read, self.data_offset + NEGATIVE_ENERGY) as f32) * 0.0001;
        self.factor = (get_data(segment_read, self.data_offset + FACTOR) as f32) * 0.001;
        self.power = power(segment_read, self.data_offset + POWER, self.power_sign);
    }
}

/// Global struct to communicate with JSY MK 194
pub struct JsyMk194<U, D>
where
    U: Uart,
    D: DelayMs<u16>,
{
    uart: U,
    delay: D,
    segment_write: [u8; SEGMENT_WRITE], //= {0x01, 0x03, 0x00, 0x48, 0x00, 0x0E, 0x44, 0x18};
    segment_read: [u8; SEGMENT_READ],
    is_crc_valid: CrcCheck,
    frequency: f32,

    pub channel1: Channel,
    pub channel2: Channel,
}

impl<U, D> JsyMk194<U, D>
where
    U: Uart,
    D: DelayMs<u16>,
{
    /// Create a new struct of JsyMk194.
    pub fn new(uart: U, delay: D) -> Self {
        Self {
            uart,
            delay,
            segment_write: [0x01, 0x03, 0x00, 0x48, 0x00, 0x0e, 0x44, 0x18],
            segment_read: [0; SEGMENT_READ],
            is_crc_valid: is_crc_ok,
            channel1: Channel::new(CHANNEL_1_OFFSET, POWER_SIGN_1),
            channel2: Channel::new(CHANNEL_2_OFFSET, POWER_SIGN_2),
            frequency: 0.0,
        }
    }

    /// Create a new struct of JsyMk194.
    pub fn new_without_crc_check(uart: U, delay: D) -> Self {
        Self {
            uart,
            delay,
            segment_write: [0x01, 0x03, 0x00, 0x48, 0x00, 0x0e, 0x44, 0x18],
            segment_read: [0; SEGMENT_READ],
            is_crc_valid: crc_always_ok,
            channel1: Channel::new(CHANNEL_1_OFFSET, POWER_SIGN_1),
            channel2: Channel::new(CHANNEL_2_OFFSET, POWER_SIGN_2),
            frequency: 0.0,
        }
    }

    /// Read data.
    pub fn read(&mut self) -> Result<(), error::UartError> {
        // send segment to JSY-MK-194
        self.uart.write(&self.segment_write)?;

        let is_read_data = self.uart.read(&mut self.segment_read, 100);

        match is_read_data {
            Ok(data_size) => {
                if data_size != READ_DATA_SIZE {
                    return Err(error::UartError::new(
                        error::UartErrorKind::ReadInsuffisantBytes,
                        format!(
                            "Try to read {} bytes, but Uart read only {} bytes",
                            READ_DATA_SIZE, data_size
                        ),
                    ));
                }

                if (self.is_crc_valid)(&self.segment_read[0..data_size]) {
                    self.channel1.update(&self.segment_read);
                    self.channel2.update(&self.segment_read);
                    self.frequency = (get_data(&self.segment_read, FREQUENCY) as f32) * 0.01;
                    Ok(())
                } else {
                    Err(error::UartError::from(error::UartErrorKind::BadCrc))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Return frequency in hz.
    pub fn frequency(&self) -> f32 {
        self.frequency
    }

    /// Default bitrate is 4800, you can update the bitrate of module
    /// the available values are : 4800, 9600, 19200, 38400.
    /// Return true if success.
    pub fn change_bitrate(
        &mut self,
        new_bitrate: ChangeBitrate,
    ) -> Result<(), error::ChangeBitrateError> {
        let mut segment: [u8; SEGMENT_WRITE_CHANGE_BIT_RATE] = [
            0x00, 0x10, 0x00, 0x04, 0x00, 0x01, 0x02, 0x01, 0x00, 0x00, 0x00,
        ];

        match new_bitrate {
            ChangeBitrate::B9600 => self.update_segment(&mut segment, 0x06, 0x2b, 0xd6),
            ChangeBitrate::B19200 => self.update_segment(&mut segment, 0x07, 0xea, 0x16),
            ChangeBitrate::B38400 => self.update_segment(&mut segment, 0x08, 0xaa, 0x12),
            _ => self.update_segment(&mut segment, 0x05, 0x6B, 0xD7),
        }

        self.delay.delay_ms(1000);

        let result = self.uart.write(&segment);

        match result {
            Ok(write_size) => {
                if write_size == segment.len() {
                    return Ok(());
                }

                Err(error::ChangeBitrateError::new(error::UartError::new(
                    error::UartErrorKind::WriteInsuffisantBytes,
                    format!(
                        "Try to write {} bytes, but Uart write only {} bytes",
                        segment.len(),
                        write_size
                    ),
                )))
            }
            Err(e) => Err(error::ChangeBitrateError::new(e)),
        }
    }

    fn update_segment(
        &self,
        data: &mut [u8; SEGMENT_WRITE_CHANGE_BIT_RATE],
        value: u8,
        crc1: u8,
        crc2: u8,
    ) {
        data[8] = value;
        data[9] = crc1;
        data[10] = crc2;
    }

    #[cfg(test)]
    fn get_uart(&self) -> &U {
        &self.uart
    }
}
