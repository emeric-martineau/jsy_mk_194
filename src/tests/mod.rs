use embedded_hal::blocking::delay::DelayMs;

/// When put this data in segment_read, Uart.read() return Ok
const READ_DATA_OK: [u8; crate::READ_DATA_SIZE] = [
    0x01, 0x03, 0x38, 0x00, 0x24, 0x8E, 0x5F, 0x00, 0x00, 0x94, 0x0B, 0x00, 0x8A, 0xC4, 0xAA, 0x00,
    0x00, 0x2D, 0xB4, 0x00, 0x00, 0x03, 0xE8, 0x00, 0x00, 0x26, 0x16, 0x01, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x13, 0x8B, 0x00, 0x24, 0x8E, 0x5F, 0x00, 0x00, 0x94, 0x0A, 0x00, 0x8A, 0x92, 0xD7, 0x00,
    0x00, 0x2D, 0xB4, 0x00, 0x00, 0x03, 0xE8, 0x00, 0x00, 0x26, 0x20, 0x87, 0xB2,
];

/// When put this data in segment_read, Uart.read() return Ok
const READ_DATA_OK_2: [u8; crate::READ_DATA_SIZE] = [
    0x01, 0x03, 0x38, 0x00, 0x23, 0xCF, 0x24, 0x00, 0x00, 0x03, 0xFD, 0x00, 0x02, 0x1F, 0x95, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x45, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x13, 0x89, 0x00, 0x23, 0xCF, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x79, 0xB0,
];

/// When put this data in segment_read, JsyMk194Hardware.read() return error, bad CRC
const READ_DATA_BAD_CRC: [u8; crate::READ_DATA_SIZE] = [
    0x01, 0x03, 0x38, 0x00, 0x24, 0x8E, 0x5F, 0x00, 0x00, 0x94, 0x0B, 0x00, 0x8A, 0xC4, 0xAA, 0x00,
    0x00, 0x2D, 0xB4, 0x00, 0x00, 0x03, 0xE8, 0x00, 0x00, 0x26, 0x16, 0x01, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x13, 0x8B, 0x00, 0x24, 0x8E, 0x5F, 0x00, 0x00, 0x94, 0x0A, 0x00, 0x8A, 0x92, 0xD7, 0x00,
    0x00, 0x2D, 0xB4, 0x00, 0x00, 0x03, 0xE8, 0x00, 0x00, 0x26, 0x20, 0x99, 0x99,
];

/// When put this data in segment_read, Uart.read() return Err
const READ_DATA_ERROR: [u8; crate::READ_DATA_SIZE] = [0xff; crate::READ_DATA_SIZE];

/// When put this data in segment_read, Uart.read() return wrong siaz
const READ_DATA_WRONG_SIZE: [u8; crate::READ_DATA_SIZE] = [0x01; crate::READ_DATA_SIZE];

/// When put this data in segment_write, Uart.write() return Ok
const WRITE_DATA_OK: [u8; crate::SEGMENT_WRITE_CHANGE_BIT_RATE] =
    [0; crate::SEGMENT_WRITE_CHANGE_BIT_RATE];

/// When put this data in segment_write, Uart.write() return Err
const WRITE_DATA_ERROR: [u8; crate::SEGMENT_WRITE_CHANGE_BIT_RATE] =
    [0xff; crate::SEGMENT_WRITE_CHANGE_BIT_RATE];

struct UartTestImpl {
    pub segment_write: [u8; crate::SEGMENT_WRITE_CHANGE_BIT_RATE],
    pub segment_write_len: usize,
    pub segment_read: [u8; crate::READ_DATA_SIZE],
}

impl crate::Uart for UartTestImpl {
    fn read(&mut self, buf: &mut [u8], _timeout: u32) -> Result<usize, crate::error::UartError> {
        match self.segment_read {
            READ_DATA_WRONG_SIZE => Ok(1),
            READ_DATA_ERROR => Err(crate::error::UartError::new(
                crate::error::UartErrorKind::ReadInsuffisantBytes,
                "Error read".to_string(),
            )),
            _ => {
                for (index, data) in self.segment_read.into_iter().enumerate() {
                    buf[index] = data;
                }

                Ok(self.segment_read.len())
            }
        }
    }

    fn write(&mut self, bytes: &[u8]) -> Result<usize, crate::error::UartError> {
        self.segment_write_len = bytes.len();

        if self.segment_write == WRITE_DATA_ERROR {
            return Err(crate::error::UartError::new(
                crate::error::UartErrorKind::WriteInsuffisantBytes,
                "Error write".to_string(),
            ));
        }

        for (index, data) in bytes.iter().enumerate() {
            self.segment_write[index] = *data;
        }

        Ok(bytes.len())
    }
}

struct DelayTestImpl {}

impl DelayMs<u16> for DelayTestImpl {
    fn delay_ms(&mut self, _ms: u16) {
        // Do nothing
    }
}

fn setup(
    read_data: [u8; crate::READ_DATA_SIZE],
    write_data: [u8; crate::SEGMENT_WRITE_CHANGE_BIT_RATE],
) -> crate::JsyMk194<UartTestImpl, DelayTestImpl> {
    let uart = UartTestImpl {
        segment_write: write_data,
        segment_read: read_data,
        segment_write_len: 0,
    };

    let delay = DelayTestImpl {};
    crate::JsyMk194::new(uart, delay)
}

