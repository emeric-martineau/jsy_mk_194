/// JSY-MK-194 is hardware to read power of line.
/// Please see official website here: https://jsy-tek.com/products/ac-electric-energy-metering-module/single-phase-2-way-power-metering-module-modbus-ttl-electric-energy-metering-pcba
/// 
/// Original code from https://github.com/clankgeek/JSY-MK-194/blob/main/src/jsy-mk-194.cpp
///
/// Data return in 61 bytes array:
///   [ 1] header of response 0, 1, 2
///   [ 2] voltage1 = 3, 4, 5, 6
///   [ 3] current1 = 7, 8, 9, 10
///   [ 4] power1 = 11, 12, 13, 14
///   [ 5] positive kwh1 = 15, 16, 17, 18
///   [ 6] power factor1 = 19, 20, 21, 22
///   [ 7] negative kwh1 = 23, 24, 25, 26
///   [ 8] negative1 = 27
///        negative2 = 28
///        not used = 29, 30
///   [ 9] frequency = 31, 32, 33, 34
///   [10] voltage2 = 35, 36, 37, 38
///   [11] current2 = 39, 40, 41, 42
///   [12] power2 = 43, 44, 45, 46
///   [13] positive kwh2 = 47, 48, 49, 50
///   [14] power factor2 = 51, 52, 53, 54
///   [15] negative kwh2 = 55, 56, 57, 58
use embedded_hal::blocking::delay::DelayMs;

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


/// Voltage 1 position in data
const VOLTAGE_1: usize = 3;
/// Voltage 2 position in data
const VOLTAGE_2: usize = 35;
/// Current 1 position in data
const CURRENT_1: usize = 7;
/// Current 1 position in data
const CURRENT_2: usize = 39;
/// Frequency position in data
const FREQUENCY: usize = 31;
/// Positive energy 1
const POSITIVE_ENERGY_1: usize = 15;
/// Positive energy 
const POSITIVE_ENERGY_2: usize = 47;
/// Negative energy 1
const NEGATIVE_ENERGY_1: usize = 23;
/// Negative energy 2
const NEGATIVE_ENERGY_2: usize = 55;
/// Power 1
const POWER_1: usize = 11;
/// Power 2
const POWER_2: usize = 43;
/// Power sign 1
const POWER_SIGN_1: usize = 27;
/// Power sign 2
const POWER_SIGN_2: usize = 28;
/// Factor 1
const FACTOR_1: usize = 19;
/// Factor 2
const FACTOR_2: usize = 51;


/// Value to change bitrate
pub enum ChangeBitrate {
  B4800,
  B9600,
  B19200,
  B38400
}

pub trait Uart {
    type Error;

    /// Read multiple bytes into a slice
    fn read(&mut self, buf: &mut [u8], timeout: u32) -> Result<usize, Self::Error>;
    
    /// Write multiple bytes from a slice
    fn write(&mut self, bytes: &[u8]) -> Result<usize, Self::Error>;
}

pub struct JsyMk194<U, D> 
where
    U: Uart,
    D: DelayMs<u16>
{
    uart: U,
    delay: D,
    segment_write: [u8; SEGMENT_WRITE], //= {0x01, 0x03, 0x00, 0x48, 0x00, 0x0E, 0x44, 0x18};
    segment_read: [u8; SEGMENT_READ]
}

