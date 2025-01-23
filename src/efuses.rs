#![allow(non_camel_case_types)]
use std::fmt::{Debug, Display, Formatter};

// NOTE: bitfields/bitflags/... are *not* trivial. See also:
// https://hecatia-elegua.github.io/blog/no-more-bit-fiddling/#how-bilge-came-to-be
// https://github.com/google/zerocopy/issues/388
// https://github.com/google/zerocopy/issues/1497
use bitfield_struct::bitfield;
use zerocopy::FromBytes;
use zerocopy_derive::{FromBytes, IntoBytes};

#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum SecureBootEnable {
    No = 0,
    X1 = 1,
    X2 = 2,
    X3 = 3,
}

impl SecureBootEnable {
    const fn into_bits(self) -> u64 {
        self as _
    }
    const fn from_bits(value: u64) -> Self {
        match value {
            0 => Self::No,
            1 => Self::X1,
            2 => Self::X2,
            3 => Self::X3,
            _ => unreachable!(),
        }
    }
}

impl Display for SecureBootEnable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let descr = match self {
            Self::No => "none",
            Self::X1 => "Mode 1",
            Self::X2 => "Mode 2",
            Self::X3 => "Mode 3",
            _ => unreachable!(),
        };
        write!(f, "{descr}")
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum AesMode {
    No = 0,
    Aes128 = 1,
    Aes192 = 2,
    Aes256 = 3,
}

impl AesMode {
    const fn into_bits(self) -> u64 {
        self as _
    }
    const fn from_bits(value: u64) -> Self {
        match value {
            0 => Self::No,
            1 => Self::Aes128,
            2 => Self::Aes192,
            3 => Self::Aes256,
            _ => unreachable!(),
        }
    }
}

impl Display for AesMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let descr = match self {
            Self::No => "none",
            Self::Aes128 => "AES 128",
            Self::Aes192 => "AES 192",
            Self::Aes256 => "AES 256",
            _ => unreachable!(),
        };
        write!(f, "{descr}")
    }
}

#[bitfield(u32)]
#[derive(FromBytes, IntoBytes)]
pub struct Config {
    #[bits(2)]
    pub spi_flash_aes_mode: AesMode,
    pub ai_dis: bool,
    pub cpu0_dis: bool,
    #[bits(2)]
    pub secure_boot_enable: SecureBootEnable,
    #[bits(4)]
    pub uart_dis: u8,
    pub ble2_dis: bool,
    pub m1542_dis: bool,
    #[bits(2)]
    pub sf_key_re_sel: u8,
    pub sdu_dis: bool,
    pub btdm_dis: bool,
    pub wifi_dis: bool,
    pub x_0_key_enc_en: bool,
    pub cam_dis: bool,
    pub m154_dis: bool,
    // This should be used as the highest of 3 bits to evaluate PSRAM info.
    pub cpu1_dis: bool,
    pub cpu_reset_debug_dis: bool,
    pub se_debug_dis: bool,
    pub efuse_debug_dis: bool,
    #[bits(2)]
    pub debug_jtag_1_dis: u8,
    #[bits(2)]
    pub debug_jtag_0_dis: u8,
    #[bits(4)]
    pub debug_mode: u8,
}

/// https://github.com/bouffalolab/bouffalo_sdk
/// 76ebf6ffcbc2a81d18dd18eb3a22810779edae1a
/// drivers/soc/bl808/std/src/bl808_ef_cfg.c#L159
#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Flash {
    No = 0,
    X_8MB = 1,
    Error = 3,
}

impl Flash {
    const fn into_bits(self) -> u64 {
        self as _
    }
    const fn from_bits(value: u64) -> Self {
        match value {
            0 => Self::No,
            1 => Self::X_8MB,
            _ => Self::Error,
        }
    }
}

