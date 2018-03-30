use byteorder::{ByteOrder, BigEndian};

use embedded_hal::blocking::i2c;
use stm32f429_hal::i2c::{I2c, Error as I2cError};

use registers::*;
use error::Error;
use volume::Volume;


pub struct SGTL5000Control<I2C: i2c::Read + i2c::Write> {
    i2c: I2C,
}

const I2C_ADDR: u8 = 0b1010;

impl<I2C: i2c::Read<Error=I2CE> + i2c::Write<Error=I2CE>, I2CE> SGTL5000Control<I2C> {
    // TODO: -pub
    pub(crate) fn read_register<R: I2cRegister>(&mut self) -> Result<R, I2CE> {
        let addr = R::register_addr();
        // Send register addr
        let mut addr_buf = [0u8; 2];
        BigEndian::write_u16(&mut addr_buf, addr);
        self.i2c.write(I2C_ADDR, &addr_buf)?;

        // Receive value
        let mut value_buf = [0u8; 2];
        self.i2c.read(I2C_ADDR, &mut value_buf)?;
        let value = BigEndian::read_u16(&value_buf);
        Ok(R::new(value))
    }

    fn write_register<R: I2cRegister>(&mut self, register: R) -> Result<(), I2CE> {
        let addr = R::register_addr();
        let value = register.to_inner();
        // Send register addr and value
        let mut buf = [0u8; 4];
        BigEndian::write_u16(&mut buf[0..2], addr);
        BigEndian::write_u16(&mut buf[2..4], value);
        self.i2c.write(I2C_ADDR, &buf)?;

        Ok(())
    }

    fn modify_register<R, F>(&mut self, f: F) -> Result<(), I2CE>
    where R: I2cRegister,
          F: Fn(R) -> R
    {
        let register: R = self.read_register()?;
        let register = f(register);
        self.write_register(register)?;

        Ok(())
    }

    pub fn new(i2c: I2C) -> Result<Self, Error<I2CE>> {
        let mut sgtl5000 = Self {
            i2c,
        };
        sgtl5000.init()?;
        Ok(sgtl5000)
    }

    /// 0.8V
    const ANA_GND_BASE: u16 = 800;
    /// 0.025V
    const ANA_GND_STEP: u16 = 25;
    
