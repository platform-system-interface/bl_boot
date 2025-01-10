use std::fmt::{Display, Formatter};

// NOTE: bitfields/bitflags/... are *not* trivial. See also:
// https://hecatia-elegua.github.io/blog/no-more-bit-fiddling/#how-bilge-came-to-be
// https://github.com/google/zerocopy/issues/388
// https://github.com/google/zerocopy/issues/1497
use bitfield_struct::bitfield;
use zerocopy::FromBytes;
use zerocopy_derive::{FromBytes, IntoBytes};

#[bitfield(u32)]
#[derive(FromBytes, IntoBytes)]
pub struct Config {
    #[bits(2)]
    ef_sf_aes_mode: u8,
    ef_ai_dis: bool,
    ef_cpu0_dis: bool,
    #[bits(2)]
    ef_sboot_en: u8,
    #[bits(4)]
    ef_uart_dis: u8,
    ef_ble2_dis: bool,
    ef_m1542_dis: bool,
    #[bits(2)]
    ef_sf_key_re_sel: u8,
    ef_sdu_dis: bool,
    ef_btdm_dis: bool,
    ef_wifi_dis: bool,
    ef_0_key_enc_en: bool,
    ef_cam_dis: bool,
    ef_m154_dis: bool,
    ef_cpu1_dis: bool,
    ef_cpu_rst_dbg_dis: bool,
    ef_se_dbg_dis: bool,
    ef_efuse_dbg_dis: bool,
    #[bits(2)]
    ef_dbg_jtag_1_dis: u8,
    #[bits(2)]
    ef_dbg_jtag_0_dis: u8,
    #[bits(4)]
    ef_dbg_mode: u8,
}

#[bitfield(u64)]
#[derive(FromBytes, IntoBytes)]
pub struct WifiMacAndInfo {
    // TODO: There might be some built-in that we can use here.
    // NOTE: Bouffalo Lab MAC prefix is b4:0e:cf
    // https://macaddress.io/macaddress/B4:0E:CF
    #[bits(48)]
    pub mac_addr: u64,
    #[bits(6)]
    _unused: u8,
    // FIXME: The vendor code has obvious bugs. Those bit sizes or the semantics
    // are incorrect, as they do not fit together here:
    // https://github.com/bouffalolab/bouffalo_sdk
    // 76ebf6ffcbc2a81d18dd18eb3a22810779edae1a
    // drivers/soc/bl808/std/src/bl808_ef_cfg.c#L150
    #[bits(3)]
    pub package: u8,
    #[bits(2)]
    pub psram: u8,
    #[bits(2)]
    pub flash: u8,
    #[bits(3)]
    pub version: u8,
}

// TODO
type Key = [u8; 16];

/// https://openbouffalo.github.io/chips/bl808/efuse/
#[bitfield(u32)]
#[derive(FromBytes, IntoBytes)]
pub struct SwConfig0 {
    /// Code at 0x900140b0. Executed before jumping to user code.
    /// Presumably disables access to the boot ROM.
    pub bootrom_protect: bool,
    /// Boot ROM debugging, see notes
    pub uart_log_disable: bool,
    /// Bootloader entry GPIO. 0: GPIO39, 1: GPIO8
    pub boot_pin_cfg: bool,
    /// Bootloader UART (UART0) pins (RX/TX): 0: GPIO20/21, 1: GPIO14/15
    pub uart_download_cfg: bool,
    /// Do not attempt boot from SPI/SD storage: bool,
    pub mediaboot_disable: bool,
    /// Disables bootloader communication via UART
    pub uartboot_disable: bool,
    /// Enable bootloader communication via USB. WARNING: broken in ROM version
    /// Sep 29 2021 17:07:23. Do not set: bool,
    pub usbboot_enable: bool,
    /// Boot ROM debugging: bool,
    pub uart_log_reopen: bool,
    pub sign_cf: bool,
    /// Disable M0 dcache
    pub dcache_disable: bool,
    /// JTAG pin configuration. 0: GPIO16-19, 1: GPIO6/7/12/13, 2/3: disabled
    #[bits(2)]
    pub jtag_cfg: u8,
    pub fix_key_sel: bool,
    /// Enable boot from SD card (untested)
    pub sdh_en: bool,
    /// Flash IO pin configuration, equivalent to enum SF_Ctrl_Pin_Select
    /// https://github.com/bouffalolab/bouffalo_sdk/
    /// 9e189b69cbc0a75ffa170f600a28820848d56432
    /// drivers/soc/bl808/std/include/bl808_sf_ctrl.h#L66-L76
    #[bits(5)]
    pub spi_flash_pin_cfg: u8,
    /// Bootloader entry GPIO polarity. 0: active high, 1: active low
    pub boot_level_revert: bool,
    /// Time to wait between configuring and sampling bootloader entry GPIO.
    /// 0: 5us, 1: 10us, 2: 100us, 3: 500us
    #[bits(2)]
    pub boot_pin_dly: u8,
    /// Apply LDO18 trimming from eFuse (0x78, see F_Ctrl_Read_LDO18IO_Vout_Trim
    pub ldo_trim_enable: bool,
    /// Apply RC32m trimming from eFuse (0x00, see F_Ctrl_Read_Xtal_Trim_RC32M
    pub trim_enable: bool,
    pub no_hd_boot_en: bool,
    /// Time to wait after power-cycling the flash (via GLB_PU_LDO18FLASH).
    /// 0: none, 1: 1ms, 2: 8ms, 3: 16ms
    #[bits(2)]
    pub flash_power_delay: u8,
    /// Wide-ranging effects. Disables some bootloader protocol commands
    /// (such as WRITE_MEMORY). Disallows ROM-based setup of cores other than M0.
    pub tz_boot: bool,
    pub encrypted_tz_boot: bool,
    pub hbn_check_sign: bool,
    /// Code at 0x900140b0. Executed before jumping to user code.
    /// Sets TZC_SEC_TZC_SBOOT_DONE to all-ones.
    pub keep_dbg_port_closed: bool,
    pub hbn_jump_disable: bool,
}

