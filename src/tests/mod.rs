use embedded_hal::blocking::delay::DelayMs;

/// When put this data in segment_read, Uart.read() return Ok
const READ_DATA_OK: [u8; crate::READ_DATA_SIZE] = [
    0x01, 0x03, 0x38,
    0x00, 0x24, 0x8E, 0x5F,
    0x00, 0x00, 0x94, 0x0B,
    0x00, 0x8A, 0xC4, 0xAA,
    0x00, 0x00, 0x2D, 0xB4,
    0x00, 0x00, 0x03, 0xE8,
    0x00, 0x00, 0x26, 0x16,
    0x01, 0x01, 0x00, 0x00,
    0x00, 0x00, 0x13, 0x8B,
    0x00, 0x24, 0x8E, 0x5F,
    0x00, 0x00, 0x94, 0x0A,
    0x00, 0x8A, 0x92, 0xD7,
    0x00, 0x00, 0x2D, 0xB4,
    0x00, 0x00, 0x03, 0xE8,
    0x00, 0x00, 0x26, 0x20,
    0x87, 0xB2
    ];

/// When put this data in segment_read, Uart.read() return Ok
const READ_DATA_OK_2: [u8; crate::READ_DATA_SIZE] = [
    0x01, 0x03, 0x38,
    0x00, 0x23, 0xCF, 0x24,
    0x00, 0x00, 0x03, 0xFD,
    0x00, 0x02, 0x1F, 0x95,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x02, 0x45,
    0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x13, 0x89,
    0x00, 0x23, 0xCF, 0x24,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x79, 0xB0 
    ];

/// When put this data in segment_read, Uart.read() return Err
const READ_DATA_ERROR: [u8; crate::READ_DATA_SIZE] = [0xff; crate::READ_DATA_SIZE];

/// When put this data in segment_read, Uart.read() return wrong siaz
const READ_DATA_WRONG_SIZE: [u8; crate::READ_DATA_SIZE] = [0x01; crate::READ_DATA_SIZE];

/// When put this data in segment_write, Uart.write() return Ok
const WRITE_DATA_OK: [u8; crate::SEGMENT_WRITE_CHANGE_BIT_RATE] = [0; crate::SEGMENT_WRITE_CHANGE_BIT_RATE];

/// When put this data in segment_write, Uart.write() return Err
const WRITE_DATA_ERROR: [u8; crate::SEGMENT_WRITE_CHANGE_BIT_RATE] = [0xff; crate::SEGMENT_WRITE_CHANGE_BIT_RATE];

struct UartTestImpl {
    pub segment_write: [u8; crate::SEGMENT_WRITE_CHANGE_BIT_RATE],
    pub segment_write_len: usize,
    pub segment_read: [u8; crate::READ_DATA_SIZE]
}

type UartError = std::io::Error;

impl crate::Uart for UartTestImpl {
    type Error = UartError;

    fn read(&mut self, buf: &mut [u8], _timeout: u32) -> Result<usize, Self::Error> {
        match self.segment_read {
            READ_DATA_WRONG_SIZE => Ok(1),
            READ_DATA_ERROR => Err(UartError::new(std::io::ErrorKind::Other, "Error read")),
            _ => {
                let mut index = 0;

                for data in self.segment_read {
                    buf[index] = data;
                    index += 1;
                }
        
                Ok(self.segment_read.len())
            }
        }
    }

    fn write(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
        self.segment_write_len = bytes.len();

        if self.segment_write == WRITE_DATA_ERROR {
            return Err(UartError::new(std::io::ErrorKind::Other, "Error write"))
        }


        let mut index = 0;

        for data in bytes {
            self.segment_write[index] = *data;
            index += 1;
        }

        Ok(bytes.len())
    }
}

struct DelayTestImpl {

}

impl DelayMs<u16> for DelayTestImpl {
    fn delay_ms(&mut self, _ms: u16) {
        // Do nothing
    }
}

fn setup(read_data: [u8; crate::READ_DATA_SIZE], write_data: [u8; crate::SEGMENT_WRITE_CHANGE_BIT_RATE]) -> crate::JsyMk194<UartTestImpl, DelayTestImpl> {
    let uart = UartTestImpl {
        segment_write: write_data,
        segment_read: read_data,
        segment_write_len: 0
    };

    let delay = DelayTestImpl {};
    crate::JsyMk194::new(uart, delay)
}

