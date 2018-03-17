pub(crate) trait I2cRegister {
    fn new(value: u16) -> Self;
    fn to_inner(&self) -> u16;
    fn register_addr() -> u16;
}

bitfield!{
    pub struct ChipId(u16);
    impl Debug;
    /// SGTL5000 Part ID
    pub u8, partid, _: 15, 8;
    /// SGTL5000 Revision ID
    pub u8, revid, _: 7, 0;
}

impl I2cRegister for ChipId {
    fn new(value: u16) -> Self {
        ChipId(value)
    }
    fn to_inner(&self) -> u16 {
        self.0
    }
    fn register_addr() -> u16 {
        0x0000
    }
}

bitfield!{
    pub struct ChipDigPower(u16);
    impl Debug;
    pub adc_powerup, set_adc_powerup: 6;
    pub dac_powerup, set_dac_powerup: 5;
    pub dap_powerup, set_dap_powerup: 4;
    pub i2s_out_powerup, set_i2s_out_powerup: 1;
    pub i2s_in_powerup, set_i2s_in_powerup: 0;
}

impl I2cRegister for ChipDigPower {
    fn new(value: u16) -> Self {
        ChipDigPower(value)
    }
    fn to_inner(&self) -> u16 {
        self.0
    }
    fn register_addr() -> u16 {
        0x0002
    }
}

bitfield!{
    pub struct ChipClkCtrl(u16);
    impl Debug;
    pub u8, rate_mode, set_rate_mode: 5, 4;
    pub u8, sys_fs, set_sys_fs: 3, 2;
    pub u8, mclk_freq, set_mclk_freq: 1, 0;
}

impl I2cRegister for ChipClkCtrl {
    fn new(value: u16) -> Self {
        ChipClkCtrl(value)
    }
    fn to_inner(&self) -> u16 {
        self.0
    }
    fn register_addr() -> u16 {
        0x0004
    }
}

bitfield!{
    pub struct ChipI2sCtrl(u16);
    impl Debug;
    pub sclkfreq, set_sclkfreq: 8;
    pub ms, set_ms: 7;
    pub sclk_inv, set_sclk_inv: 6;
    pub u8, dlen, set_dlen: 5, 4;
    pub u8, i2s_mode, set_i2s_mode: 3, 2;
    pub lralign, set_lralign: 1;
    pub lrpol, set_lrpol: 0;
}

impl I2cRegister for ChipI2sCtrl {
    fn new(value: u16) -> Self {
        ChipI2sCtrl(value)
    }
    fn to_inner(&self) -> u16 {
        self.0
    }
    fn register_addr() -> u16 {
        0x0006
    }
}

bitfield!{
    pub struct ChipSssCtrl(u16);
    impl Debug;
    /// DAP Mixer Input Swap
    pub dap_mix_lrswap, set_dap_mix_lrswap: 14;
    /// DAP Input Swap
    pub dap_lrswap, set_dap_lrswap: 13;
    /// DAC Input Swap
    pub dac_lrswap, set_dac_lrswap: 12;
    /// 
    pub i2s_lrswap, set_i2s_lrswap: 10;
    /// 
    pub u8, dap_mix_select, set_dap_mix_select: 9, 8;
    /// 
    pub u8, dap_select, set_dap_select: 7, 6;
    /// 
    pub u8, dac_select, set_dac_select: 5, 4;
    /// 
    pub u8, i2s_select, set_i2s_select: 1, 0;
}

impl I2cRegister for ChipSssCtrl {
    fn new(value: u16) -> Self {
        ChipSssCtrl(value)
    }
    fn to_inner(&self) -> u16 {
        self.0
    }
    fn register_addr() -> u16 {
        0x000A
    }
}

bitfield!{
    pub struct ChipAdcdacCtrl(u16);
    impl Debug;
    // TODO: rest
    ///
    pub dac_mute_right, set_dac_mute_right: 3;
    ///
    pub dac_mute_left, set_dac_mute_left: 2;
}

impl I2cRegister for ChipAdcdacCtrl {
    fn new(value: u16) -> Self {
        ChipAdcdacCtrl(value)
    }
    fn to_inner(&self) -> u16 {
        self.0
    }
    fn register_addr() -> u16 {
        0x000E
    }
}

bitfield!{
    pub struct ChipDacVol(u16);
    impl Debug;
    /// DAC Right Channel Volume
    pub dac_vol_right, set_dac_vol_right: 15, 8;
    /// DAC Left Channel Volume
    pub dac_vol_left, set_dac_vol_left: 7, 0;
}

impl I2cRegister for ChipDacVol {
    fn new(value: u16) -> Self {
        ChipDacVol(value)
    }
    fn to_inner(&self) -> u16 {
        self.0
    }
    fn register_addr() -> u16 {
        0x0010
    }
}

bitfield!{
    pub struct ChipAnaHpCtrl(u16);
    impl Debug;
    /// Headphone Right Channel Volume
    pub u8, hp_vol_right, set_hp_vol_right: 14, 8;
    /// Headphone Left Channel Volume
    pub u8, hp_vol_left, set_hp_vol_left: 6, 0;
}

impl I2cRegister for ChipAnaHpCtrl {
    fn new(value: u16) -> Self {
        ChipAnaHpCtrl(value)
    }
    fn to_inner(&self) -> u16 {
        self.0
    }
    fn register_addr() -> u16 {
        0x0022
    }
}

