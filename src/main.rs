#![no_std]
//#![feature(used)]

#[macro_use]
extern crate bitfield;
// extern crate byteorder;
extern crate cortex_m;
extern crate cortex_m_rt;
extern crate cortex_m_semihosting;
#[macro_use(exception, interrupt)]
extern crate stm32f429;
extern crate embedded_hal;
extern crate stm32f429_hal;

// use byteorder::{ByteOrder, BigEndian};

use cortex_m::asm;
use stm32f429::{Peripherals, CorePeripherals};
use embedded_hal::blocking::*;
use stm32f429_hal::time::*;
use stm32f429_hal::gpio::GpioExt;
use stm32f429_hal::flash::FlashExt;
use stm32f429_hal::rcc::RccExt;
use stm32f429_hal::i2c::{I2c, Error as I2cError};
use stm32f429_hal::i2s::{I2s, I2sStandard};
use stm32f429_hal::dma::DmaExt;

use core::fmt::Write;
use cortex_m_semihosting::hio;

mod registers;
use registers::*;
mod control;
use control::SGTL5000Control;
mod error;
use error::Error;


/// https://www.nxp.com/docs/en/data-sheet/SGTL5000.pdf
struct SGTL5000<I2C: i2c::Read + i2c::Write> {
    control: SGTL5000Control<I2C>,
}

const I2C_ADDR: u8 = 0b1010;

impl<I2C: i2c::Read<Error=I2CE> + i2c::Write<Error=I2CE>, I2CE> SGTL5000<I2C> {
    pub fn new(i2c: I2C) -> Result<Self, Error<I2CE>> {
        let control = SGTL5000Control::new(i2c)?;
        let mut sgtl5000 = Self {
            control,
        };
        Ok(sgtl5000)
    }
}

fn main() {
    let mut stdout = hio::hstdout().unwrap();
    writeln!(stdout, "Hello!");

    let p = Peripherals::take().unwrap();
    // let mut cp = CorePeripherals::take().unwrap();
  
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    
    // TRY the other clock configuration
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    // let clocks = rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz()).freeze(&mut flash.acr);
    // writeln!(stdout, "clocks: {:?}", clocks);
    
    let mut gpiob = p.GPIOB.split(&mut rcc.ahb1);
    // Bit 1 I2C1_REMAP: I2C1 remapping
    // This bit is set and cleared by software. It controls the mapping of I2C1 SCL and SDA
    // alternate functions on the GPIO ports.
    // 0: No remap (SCL/PB6, SDA/PB7)
    // 1: Remap (SCL/PB8, SDA/PB9)
    let scl = gpiob.pb8
        .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper)
        .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr)
        .into_af4(&mut gpiob.moder, &mut gpiob.afrh);
    let sda = gpiob.pb9
        .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper)
        .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr)
        .into_af4(&mut gpiob.moder, &mut gpiob.afrh);
    writeln!(stdout, "I2C");
    let i2c = I2c::i2c1(p.I2C1, scl, sda, 100.khz(), clocks, &mut rcc.apb1);
    writeln!(stdout, "SGTL");
    let mut sgtl = SGTL5000::new(i2c).unwrap();

    let sd = gpiob.pb15.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
    let ck = gpiob.pb13.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
    let ws = gpiob.pb12.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
    writeln!(stdout, "I2S");
    let i2s = I2s::spi2(p.SPI2, sd, ck, ws, clocks, &mut rcc.apb1);
    writeln!(stdout, "I2S output");
    let mut output = i2s.into_slave_output(I2sStandard::MsbJustified);

    writeln!(stdout, "DMA setup");
    let streams = p.DMA1.split(&mut rcc.ahb1);
    let mut stream = streams.s4;
    
    let mut values = [0u16; 32768];
    let freq = 220;
    let interval = 48000 / freq;
    let delta = u16::max_value() / interval;
    writeln!(stdout, "interval={} delta={}", interval, delta);
    let mut vo = 0;
    for i in 0..values.len() {
        values[i] = vo;
        vo += delta;
    }
    
    loop {
        // writeln!(stdout, "Send values");
        // match output.dma_write(&values, stream) {
        //     Ok(s) => {
        //         stream = s;
        //     },
        //     Err(s) => {
        //         writeln!(stdout, "DMA error");
        //         stream = s;
        //     },
        // }

        for f in 2..1000 {
            for v in values.iter() {
                output.write(*v);
            }
        }

        // let chip_id: Result<ChipId, _> = sgtl.control.read_register();
        // match chip_id {
        //     Ok(chip_id) =>
        //         writeln!(stdout, "chip id: {:?}", chip_id).unwrap(),
        //     Err(e) =>
        //         writeln!(stdout, "error: {:?}", e).unwrap(),
        // }
    }
}