#[bitfield(u32)]
#[derive(FromBytes, IntoBytes)]
pub struct SwConfig1 {
    #[bits(3)]
    pub xtal_type: u8,
    pub wifipll_pu: bool,
    pub aupll_pu: bool,
    pub cpupll_pu: bool,
    pub mipipll_pu: bool,
    pub uhspll_pu: bool,
    #[bits(3)]
    pub mcu_clk: u8,
    pub mcu_clk_div: bool,
    #[bits(2)]
    pub mcu_pbclk_div: u8,
    pub lp_div: bool,
    #[bits(2)]
    pub dsp_clk: u8,
    pub dsp_clk_div: bool,
    #[bits(2)]
    pub dsp_pbclk: u8,
    #[bits(2)]
    pub emi_clk: u8,
    pub emi_clk_div: bool,
    #[bits(3)]
    pub flash_clk_type: u8,
    pub flash_clk_div: bool,
    /// Sets GLB_LDO18FLASH_BYPASS
    pub ldo18flash_bypass_cfg: bool,
    /// Boot ROM debug UART (UART1) output pin. 0: GPIO39, 1: GPIO8
    pub bootlog_pin_cfg: bool,
    /// Bootloader UART autobaud tolerance (see UART_SetAllowableError0X55). 0: 7, 1: 3
    pub abt_offset: bool,
    /// Boot pin pull direction. 0: down, 1: up
    pub boot_pull_cfg: bool,
    /// Disable USB interrupts before jumping to user code
    pub usb_if_int_disable: bool,
}

#[derive(Clone, Debug, Copy, FromBytes, IntoBytes)]
struct SwConfig {
    sw_config0: SwConfig0,
    sw_config1: SwConfig1,
    // TODO: What is this for?
    _sw_config2: u32,
    _sw_config3: u32,
}

impl Display for SwConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cfg0 = self.sw_config0;
        let cfg1 = self.sw_config1;
        write!(f, "{cfg0:#?}\n{cfg1:#?}")
    }
}

#[bitfield(u32)]
#[derive(FromBytes, IntoBytes)]
pub struct Data0Lock {
    #[bits(4)]
    ef_sec_lifecycle: u8,
    #[bits(10)]
    _wr_lock_reserved_0: u16,
    wr_lock_boot_mode: bool,
    wr_lock_dbg_pwd: bool,
    wr_lock_wifi_mac: bool,
    wr_lock_key_slot_0: bool,
    wr_lock_key_slot_1: bool,
    wr_lock_key_slot_2: bool,
    wr_lock_key_slot_3: bool,
    wr_lock_sw_usage_0: bool,
    wr_lock_sw_usage_1: bool,
    wr_lock_sw_usage_2: bool,
    wr_lock_sw_usage_3: bool,
    wr_lock_key_slot_11: bool,
    rd_lock_dbg_pwd: bool,
    rd_lock_key_slot_0: bool,
    rd_lock_key_slot_1: bool,
    rd_lock_key_slot_2: bool,
    rd_lock_key_slot_3: bool,
    rd_lock_key_slot_11: bool,
}