bitfield!{
    pub struct ChipAnaCtrl(u16);
    impl Debug;
    /// LINEOUT mute
    pub mute_lo, set_mute_lo: 8;
    /// Select the headphone input
    pub select_hp, set_select_hp: 6;
    // TODO: add the rest
    pub mute_hp, set_mute_hp: 4;
    /// Mute the ADC analog volume
    pub mute_adc, set_mute_adc: 0;
    /// Enable the headphone zero cross detector (ZCD)
    pub en_zcd_hp, set_en_zcd_hp: 5;
}

impl I2cRegister for ChipAnaCtrl {
    fn new(value: u16) -> Self {
        ChipAnaCtrl(value)
    }
    fn to_inner(&self) -> u16 {
        self.0
    }
    fn register_addr() -> u16 {
        0x0024
    }
}

bitfield!{
    pub struct ChipRefCtrl(u16);
    impl Debug;
    /// Analog Ground Voltage Control
    pub u8, vag_val, set_vag_val: 8, 4;
    /// Bias control
    pub u8, bias_ctrl, set_bias_ctrl: 3, 1;
    /// VAG Ramp Control
    pub small_pop, set_small_pop: 0;
}

impl I2cRegister for ChipRefCtrl {
    fn new(value: u16) -> Self {
        ChipRefCtrl(value)
    }
    fn to_inner(&self) -> u16 {
        self.0
    }
    fn register_addr() -> u16 {
        0x0028
    }
}

bitfield!{
    pub struct ChipLinregCtrl(u16);
    impl Debug;
    /// Determines chargepump source when VDDC_ASSN_OVRD is set.
    pub vdcc_man_assn, set_vdcc_man_assn: 6;
    /// Charge pump Source Assignment Override
    pub vdcc_assn_ovrd, set_vdcc_assn_ovrd: 5;
    /// Sets the VDDD linear regulator output voltage in 50 mV
    /// steps. Must clear the LINREG_SIMPLE_POWERUP and
    /// STARTUP_POWERUP bits in the 0x0030 register after power-up,
    /// for this setting to produce the proper VDDD voltage.
    pub d_programming, set_d_programming: 3, 0;
}

impl I2cRegister for ChipLinregCtrl {
    fn new(value: u16) -> Self {
        ChipLinregCtrl(value)
    }
    fn to_inner(&self) -> u16 {
        self.0
    }
    fn register_addr() -> u16 {
        0x0026
    }
}

bitfield!{
    pub struct ChipLineOutCtrl(u16);
    impl Debug;
    /// Controls the output bias current for the LINEOUT amplifiers.
    pub u8, out_current, set_out_current: 11, 8;
    /// LINEOUT Amplifier Analog Ground Voltage
    pub u8, lo_vagcntrl, set_lo_vagcntrl: 5, 0;
}

impl I2cRegister for ChipLineOutCtrl {
    fn new(value: u16) -> Self {
        ChipLineOutCtrl(value)
    }
    fn to_inner(&self) -> u16 {
        self.0
    }
    fn register_addr() -> u16 {
        0x002C
    }
}

bitfield!{
    pub struct ChipAnaPower(u16);
    impl Debug;
    pub dac_mono, set_dac_mono: 14;
    pub linreg_simple_powerup, set_linreg_simple_powerup: 13;
    pub startup_powerup, set_startup_powerup: 12;
    pub vddc_chrgpmp_powerup, set_vddc_chrgpmp_powerup: 11;
    pub pll_powerup, set_pll_powerup: 10;
    pub linreg_d_powerup, set_linreg_d_powerup: 9;
    pub vcoamp_powerup, set_vcoamp_powerup: 8;
    pub vag_powerup, set_vag_powerup: 7;
    pub adc_mono, set_adc_mono: 6;
    pub reftop_powerup, set_reftop_powerup: 5;
    pub headphone_powerup, set_headphone_powerup: 4;
    pub dac_powerup, set_dac_powerup: 3;
    pub capless_headphone_powerup, set_capless_headphone_powerup: 2;
    pub adc_powerup, set_adc_powerup: 1;
    pub lineout_powerup, set_lineout_powerup: 0;
}

impl I2cRegister for ChipAnaPower {
    fn new(value: u16) -> Self {
        ChipAnaPower(value)
    }
    fn to_inner(&self) -> u16 {
        self.0
    }
    fn register_addr() -> u16 {
        0x0030
    }
}

// impl ChipAnaPower {
//     fn default() -> Self {
//         Self::new(0x7060)
//     }
// }

bitfield!{
    pub struct ChipLineOutVol(u16);
    impl Debug;
    pub u8, lo_vol_right, set_lo_vol_right: 12, 8;
    pub u8, lo_vol_left, set_lo_vol_left: 4, 0;
}

impl I2cRegister for ChipLineOutVol {
    fn new(value: u16) -> Self {
        ChipLineOutVol(value)
    }
    fn to_inner(&self) -> u16 {
        self.0
    }
    fn register_addr() -> u16 {
        0x002E
    }
}

bitfield!{
    pub struct DapControl(u16);
    impl Debug;
    /// Enable/Disable the DAP mixer path
    pub mix_en, set_mix_en: 4;
    /// Enable/Disable digital audio processing (DAP)
    pub dap_en, set_dap_en: 0;
}

impl I2cRegister for DapControl {
    fn new(value: u16) -> Self {
        DapControl(value)
    }
    fn to_inner(&self) -> u16 {
        self.0
    }
    fn register_addr() -> u16 {
        0x0100
    }
}