impl Display for Flash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let descr = match self {
            Self::No => "none",
            Self::X_8MB => "8MB",
            _ => "ERROR",
        };
        write!(f, "{descr}")
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Psram {
    No = 0,
    WB_4MB = 1,
    UHS_32MB = 2,
    UHS_64MB = 3,
    WB_32MB = 4,
    WB_16MB = 5,
    Error = 6,
}

impl Psram {
    const fn into_bits(self) -> u64 {
        self as _
    }

    const fn from_bits(value: u64) -> Self {
        match value {
            0 => Self::No,
            1 => Self::WB_4MB,
            2 => Self::UHS_32MB,
            3 => Self::UHS_64MB,
            4 => Self::WB_32MB,
            5 => Self::WB_16MB,
            _ => Self::Error,
        }
    }

    pub fn from_u64(value: u64) -> Self {
        Self::from_bits(value)
    }
}

impl Display for Psram {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let descr = match self {
            Self::No => "none",
            Self::WB_4MB => "4MB WB",
            Self::UHS_32MB => "32MB UHS",
            Self::UHS_64MB => "64MB UHS",
            Self::WB_32MB => "32MB WB",
            Self::WB_16MB => "16MB WB",
            _ => "ERROR",
        };
        write!(f, "{descr}")
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Package {
    QFN68 = 0,
    QFN88_808C = 1,
    QFN88_808D = 2,
    QFN88_608P = 3,
    Error = 4,
}

impl Package {
    const fn into_bits(self) -> u64 {
        self as _
    }
    const fn from_bits(value: u64) -> Self {
        match value {
            0 => Self::QFN68,
            1 => Self::QFN88_808C,
            2 => Self::QFN88_808D,
            3 => Self::QFN88_608P,
            _ => Self::Error,
        }
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let descr = match self {
            Self::QFN68 => "QFN68",
            Self::QFN88_808C => "QFN88 (808C)",
            Self::QFN88_808D => "QFN88 (808D)",
            Self::QFN88_608P => "QFN88 (608P)",
            _ => "ERROR",
        };
        write!(f, "{descr}")
    }
}

// https://github.com/bouffalolab/bouffalo_sdk
// 76ebf6ffcbc2a81d18dd18eb3a22810779edae1a
// drivers/soc/bl808/std/src/bl808_ef_cfg.c#L150
#[bitfield(u16)]
#[derive(FromBytes, IntoBytes)]
pub struct Info {
    #[bits(6)]
    _unused: u8,
    #[bits(3)]
    pub package: Package,
    // NOTE: This bitfield in itself is not sufficient.
    // These 2 lowest bits of PSRAM info need another bit from Config to create
    // the full value as per vendor code, though that bit is called `cpu1_dis`.
    #[bits(2)]
    pub psram_low: u8,
    #[bits(2)]
    pub flash: Flash,
    #[bits(3)]
    pub version: u8,
}

impl Display for Info {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let package = self.package();
        let package = format!("Package: {package}");
        let flash = self.flash();
        let flash = format!("Flash: {flash}");
        let version = self.version();
        let version = format!("Version: {version}");

        write!(f, "{package}\n{flash}\n{version}")
    }
}

#[bitfield(u64)]
#[derive(FromBytes, IntoBytes)]
pub struct WifiMacAndInfo {
    // TODO: There might be some built-in that we can use here.
    // NOTE: Bouffalo Lab MAC prefix is b4:0e:cf
    // https://macaddress.io/macaddress/B4:0E:CF
    #[bits(48)]
    pub mac_addr: u64,
    #[bits(16)]
    pub info: Info,
}

// TODO
type Key = [u8; 16];