#[bitfield(u32)]
#[derive(FromBytes, IntoBytes)]
pub struct Data1Lock {
    #[bits(15)]
    _reserved: u16,
    wr_lock_key_slot_4: bool,
    wr_lock_key_slot_5: bool,
    wr_lock_key_slot_6: bool,
    wr_lock_key_slot_7: bool,
    wr_lock_key_slot_8: bool,
    wr_lock_key_slot_9: bool,
    wr_lock_key_slot_10: bool,
    _wr_lock_dat_1_rsvd_0: bool,
    _wr_lock_dat_1_rsvd_1: bool,
    _wr_lock_dat_1_rsvd_2: bool,
    rd_lock_key_slot_4: bool,
    rd_lock_key_slot_5: bool,
    rd_lock_key_slot_6: bool,
    rd_lock_key_slot_7: bool,
    rd_lock_key_slot_8: bool,
    rd_lock_key_slot_9: bool,
    rd_lock_key_slot_10: bool,
}

/// https://github.com/bouffalolab/bouffalo_sdk/
/// drivers/soc/bl808/std/include/hardware/ef_data_0_reg.h
#[derive(FromBytes, IntoBytes, Clone, Debug)]
#[repr(C, packed)]
pub struct EfuseBlock0 {
    config: Config,
    debug_password1: u64,
    debug_password2: u64,
    wifi_mac_x: WifiMacAndInfo,
    key0: Key,
    key1: Key,
    key2: Key,
    key3: Key,
    sw_config: SwConfig,
    key11: Key,
    lock: Data0Lock,
}

impl Display for EfuseBlock0 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cfg = self.config;
        let cfg = format!("Config: {cfg:#?}");
        let pw1 = self.debug_password1;
        let pw1 = format!("Password 1: {pw1:016x}");
        let pw2 = self.debug_password2;
        let pw2 = format!("Password 2: {pw2:016x}");

        let macx = self.wifi_mac_x;
        let mac = macx.mac_addr();
        let mac = format!("Wi-Fi MAC: {mac:012x}");

        let package = macx.package();
        let package = format!("Package: {package}");
        let psram = macx.psram();
        let psram = format!("PSRAM: {psram}");
        let flash = macx.flash();
        let flash = format!("Flash: {flash}");
        let version = macx.version();
        let version = format!("Version: {version}");
        let info = format!("{package}\n{psram}\n{flash}\n{version}");

        let sw_cfg = self.sw_config;
        let sw_cfg = format!("SW config: {sw_cfg}");

        let lock = self.lock;
        let lock = format!("Lock: {lock:#?}");

        let key0 = format!("Key 0: {:02x?}", self.key0);
        let key1 = format!("Key 1: {:02x?}", self.key1);
        let key2 = format!("Key 2: {:02x?}", self.key2);
        let key3 = format!("Key 3: {:02x?}", self.key3);
        let key11 = format!("Key 11: {:02x?}", self.key11);

        let keys = format!("{key0}\n{key1}\n{key2}\n{key3}\n{key11}");

        write!(
            f,
            "{cfg}\n{pw1}\n{pw2}\n{mac}\n{info}\n{sw_cfg}\n{lock}\n{keys}"
        )
    }
}

/// https://github.com/bouffalolab/bouffalo_sdk/
/// drivers/soc/bl808/std/include/hardware/ef_data_1_reg.h
#[derive(FromBytes, IntoBytes, Clone, Debug)]
#[repr(C, packed)]
pub struct EfuseBlock1 {
    key4: Key,
    key5: Key,
    key6: Key,
    key7: Key,
    key8: Key,
    key9: Key,
    key10: Key,
    _reserved0: u32,
    _reserved1: u32,
    _reserved2: u32,
    lock: Data1Lock,
}

impl Display for EfuseBlock1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lock = self.lock;
        let lock = format!("Lock: {lock:#?}");

        let key4 = format!("Key 4: {:02x?}", self.key4);
        let key5 = format!("Key 5: {:02x?}", self.key5);
        let key6 = format!("Key 6: {:02x?}", self.key6);
        let key7 = format!("Key 7: {:02x?}", self.key7);
        let key8 = format!("Key 8: {:02x?}", self.key8);
        let key9 = format!("Key 9: {:02x?}", self.key9);
        let key10 = format!("Key 10: {:02x?}", self.key10);

        let keys0 = format!("{key4}\n{key5}\n{key6}\n{key7}");
        let keys1 = format!("{key8}\n{key9}\n{key10}");
        write!(f, "{lock}\n{keys0}\n{keys1}")
    }
}