#[test]
fn test_read_ok() {
    let mut device = setup(READ_DATA_OK, WRITE_DATA_OK);

    assert!(device.read());

    let voltage_1 = device.voltage_1();
    let current_1 = device.current_1();
    let power_1 = device.power_1();
    let positive_energy_1 = device.positive_energy_1();
    let negative_energy_1 = device.negative_energy_1();
    let factor_1 = device.factor_1();

    let voltage_2 = device.voltage_2();
    let current_2 = device.current_2();
    let power_2 = device.power_2();
    let positive_energy_2 = device.positive_energy_2();
    let negative_energy_2 = device.negative_energy_2();
    let factor_2 = device.factor_1();

    let frequency = device.frequency();

    assert_eq!(voltage_1, 239.574295);
    assert_eq!(current_1, 3.78989983);
    assert_eq!(power_1, -909.431396);
    assert_eq!(positive_energy_1, 1.16999996);
    assert_eq!(negative_energy_1, 0.974999964);
    assert_eq!(factor_1, 1.0);

    assert_eq!(voltage_2, 239.574295);
    assert_eq!(current_2, 3.78979993);
    assert_eq!(power_2, -908.155883);
    assert_eq!(positive_energy_2, 1.16999996);
    assert_eq!(negative_energy_2, 0.975999951);
    assert_eq!(factor_2, 1.0);
    
    assert_eq!(frequency, 50.0299988);
}

#[test]
fn test_read_ok_2() {
    let mut device = setup(READ_DATA_OK_2, WRITE_DATA_OK);

    assert!(device.read());

    let voltage_1 = device.voltage_1();
    let current_1 = device.current_1();
    let power_1 = device.power_1();
    let positive_energy_1 = device.positive_energy_1();
    let negative_energy_1 = device.negative_energy_1();
    let factor_1 = device.factor_1();

    let voltage_2 = device.voltage_2();
    let current_2 = device.current_2();
    let power_2 = device.power_2();
    let positive_energy_2 = device.positive_energy_2();
    let negative_energy_2 = device.negative_energy_2();

    let frequency = device.frequency();

    assert_eq!(voltage_1, 234.678787);
    assert_eq!(current_1, 0.1021);
    assert_eq!(power_1, -13.9157);
    assert_eq!(positive_energy_1, 0.0);
    assert_eq!(negative_energy_1, 0.0);
    assert_eq!(factor_1, 0.58100003);

    assert_eq!(voltage_2, 234.678787);
    assert_eq!(current_2, 0.0);
    assert_eq!(power_2, 0.0);
    assert_eq!(positive_energy_2, 0.0);
    assert_eq!(negative_energy_2, 0.0);
    
    assert_eq!(frequency, 50.0099983);
}

#[test]
fn test_jsk_mk_196_read_method_return_error_cause_write_to_device_return_error() {
    let mut device = setup(READ_DATA_OK, WRITE_DATA_ERROR);

    assert!(device.read() == false);
}

#[test]
fn test_jsk_mk_196_read_method_return_error_cause_read_to_device_return_error() {
    let mut device = setup(READ_DATA_ERROR, WRITE_DATA_OK);

    assert!(device.read() == false);
}

#[test]
fn test_jsk_mk_196_read_method_return_error_cause_read_to_device_return_wrong_size() {
    let mut device = setup(READ_DATA_WRONG_SIZE, WRITE_DATA_OK);

    assert!(device.read() == false);
}

#[test]
fn test_jsk_mk_196_change_bitrate_method_return_true() {
    let mut device = setup(READ_DATA_OK, WRITE_DATA_OK);

    device.change_bitrate(crate::ChangeBitrate::B9600);

    assert_eq!(device.get_uart().segment_write_len, crate::SEGMENT_WRITE_CHANGE_BIT_RATE);
    assert_eq!(device.get_uart().segment_write, [0x00, 0x10, 0x00, 0x04, 0x00, 0x01, 0x02, 0x01, 0x06, 0x2b, 0xd6]);
}
