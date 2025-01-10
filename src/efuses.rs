use std::fmt::{Display, Formatter};

// NOTE: bitfields/bitflags/... are *not* trivial. See also:
// https://hecatia-elegua.github.io/blog/no-more-bit-fiddling/#how-bilge-came-to-be
// https://github.com/google/zerocopy/issues/388
// https://github.com/google/zerocopy/issues/1497
use bitfield_struct::bitfield;
use zerocopy::FromBytes;
use zerocopy_derive::{FromBytes, IntoBytes};

// TODO
type Config = u32;

// TODO: There might be some built-in that we can use here.
// NOTE: Bouffalo Lab MAC prefix is b4:0e:cf
// https://macaddress.io/macaddress/B4:0E:CF
#[bitfield(u64)]
#[derive(FromBytes, IntoBytes)]
pub struct WifiMacAndX {
    #[bits(48)]
    pub mac_addr: u64,
    pub x0: u8,
    pub x1: u8,
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
    rest: [u8; 8],
}

impl Display for SwConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cfg0 = self.sw_config0;
        let cfg1 = self.sw_config1;
        let rest = self.rest;
        write!(f, "{cfg0:#?}\n{cfg1:#?}\n{rest:02x?}")
    }
}

// TODO
type Data0Lock = u32;

/// drivers/soc/bl808/std/include/hardware/ef_data_0_reg.h
#[derive(FromBytes, IntoBytes, Clone, Debug)]
#[repr(C, packed)]
pub struct Efuse {
    config: Config,
    debug_password1: u64,
    debug_password2: u64,
    wifi_mac_x: WifiMacAndX,
    key0: Key,
    key1: Key,
    key2: Key,
    key3: Key,
    sw_config: SwConfig,
    key11: Key,
    data_0_lock: Data0Lock,
}

impl Display for Efuse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cfg = self.config;
        let cfg = format!("Config: {cfg:08x?}");
        let pw1 = self.debug_password1;
        let pw1 = format!("Password 1: {pw1:016x}");
        let pw2 = self.debug_password2;
        let pw2 = format!("Password 2: {pw2:016x}");
        let macx = self.wifi_mac_x;
        let mac = macx.mac_addr();
        let mac = format!("Wi-Fi MAC: {mac:012x}");
        let xx = format!("???: {:02x} {:02x}", macx.x0(), macx.x1());

        let sw_cfg = self.sw_config;
        let sw_cfg = format!("SW config: {sw_cfg}");

        let lock = self.data_0_lock;
        let lock = format!("Data 0 lock: {lock:08x}");

        let key0 = format!("Key 0: {:02x?}", self.key0);
        let key1 = format!("Key 1: {:02x?}", self.key1);
        let key2 = format!("Key 2: {:02x?}", self.key2);
        let key3 = format!("Key 3: {:02x?}", self.key3);
        let key11 = format!("Key 11: {:02x?}", self.key11);

        let keys = format!("{key0}\n{key1}\n{key2}\n{key3}\n{key11}");

        write!(
            f,
            "{cfg}\n{pw1}\n{pw2}\n{mac}\n{xx}\n{sw_cfg}\n{lock}\n{keys}"
        )
    }
}
