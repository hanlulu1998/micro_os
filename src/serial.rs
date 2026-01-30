#![allow(dead_code)]

use crate::io_port::Port;

const COM1: u16 = 0x3F8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SerialPort {
    data: Port<u8>,
    interrupt: Port<u8>,
    line_control: Port<u8>,
    fifo_control: Port<u8>,
    modem_control: Port<u8>,
    line_status: Port<u8>,
}

impl SerialPort {
    pub const fn new(base: u16) -> Self {
        Self {
            data: Port::new(base),
            interrupt: Port::new(base + 1),
            fifo_control: Port::new(base + 2),
            line_control: Port::new(base + 3),
            modem_control: Port::new(base + 4),
            line_status: Port::new(base + 5),
        }
    }

    pub fn init(&mut self) {
        self.interrupt.write(0x00);
        self.line_control.write(0x80);
        self.data.write(0x03);
        self.line_control.write(0x03);
        self.fifo_control.write(0xC7);
        self.modem_control.write(0x0B);
    }

    fn is_transmit_empty(&self) -> bool {
        self.line_status.read() & 0x20 != 0
    }

    pub fn write_byte(&mut self, byte: u8) {
        while !self.is_transmit_empty() {}
        self.data.write(byte);
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
           self.write_byte(byte);
        }
    }
}

use core::fmt;

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

use spin::Mutex;
pub static SERIAL: Mutex<SerialPort> = Mutex::new(
    SerialPort::new(COM1)
);

use spin::Once;

static INIT: Once<()> = Once::new();

fn init_serial() {
    INIT.call_once(|| {
        SERIAL.lock().init();
    });
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}


#[macro_export]
macro_rules! serial_println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}


#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    init_serial();
    use core::fmt::Write;
    SERIAL
        .lock()
        .write_fmt(args)
        .unwrap();
}