    fn init(&mut self) -> Result<(), Error<I2CE>> {
        let chip_id: ChipId = self.read_register()?;
        if chip_id.partid() != 0xA0 {
            return Err(Error::Identification)
        }
        
        self.modify_register(|mut ana_power: ChipAnaPower| {
            ana_power.set_reftop_powerup(true);
            // Enable stereo
            ana_power.set_dac_mono(true);
            // ana_power.set_adc_mono(true);

            ana_power
        })?;
        self.modify_register(|mut linreg: ChipLinregCtrl| {
            // VDDA & VDDIO both over 3.1V
            linreg.set_vdcc_assn_ovrd(false);
            linreg.set_vdcc_man_assn(false);
            linreg.set_d_programming(0);
            linreg
        })?;
        // // Setup PL for 12 MHz clock
        // self.modify_register(|mut clk_top_ctrl: ChipClkTopCtrl| {
        //     clk_top_ctrl.set_input_freq_div2(false);
        //     clk_top_ctrl
        // })?;
        // self.modify_register(|mut pll_ctrl: ChipPllCtrl| {
        //     pll_ctrl.set_int_divisor(16);
        //     pll_ctrl.set_frac_divisor(786);
        //     pll_ctrl
        // })?;

        // 3.3V
        let vdda = 3300;
        // 0.8..1575V
        let vag: u16 = vdda / 2;
        let vag_val = vag.saturating_sub(Self::ANA_GND_BASE) / Self::ANA_GND_STEP;
        self.modify_register(|mut ref_ctrl: ChipRefCtrl| {
            ref_ctrl.set_vag_val(vag_val.min(0x1F) as u8);
            ref_ctrl.set_bias_ctrl(1);
            ref_ctrl
        })?;
        self.modify_register(|mut line_out_ctrl: ChipLineOutCtrl| {
            // LO_VAGCNTRL=1.65V
            line_out_ctrl.set_lo_vagcntrl(vag_val.min(0x23) as u8);
            // OUT_CURRENT=0.54mA
            line_out_ctrl.set_out_current(0xF);
            line_out_ctrl
        })?;
        self.modify_register(|mut short_ctrl: ChipShortCtrl| {
            short_ctrl.set_lvladjr(4);
            short_ctrl.set_lvladjl(4);
            short_ctrl.set_lvladjc(4);
            short_ctrl.set_mode_lr(1);
            short_ctrl.set_mode_cm(2);
            short_ctrl
        })?;
        self.modify_register(|mut ana_ctrl: ChipAnaCtrl| {
            // Select DAC as the input to HP_OUT
            ana_ctrl.set_select_hp(false);
            // Unmute
            ana_ctrl.set_mute_hp(false);
            ana_ctrl.set_mute_lo(false);
            // ana_ctrl.set_mute_adc(false);
            ana_ctrl.set_en_zcd_hp(true);
            ana_ctrl
        })?;

        self.modify_register(|mut ana_power: ChipAnaPower| {
            // Power up internal linear regulator (Set bit 9)
            ana_power.set_linreg_d_powerup(false);
            ana_power.set_vddc_chrgpmp_powerup(true);

            ana_power.set_pll_powerup(false);
            ana_power.set_vcoamp_powerup(true);

            // Enable stereo
            ana_power.set_dac_mono(true);

            ana_power
        })?;
        let mut clk_ctrl = ChipClkCtrl::new(0);
        // Configure SYS_FS clock to 48 kHz
        clk_ctrl.set_sys_fs(2);
        // Configure MCLK_FREQ to 256*Fs
        clk_ctrl.set_mclk_freq(0);
        // // Use PLL
        // clk_ctrl.set_mclk_freq(3);
        // 1/1
        clk_ctrl.set_rate_mode(0);
        self.write_register(clk_ctrl)?;
        self.modify_register(|mut i2s_ctrl: ChipI2sCtrl| {
            // Master mode
            i2s_ctrl.set_ms(true);
            // 32Fs
            i2s_ctrl.set_sclkfreq(true);
            i2s_ctrl.set_sclk_inv(false);
            i2s_ctrl.set_lralign(false);
            i2s_ctrl.set_lrpol(false);
            // i2s_ctrl.set_pcmsync(false);
            // I2S data length: 0: 32 bits, 3: 16 bits
            // TODO: let depend on types
            i2s_ctrl.set_dlen(3);
            // PCM standard
            i2s_ctrl.set_i2s_mode(2);
            i2s_ctrl
        })?;
        self.modify_register(|mut ana_power: ChipAnaPower| {
            ana_power.set_startup_powerup(false);
            ana_power.set_linreg_simple_powerup(true);

            // Power up desired digital blocks
            ana_power.set_lineout_powerup(true);
            ana_power.set_adc_powerup(true);
            ana_power.set_capless_headphone_powerup(true);
            ana_power.set_dac_powerup(true);
            ana_power.set_headphone_powerup(true);
            ana_power.set_reftop_powerup(true);

            ana_power.set_vag_powerup(true);
            ana_power
        })?;
        self.modify_register(|mut dap_control: DapControl| {
            dap_control.set_dap_en(true);
            dap_control
        })?;

        // Power up desired digital blocks
        self.modify_register(|mut dig_power: ChipDigPower| {
            // dig_power.set_adc_powerup(true);
            dig_power.set_dac_powerup(true);
            dig_power.set_dap_powerup(true);
            // dig_power.set_i2s_out_powerup(true);
            dig_power.set_i2s_in_powerup(true);
            dig_power
        })?;

        // Setup routing
        // Example 1: I2S_IN -> DAP -> DAC -> LINEOUT, HP_OUT
        self.modify_register(|mut sss_ctrl: ChipSssCtrl| {
            // Route I2S_IN to DAP
            sss_ctrl.set_dap_select(1);
            // Route DAP to DAC
            sss_ctrl.set_dac_select(3);
            sss_ctrl
        })?;
        self.modify_register(|mut adcdac_ctrl: ChipAdcdacCtrl| {
            adcdac_ctrl.set_vol_ramp_en(true);
            adcdac_ctrl.set_vol_expo_ramp(false);
            adcdac_ctrl.set_dac_mute_right(false);
            adcdac_ctrl.set_dac_mute_left(false);
            adcdac_ctrl
        })?;

        // Volume
        let mut ana_hp_ctrl = ChipAnaHpCtrl::new(0);
        ana_hp_ctrl.set_hp_vol_right(0x18);
        ana_hp_ctrl.set_hp_vol_left(0x18);
        self.write_register(ana_hp_ctrl)?;

        let mut line_out_vol = ChipLineOutVol::new(0);
        line_out_vol.set_lo_vol_right(0x19);
        line_out_vol.set_lo_vol_left(0x19);
        self.write_register(line_out_vol)?;

        let mut dac_vol = ChipDacVol::new(0);
        // Min: 0xFC, Max: 0x3c
        dac_vol.set_dac_vol_right(0x3c);
        dac_vol.set_dac_vol_left(0x3c);
        self.write_register(dac_vol)?;
        // self.set_dac_vol(0xff);
        // self.set_lineout_vol(0xff);
        // self.set_hp_vol(0xff);

                             // if false {
        use core::fmt::Write;
        use cortex_m_semihosting::hio;
        let mut stdout = hio::hstdout().unwrap();
        for a in 0x1B..0x1D /*0..0x20*/ {
            let addr = 2 * a;
            // Send register addr
            let mut addr_buf = [0u8; 2];
            BigEndian::write_u16(&mut addr_buf, addr);
            self.i2c.write(I2C_ADDR, &addr_buf)?;

            // Receive value
            let mut value_buf = [0u8; 2];
            self.i2c.read(I2C_ADDR, &mut value_buf)?;
            let value = BigEndian::read_u16(&value_buf);
            
            writeln!(stdout, "R {:02X} = {:04X}", addr, value).unwrap();
        }
                             // }

        Ok(())
    }

    /// Set DAC volume
    pub fn set_dac_vol<V: Into<Volume>>(&mut self, v: V) {
        let volume = v.into();
        let (left, right) = volume.to_range(0xFC, 0x3C);

        let mut dac_vol = ChipDacVol::new(0);
        dac_vol.set_dac_vol_left(left);
        dac_vol.set_dac_vol_right(right);
        self.write_register(dac_vol);
    }

    /// Set LINE_OUT volume
    pub fn set_lineout_vol<V: Into<Volume>>(&mut self, v: V) {
        let volume = v.into();
        let (left, right) = volume.to_range(0, 0x1F);

        let mut line_out_vol = ChipLineOutVol::new(0);
        line_out_vol.set_lo_vol_left(left);
        line_out_vol.set_lo_vol_right(right);
        self.write_register(line_out_vol);
    }

    /// Set headphones volume
    pub fn set_hp_vol<V: Into<Volume>>(&mut self, v: V) {
        let volume = v.into();
        let (left, right) = volume.to_range(0x7F, 0);

        let mut ana_hp_ctrl = ChipAnaHpCtrl::new(0);
        ana_hp_ctrl.set_hp_vol_right(left);
        ana_hp_ctrl.set_hp_vol_left(right);
        self.write_register(ana_hp_ctrl);
    }
}