/// https://github.com/bouffalolab/bouffalo_sdk/
/// drivers/lhal/src/flash/bflb_sf_ctrl.h
/// formerly (`9e189b69cbc0a75ffa170f600a28820848d56432`):
/// drivers/soc/bl808/std/include/bl808_sf_ctrl.h#L66-L76
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FlashPinCfg {
    /// embedded flash, swap io0 with io3
    EmbeddedSwapIO0IO3 = 0x0,
    /// embedded flash, swap dual io0 with io3
    EmbeddedSwapDualIO0IO3 = 0x1,
    /// embedded flash, no swap
    EmbeddedSwapNone = 0x2,
    /// embedded flash, no swap and use dual io0
    EmbeddedSwapNoneDualIO0 = 0x3,
    /// external flash, SF2 via gpio34-39
    ExternalSF2 = 0x4,
    /// embedded flash, swap io0 with io3 and external SF2 via gpio34-39
    EmbeddedSwapIO0IO3AndExternalSF2 = 0x14,
    /// embedded flash, swap dual io0 with io3 and external SF2 via gpio34-39
    EmbeddedSwapDualIO0IO3AndExternalSF2 = 0x15,
    /// embedded flash, no swap and external SF2 via gpio34-39
    EmbeddedSwapNoneAndExternalSF2 = 0x16,
    /// embedded flash, no swap, use dual io0 and external SF2 via gpio34-39
    EmbeddedSwapNoneDualIO0AndExternalSF2 = 0x17,
    Invalid = 0x1f,
}

impl FlashPinCfg {
    const fn into_bits(self) -> u64 {
        self as _
    }
    const fn from_bits(value: u64) -> Self {
        match value {
            0x00 => Self::EmbeddedSwapIO0IO3,
            0x01 => Self::EmbeddedSwapDualIO0IO3,
            0x02 => Self::EmbeddedSwapNone,
            0x03 => Self::EmbeddedSwapNoneDualIO0,
            0x04 => Self::ExternalSF2,
            0x14 => Self::EmbeddedSwapIO0IO3AndExternalSF2,
            0x15 => Self::EmbeddedSwapDualIO0IO3AndExternalSF2,
            0x16 => Self::EmbeddedSwapNoneAndExternalSF2,
            0x17 => Self::EmbeddedSwapNoneDualIO0AndExternalSF2,
            // NOTE: 2 bits only, so this will not occur
            _ => Self::Invalid,
        }
    }
}

/// Time to wait between configuring and sampling bootloader entry GPIO.
#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum BootPinDelay {
    Delay5us = 0,
    Delay10us = 1,
    Delay100us = 2,
    Delay500us = 3,
}

impl BootPinDelay {
    const fn into_bits(self) -> u64 {
        self as _
    }
    const fn from_bits(value: u64) -> Self {
        match value {
            0 => Self::Delay5us,
            1 => Self::Delay10us,
            2 => Self::Delay100us,
            3 => Self::Delay500us,
            // NOTE: 2 bits only, so this will not occur
            _ => unreachable!(),
        }
    }
}

impl Debug for BootPinDelay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for BootPinDelay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let delay = match self {
            Self::Delay5us => 5,
            Self::Delay10us => 10,
            Self::Delay100us => 100,
            Self::Delay500us => 500,
            _ => unreachable!(),
        };
        write!(f, "{delay} microseconds")
    }
}

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
    pub media_boot_disable: bool,
    /// Disables bootloader communication via UART
    pub uart_boot_disable: bool,
    /// Enable bootloader communication via USB. WARNING: broken in ROM version
    /// Sep 29 2021 17:07:23. Do not set: bool,
    pub usb_boot_enable: bool,
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
    /// Flash IO pin configuration
    #[bits(5)]
    pub spi_flash_pin_cfg: FlashPinCfg,
    /// Bootloader entry GPIO polarity. 0: active high, 1: active low
    pub boot_level_revert: bool,
    #[bits(2)]
    pub boot_pin_delay: BootPinDelay,
    /// Apply LDO18 trimming from eFuse (0x78, see F_Ctrl_Read_LDO18IO_Vout_Trim
    pub ldo_trim_enable: bool,
    /// Apply RC32m trimming from eFuse (0x00, see F_Ctrl_Read_Xtal_Trim_RC32M)
    pub trim_enable: bool,
    pub no_hd_boot_en: bool,
    /// Time to wait after power-cycling the flash (via GLB_PU_LDO18FLASH).
    /// 0: none, 1: 1ms, 2: 8ms, 3: 16ms
    #[bits(2)]
    pub flash_power_delay: u8,
    /// Wide-ranging effects. Disables some bootloader protocol commands
    /// (such as WRITE_MEMORY). Disallows ROM-based setup of cores other than M0.
    pub trusted_boot: bool,
    pub encrypted_trusted_boot: bool,
    pub hbn_check_sign: bool,
    /// Code at 0x900140b0. Executed before jumping to user code.
    /// Sets TZC_SEC_TZC_SBOOT_DONE to all-ones.
    pub keep_debug_port_closed: bool,
    pub hbn_jump_disable: bool,
}

