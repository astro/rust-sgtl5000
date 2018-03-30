#![no_std]
#![feature(used)]

#[macro_use]
extern crate bitfield;
extern crate byteorder;
extern crate cortex_m;
extern crate cortex_m_rt;
extern crate cortex_m_semihosting;
#[macro_use(exception, interrupt)]
extern crate stm32f429;
extern crate embedded_hal;
extern crate stm32f429_hal;

// use byteorder::{ByteOrder, BigEndian};

use core::cell::RefCell;
use cortex_m::asm;
use cortex_m::interrupt::Mutex;
use stm32f429::{Peripherals, CorePeripherals, SYST};
use embedded_hal::blocking::*;
use embedded_hal::digital::OutputPin;
use stm32f429_hal::time::*;
use stm32f429_hal::gpio::GpioExt;
use stm32f429_hal::flash::FlashExt;
use stm32f429_hal::rcc::RccExt;
use stm32f429_hal::i2c::{I2c, Error as I2cError};
use stm32f429_hal::i2s::{I2s, I2sStandard};
use stm32f429_hal::dma::{DmaExt, C0};
use stm32f429_hal::dma::dma1::s4::DoubleBufferedTransfer;

use core::fmt::Write;
use cortex_m_semihosting::hio;

mod volume;
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
  
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    
    // TRY the other clock configuration
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    // let clocks = rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz()).freeze(&mut flash.acr);
    // writeln!(stdout, "clocks: {:?}", clocks);
    
    let mut cp = CorePeripherals::take().unwrap();
    setup_systick(&mut cp.SYST);

    let mut gpiob = p.GPIOB.split(&mut rcc.ahb1);
    let mut led1 = gpiob.pb0.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let mut led2 = gpiob.pb7.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let mut led3 = gpiob.pb14.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    
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

    let sd = gpiob.pb15.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
    let ck = gpiob.pb13.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
    let ws = gpiob.pb12.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
    writeln!(stdout, "I2S");
    let i2s = I2s::spi2(p.SPI2, sd, ck, ws, clocks, &mut rcc.apb1);
    writeln!(stdout, "I2S output");
    let mut output = i2s.into_slave_output::<u16>(I2sStandard::Pcm);

    writeln!(stdout, "DMA setup");
    let streams = p.DMA1.split(&mut rcc.ahb1);
    let mut i2s_stream = streams.s4;

    writeln!(stdout, "SGTL");
    let mut sgtl = SGTL5000::new(i2c).unwrap();

    let mut last_stats = get_time();
    let mut total_samples_prev = 0usize;
    let mut total_samples = 0usize;
    let mut total_transfers_prev = 0usize;
    let mut total_transfers = 0usize;
    let mut freq = 50;
    let mut vo = 0;
    // let mut volume = 0xFF;
    let mut values = [[0u16; 16384]; 3];
    let mut next_values = 2;
    // let mut freq = 50;
    let mut transfer: DoubleBufferedTransfer<u16> =
        output.dma_transfer(i2s_stream, C0, (&values[0], &values[1]));
    loop {
        led2.set_high();
        let this_values = next_values;
        freq += 1;
        if freq > 440 {
            freq = 50;
            writeln!(stdout, "L {}", get_time()).unwrap();
        }

        let interval = 2 * (48000 / freq);
        let delta = u16::max_value() / interval / 2;
        for (i, v) in values[this_values].iter_mut().enumerate() {
            // *v = vo;
            *v = if vo & 0x8000u16 == 0 { 0x7fff } else { 0x0 };
            vo += delta;
            // // Clip signedness
            // vo &= 0x7FFFu16;
        }
        led2.set_low();

        next_values += 1;
        if next_values > values.len() {
            next_values = 0;
        }
        
        next_values += 1;
        if next_values > values.len() {
            next_values = 0;
        }

        led3.set_high();
        let mut retries = 0;
        while !transfer.writable() { retries += 1; }
        transfer.write(&values[this_values]).unwrap();
        led3.set_low();

        if retries < 1 {
            writeln!(stdout, "Underrun?").unwrap();
        }

        // volume -= 1;
        // sgtl.control.set_dac_vol(volume);
        // sgtl.control.set_lineout_vol((volume & 0xF) << 4);
        // sgtl.control.set_hp_vol((volume & 0xF) << 4);

        led1.set_high();
        total_samples += values[this_values].len();
        total_transfers += 1;
        let now = get_time();
        let since_last_stats = now - last_stats;
        if since_last_stats >= 10 {
            writeln!(stdout, "{} s/s, {} t/s", (total_samples - total_samples_prev) / since_last_stats, (total_transfers - total_transfers_prev) / since_last_stats).unwrap();
            total_samples_prev = total_samples;
            total_transfers_prev = total_transfers;
            last_stats = now;
        }
        led1.set_low();
    }
}

static TIME: Mutex<RefCell<usize>> = Mutex::new(RefCell::new(0));

fn get_time() -> usize {
    cortex_m::interrupt::free(|cs| {
        *TIME.borrow(cs)
            .borrow()
    })
}

fn setup_systick(syst: &mut SYST) {
    syst.set_reload(100 * SYST::get_ticks_per_10ms());
    syst.enable_counter();
    syst.enable_interrupt();

    if ! SYST::is_precise() {
        let mut stderr = hio::hstderr().unwrap();
        writeln!(
            stderr,
            "Warning: SYSTICK with source {:?} is not precise",
            syst.get_clock_source()
        ).unwrap();
    }
}

fn systick_interrupt_handler() {
    cortex_m::interrupt::free(|cs| {
        let mut time =
            TIME.borrow(cs)
            .borrow_mut();
        *time += 1;
    })
}

#[used]
exception!(SYS_TICK, systick_interrupt_handler);
