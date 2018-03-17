use embedded_hal::blocking::i2c;
use stm32f429_hal::i2c::{I2c, Error as I2cError};

use registers::*;
use error::Error;

pub struct SGTL5000Control<I2C: i2c::Read + i2c::Write> {
    i2c: I2C,
}

const I2C_ADDR: u8 = 0b1010;

impl<I2C: i2c::Read<Error=I2CE> + i2c::Write<Error=I2CE>, I2CE> SGTL5000Control<I2C> {
    // TODO: -pub
    pub(crate) fn read_register<R: I2cRegister>(&mut self) -> Result<R, I2CE> {
        let addr = R::register_addr();
        // Send register addr
        // let mut addr_buf = [0u8; 2];
        // BigEndian::write_u16(&mut addr_buf, addr);
        let mut addr_buf = [(addr >> 8) as u8, addr as u8];
        self.i2c.write(I2C_ADDR, &addr_buf)?;

        // Receive value
        let mut value_buf = [0u8; 2];
        self.i2c.read(I2C_ADDR, &mut value_buf)?;
        // let value = BigEndian::read_u16(&value_buf);
        let value = ((value_buf[0] as u16) << 8) | (value_buf[1] as u16);
        Ok(R::new(value))
    }

    fn write_register<R: I2cRegister>(&mut self, register: R) -> Result<(), I2CE> {
        let addr = R::register_addr();
        let value = register.to_inner();
        // Send register addr and value
        // let mut buf = [0u8; 4];
        // BigEndian::write_u16(&mut buf[0..2], addr);
        // BigEndian::write_u16(&mut buf[2..4], value);
        let mut buf = [(addr >> 8) as u8, addr as u8,
                       (value >> 8) as u8, value as u8];
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
        
        //--------------- Power Supply Configuration----------------
        // NOTE: This next 2 Write calls is needed ONLY if VDDD is
        // internally driven by the chip
        // Configure VDDD level to 1.2V (bits 3:0)
        self.modify_register(|mut linreg: ChipLinregCtrl| {
            // VDDA & VDDIO both over 3.1V
            linreg.set_vdcc_man_assn(true);
            linreg.set_vdcc_assn_ovrd(true);
            linreg.set_d_programming(0);
            linreg
        })?;
        // 3.3V
        let vdda = 3300;
        // 0.8..1575V
        let vag: u16 = vdda / 2;
        let vag_val = vag.saturating_sub(Self::ANA_GND_BASE) / Self::ANA_GND_STEP;
        self.modify_register(|mut ref_ctrl: ChipRefCtrl| {
            ref_ctrl.set_vag_val(vag_val.min(0x1F) as u8);
            // ref_ctrl.set_vag_val(0x1F);
            ref_ctrl.set_bias_ctrl(1);
            ref_ctrl
        })?;
        self.modify_register(|mut ana_power: ChipAnaPower| {
            // ana_power.set_startup_powerup(false);
            // ana_power.set_linreg_simple_powerup(false);

            // Power up internal linear regulator (Set bit 9)
            ana_power.set_linreg_d_powerup(true);
            // NOTE: This next Write call is needed ONLY if VDDD is
            // externally driven. Turn off startup power supplies to
            // save power (Clear bit 12 and 13).
            ana_power.set_vddc_chrgpmp_powerup(true);
            ana_power
        })?;
        // // NOTE: The next Write calls is needed only if both VDDA and
        // // VDDIO power supplies are less than 3.1V.
        // // Enable the internal oscillator for the charge pump (Set bit 11)
        // Write CHIP_CLK_TOP_CTRL 0x0800
        // // Enable charge pump (Set bit 11)
        // Write CHIP_ANA_POWER 0x4A60

        // NOTE: The next modify call is only needed if both VDDA and
        // VDDIO are greater than 3.1 V
        // Configure the charge pump to use the VDDIO rail (set bit 5 and bit 6)
        //---- Reference Voltage and Bias Current Configuration----
        self.modify_register(|mut line_out_ctrl: ChipLineOutCtrl| {
            // LO_VAGCNTRL=1.65V
            line_out_ctrl.set_lo_vagcntrl(0x22);
            // OUT_CURRENT=0.54mA
            line_out_ctrl.set_out_current(0xF);
            line_out_ctrl
        })?;

        // //------------Other Analog Block Configurations--------------
        // // Configure slow ramp up rate to minimize pop (bit 0)
        // Write CHIP_REF_CTRL 0x004F
        // // Enable short detect mode for headphone left/right
        // // and center channel and set short detect current trip level
        // // to 75 mA
        // Write CHIP_SHORT_CTRL 0x1106
        // // Enable Zero-cross detect if needed for HP_OUT (bit 5) and ADC (bit 1)
        // Write CHIP_ANA_CTRL 0x0133

        //------------Power up Inputs/Outputs/Digital Blocks---------
        // Power up LINEOUT, HP, ADC, DAC
        self.modify_register(|mut ana_power: ChipAnaPower| {
            ana_power.set_vag_powerup(true);
            // Power up desired digital blocks
            ana_power.set_lineout_powerup(true);
            ana_power.set_adc_powerup(true);
            ana_power.set_capless_headphone_powerup(true);
            ana_power.set_dac_powerup(true);
            ana_power.set_headphone_powerup(true);
            ana_power.set_reftop_powerup(true);
            ana_power.set_dac_mono(false);
            ana_power
        })?;
        // Power up desired digital blocks
        self.modify_register(|mut dig_power: ChipDigPower| {
            // dig_power.set_adc_powerup(true);
            dig_power.set_dac_powerup(true);
            // dig_power.set_dap_powerup(true);
            // dig_power.set_i2s_out_powerup(true);
            dig_power.set_i2s_in_powerup(true);
            dig_power
        })?;
        // self.modify_register(|mut dap_control: DapControl| {
        //     dap_control.set_dap_en(true);
        //     dap_control
        // })?;

        self.modify_register(|mut clk_ctrl: ChipClkCtrl| {
            // Configure SYS_FS clock to 32 kHz
            clk_ctrl.set_sys_fs(0);
            // Configure MCLK_FREQ to 256*Fs
            clk_ctrl.set_mclk_freq(0);
            clk_ctrl
        })?;
        self.modify_register(|mut i2s_ctrl: ChipI2sCtrl| {
            // Master mode
            i2s_ctrl.set_ms(true);
            // 32Fs
            i2s_ctrl.set_sclkfreq(true);
            // I2S data length: 16 bits
            // TODO: let depend on types
            i2s_ctrl.set_dlen(3);
            // MSB-justified
            i2s_ctrl.set_i2s_mode(0);
            i2s_ctrl
        })?;

        //----------------Set LINEOUT Volume Level-------------------
        // Set the LINEOUT volume level based on voltage reference
        // (VAG) values using this formula Value =
        // (int)(40*log(VAG_VAL/LO_VAGCNTRL) + 15) Assuming VAG_VAL
        // and LO_VAGCNTRL is set to 0.9 V and 1.65 V respectively,
        // the left LO vol (bits 12:8) and right LO volume (bits 4:0)
        // value should be set to 5
        //
        // default approx 1.3 volts peak-to-peak
        let mut line_out_vol = ChipLineOutVol::new(0);
        line_out_vol.set_lo_vol_right(0x1D);
        line_out_vol.set_lo_vol_left(0x1D);
        self.write_register(line_out_vol)?;

        // Setup routing
        // Example 1: I2S_IN /*-> DAP*/ -> DAC -> LINEOUT, HP_OUT
        self.modify_register(|mut sss_ctrl: ChipSssCtrl| {
            // // Route I2S_IN to DAP
            // sss_ctrl.set_dap_select(1);
            // Route I2S_IN to DAC
            sss_ctrl.set_dac_select(1);
            sss_ctrl
        })?;
        self.modify_register(|mut ana_ctrl: ChipAnaCtrl| {
            // Select DAC as the input to HP_OUT
            ana_ctrl.set_select_hp(false);
            // Unmute
            ana_ctrl.set_mute_hp(false);
            // ana_ctrl.set_mute_adc(false);
            ana_ctrl.set_en_zcd_hp(false);
            ana_ctrl
        })?;
        self.modify_register(|mut adcdac_ctrl: ChipAdcdacCtrl| {
            adcdac_ctrl.set_dac_mute_right(false);
            adcdac_ctrl.set_dac_mute_left(false);
            adcdac_ctrl
        })?;

        // Volume
        let mut ana_hp_ctrl = ChipAnaHpCtrl::new(0);
        ana_hp_ctrl.set_hp_vol_right(0x18);
        ana_hp_ctrl.set_hp_vol_left(0x18);
        self.write_register(ana_hp_ctrl)?;

        let mut dac_vol = ChipDacVol::new(0);
        dac_vol.set_dac_vol_right(0x3c);
        dac_vol.set_dac_vol_left(0x3c);
        self.write_register(dac_vol)?;
        
        Ok(())
    }
}