#[test]
fn test_read_ok() {
    let mut device = setup(READ_DATA_OK, WRITE_DATA_OK);

    assert!(device.read().is_ok());

    let voltage_1 = device.channel1.voltage();
    let current_1 = device.channel1.current();
    let power_1 = device.channel1.power();
    let positive_energy_1 = device.channel1.positive_energy();
    let negative_energy_1 = device.channel1.negative_energy();
    let factor_1 = device.channel1.factor();

    let voltage_2 = device.channel2.voltage();
    let current_2 = device.channel2.current();
    let power_2 = device.channel2.power();
    let positive_energy_2 = device.channel2.positive_energy();
    let negative_energy_2 = device.channel2.negative_energy();
    let factor_2 = device.channel2.factor();

    let frequency = device.frequency();

    assert_eq!(voltage_1, 239.574_3);
    assert_eq!(current_1, 3.789_899_8);
    assert_eq!(power_1, -909.431_4);
    assert_eq!(positive_energy_1, 1.17);
    assert_eq!(negative_energy_1, 0.974_999_96);
    assert_eq!(factor_1, 1.0);

    assert_eq!(voltage_2, 239.574_3);
    assert_eq!(current_2, 3.789_8);
    assert_eq!(power_2, -908.155_9);
    assert_eq!(positive_energy_2, 1.17);
    assert_eq!(negative_energy_2, 0.975_999_95);
    assert_eq!(factor_2, 1.0);

    assert_eq!(frequency, 50.03);
}

#[test]
fn test_read_ok_2() {
    let mut device = setup(READ_DATA_OK_2, WRITE_DATA_OK);

    assert!(device.read().is_ok());

    let voltage_1 = device.channel1.voltage();
    let current_1 = device.channel1.current();
    let power_1 = device.channel1.power();
    let positive_energy_1 = device.channel1.positive_energy();
    let negative_energy_1 = device.channel1.negative_energy();
    let factor_1 = device.channel1.factor();

    let voltage_2 = device.channel2.voltage();
    let current_2 = device.channel2.current();
    let power_2 = device.channel2.power();
    let positive_energy_2 = device.channel2.positive_energy();
    let negative_energy_2 = device.channel2.negative_energy();
    let factor_2 = device.channel2.factor();

    let frequency = device.frequency();

    assert_eq!(voltage_1, 234.678_79);
    assert_eq!(current_1, 0.1021);
    assert_eq!(power_1, -13.9157);
    assert_eq!(positive_energy_1, 0.0);
    assert_eq!(negative_energy_1, 0.0);
    assert_eq!(factor_1, 0.58100003);

    assert_eq!(voltage_2, 234.678_79);
    assert_eq!(current_2, 0.0);
    assert_eq!(power_2, 0.0);
    assert_eq!(positive_energy_2, 0.0);
    assert_eq!(negative_energy_2, 0.0);
    assert_eq!(factor_2, 0.0);

    assert_eq!(frequency, 50.01);
}

#[test]
fn test_jsk_mk_196_read_method_return_error_cause_write_to_device_return_error() {
    let mut device = setup(READ_DATA_OK, WRITE_DATA_ERROR);

    match device.read() {
        Ok(()) => panic!(),
        Err(e) => assert_eq!(e.kind, crate::error::UartErrorKind::WriteInsuffisantBytes),
    };
}

#[test]
fn test_jsk_mk_196_read_method_return_error_cause_read_to_device_return_error() {
    let mut device = setup(READ_DATA_ERROR, WRITE_DATA_OK);

    match device.read() {
        Ok(()) => panic!(),
        Err(e) => assert_eq!(e.kind, crate::error::UartErrorKind::ReadInsuffisantBytes),
    };
}

#[test]
fn test_jsk_mk_196_read_method_return_error_cause_read_to_device_return_wrong_size() {
    let mut device = setup(READ_DATA_WRONG_SIZE, WRITE_DATA_OK);

    match device.read() {
        Ok(()) => panic!(),
        Err(e) => assert_eq!(e.kind, crate::error::UartErrorKind::ReadInsuffisantBytes),
    };
}

#[test]
fn test_jsk_mk_196_change_bitrate_method_return_true() {
    let mut device = setup(READ_DATA_OK, WRITE_DATA_OK);

    assert!(device.change_bitrate(crate::ChangeBitrate::B9600).is_ok());

    assert_eq!(
        device.get_uart().segment_write_len,
        crate::SEGMENT_WRITE_CHANGE_BIT_RATE
    );
    assert_eq!(
        device.get_uart().segment_write,
        [0x00, 0x10, 0x00, 0x04, 0x00, 0x01, 0x02, 0x01, 0x06, 0x2b, 0xd6]
    );
}

#[test]
fn test_crc() {
    assert!(crate::is_crc_ok(&READ_DATA_OK));
    assert!(crate::is_crc_ok(&READ_DATA_OK_2));
}

#[test]
fn test_jsk_mk_196_read_method_return_error_cause_read_to_device_return_bad_crc() {
    let mut device = setup(READ_DATA_BAD_CRC, WRITE_DATA_OK);

    match device.read() {
        Ok(()) => panic!(),
        Err(e) => assert_eq!(e.kind, crate::error::UartErrorKind::BadCrc),
    };
}

#[test]
fn test_jsk_mk_196_read_method_return_ok_when_read_to_device_return_bad_crc() {
    let uart = UartTestImpl {
        segment_write: WRITE_DATA_OK,
        segment_read: READ_DATA_BAD_CRC,
        segment_write_len: 0,
    };

    let delay = DelayTestImpl {};

    let mut device = crate::JsyMk194::new_without_crc_check(uart, delay);

    if device.read().is_err() {
        panic!();
    }
}