#[bitfield(u32)]
#[derive(FromBytes, IntoBytes)]
pub struct SwConfig1 {
    #[bits(3)]
    pub xtal_type: u8,
    pub wifi_pll_pu: bool,
    pub au_pll_pu: bool,
    pub cpu_pll_pu: bool,
    pub mipi_pll_pu: bool,
    pub uhs_pll_pu: bool,
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
    /// Boot ROM debug UART (UART1) output pin.
    /// 0: GPIO39, 1: GPIO8
    pub bootlog_pin_cfg: bool,
    /// Bootloader UART autobaud tolerance (see UART_SetAllowableError0X55).
    /// 0: 7, 1: 3
    pub auto_baud_tolerance_offset: bool,
    /// Boot pin pull direction. 0: down, 1: up
    pub boot_pull_cfg: bool,
    /// Disable USB interrupts before jumping to user code
    pub usb_interface_interrupt_disable: bool,
}

#[derive(Clone, Copy, Debug, FromBytes, IntoBytes)]
pub struct SwConfig {
    pub sw_config0: SwConfig0,
    pub sw_config1: SwConfig1,
    // TODO: What is this for?
    pub _sw_config2: u32,
    pub _sw_config3: u32,
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
    wr_lock_debug_password: bool,
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
    rd_lock_debug_password: bool,
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
    pub config: Config,
    pub debug_password1: u64,
    pub debug_password2: u64,
    pub wifi_mac_x: WifiMacAndInfo,
    pub key0: Key,
    pub key1: Key,
    pub key2: Key,
    pub key3: Key,
    pub sw_config: SwConfig,
    pub key11: Key,
    pub lock: Data0Lock,
}

impl Display for EfuseBlock0 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cfg = self.config;
        let psram_high = if cfg.cpu1_dis() { 1 } else { 0 };
        let cfg = format!("Config: {cfg:#?}");
        let pw1 = self.debug_password1;
        let pw1 = format!("Password 1: {pw1:016x}");
        let pw2 = self.debug_password2;
        let pw2 = format!("Password 2: {pw2:016x}");

        let macx = self.wifi_mac_x;
        let mac = macx.mac_addr();
        let mac = format!("Wi-Fi MAC: {mac:012x}");

        let info = macx.info();
        let psram_low = info.psram_low();
        let psram = Psram::from_u64(((psram_high << 2) | psram_low) as u64);
        let psram = format!("PSRAM: {psram}");

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
            "{cfg}\n{pw1}\n{pw2}\n{mac}\n{psram}\n{info}\n{sw_cfg}\n{lock}\n{keys}"
        )
    }
}

/// https://github.com/bouffalolab/bouffalo_sdk/
/// drivers/soc/bl808/std/include/hardware/ef_data_1_reg.h
#[derive(FromBytes, IntoBytes, Clone, Debug)]
#[repr(C, packed)]
pub struct EfuseBlock1 {
    pub key4: Key,
    pub key5: Key,
    pub key6: Key,
    pub key7: Key,
    pub key8: Key,
    pub key9: Key,
    pub key10: Key,
    pub _reserved0: u32,
    pub _reserved1: u32,
    pub _reserved2: u32,
    pub lock: Data1Lock,
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