impl<U, D> JsyMk194<U, D> 
where
    U: Uart,
    D: DelayMs<u16>
{
    /// Create a new struct of JsyMk194.
    pub fn new(uart: U, delay: D) -> Self {
      JsyMk194 {
        uart,
        delay,
        segment_write: [0x01, 0x03, 0x00, 0x48, 0x00, 0x0e, 0x44, 0x18],
        segment_read: [0; SEGMENT_READ]
      }
    }

    /// Read data.
    pub fn read(&mut self) -> bool {     
        // send segment to JSY-MK-194
        let write_result = self.uart.write(&self.segment_write);

        if write_result.is_err() {
          return false;
        }        

        let is_read_data = self.uart.read(&mut self.segment_read, 100);

        match is_read_data {
          Ok(data_size) => {
            if data_size != READ_DATA_SIZE {
              return false;
            }

            // TODO check CRC ?
          },
          Err(_) => return false
        }

        return true;
    }

    /// Return the voltage of first channel in volt.
    pub fn voltage_1(&self) -> f32 {
        (self.get_data(VOLTAGE_1) as f32) * 0.0001
    }

    /// Return the voltage of second channel in volt.
    pub fn voltage_2(&self) -> f32 {
        (self.get_data(VOLTAGE_2) as f32) * 0.0001
    }

    /// Return frequency in hz.
    pub fn frequency(&self) -> f32 {
        (self.get_data(FREQUENCY) as f32) * 0.01
    }

    /// Return current in A of channel 1.
    pub fn current_1(&self) -> f32 { 
        (self.get_data(CURRENT_1) as f32) * 0.0001
    }

    /// Return current in A of channel 2.
    pub fn current_2(&self)-> f32 {
        (self.get_data(CURRENT_2) as f32) * 0.0001
    }

    /// Return positive energy in kW/h of channel 1.
    pub fn positive_energy_1(&self)-> f32 {
        (self.get_data(POSITIVE_ENERGY_1) as f32) * 0.0001
    }

    /// Return negative energy in kW/h of channel 1.
    pub fn negative_energy_1(&self)-> f32 {
        (self.get_data(NEGATIVE_ENERGY_1) as f32) * 0.0001
    }

    /// Return positive energy in kW/h of channel 2.
    pub fn positive_energy_2(&self)-> f32 {
        (self.get_data(POSITIVE_ENERGY_2) as f32) * 0.0001
    }

    /// Return negative energy in kW/h of channel 2.
    pub fn negative_energy_2(&self)-> f32 {
        (self.get_data(NEGATIVE_ENERGY_2) as f32) * 0.0001
    }

    /// Return the power of channel 1 in watt.
    pub fn power_1(&self) -> f32 {
       self.power(POWER_1, POWER_SIGN_1)
    }

    /// Return the power of channel 1 in watt.
    pub fn power_2(&self) -> f32 {
       self.power(POWER_2, POWER_SIGN_2)
    }

    /// Return the power of channel 1 in watt.
    pub fn factor_1(&self) -> f32 {
        (self.get_data(FACTOR_1) as f32) * 0.001
     }
 
     /// Return the power of channel 1 in watt.
     pub fn factor_2(&self) -> f32 {
        (self.get_data(FACTOR_2) as f32) * 0.001
     }

    /// Default bitrate is 4800, you can update the bitrate of module
    /// the available values are : 4800, 9600, 19200, 38400.
    /// Return true if success.
    pub fn change_bitrate(&mut self, new_bitrate: ChangeBitrate) -> bool {
        let mut segment: [u8; SEGMENT_WRITE_CHANGE_BIT_RATE] = [0x00, 0x10, 0x00, 0x04, 0x00, 0x01,
                                     0x02, 0x01, 0x00, 0x00, 0x00];

        match new_bitrate {
          ChangeBitrate::B9600 => self.update_segment(&mut segment, 0x06, 0x2b, 0xd6),
          ChangeBitrate::B19200 => self.update_segment(&mut segment, 0x07, 0xea, 0x16),
          ChangeBitrate::B38400 => self.update_segment(&mut segment, 0x08, 0xaa, 0x12),
          _ => self.update_segment(&mut segment, 0x05, 0x6B, 0xD7)
        }

        self.delay.delay_ms(1000);

        let result = self.uart.write(&segment);

        match result {
          Ok(write_size) => write_size == segment.len(),
          Err(_) => false
        }
    }

    /// Convert a 32 bits value returned in 4 bytes, to a 32 bit
    fn conv8to32(&self, hi_byte: u8, mid_byte_2: u8, mid_byte_1: u8, lo_byte: u8) -> u32 {
        lo_byte as u32 + ((mid_byte_1 as u32) << 8) + ((mid_byte_2 as u32) << 16) + ((hi_byte as u32) << 24)
    }
  
    /// Get data number X (see crate doc)
    fn get_data(&self, n: usize) -> u32 {
        self.conv8to32(
        self.segment_read[n], 
        self.segment_read[n + 1],
        self.segment_read[n + 2],
        self.segment_read[n + 3])
    }

    /// Get power with right sign.
    fn power(&self, power: usize, sign: usize) -> f32 {
        let p = (self.get_data(power) as f32) * 0.0001;

        if self.segment_read[sign] == 1 && p > 0.0 {
          return -p;
        }

        return p;
    }

    fn update_segment(&self, data: &mut [u8; SEGMENT_WRITE_CHANGE_BIT_RATE], value: u8, crc1: u8, crc2: u8) {
        data[8] = value;
        data[9] = crc1;
        data[10] = crc2;
    }

    #[cfg(test)]
    fn get_uart(&self) -> &U {
        &self.uart
    }
}
