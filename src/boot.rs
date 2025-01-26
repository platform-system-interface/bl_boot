// reference:
// https://github.com/openbouffalo/bflb-mcu-tool
// libs/bl808/bootheader_cfg_keys.py
use bitfield_struct::bitfield;
use sha2::Digest;
use zerocopy::{FromBytes, IntoBytes};
use zerocopy_derive::{FromBytes, Immutable, IntoBytes};

pub const M0_LOAD_ADDR: u32 = crate::mem_map::OCRAM_BASE;
// TODO: at the moment, we can only boot from this offset; not sure yet why
pub const D0_LOAD_ADDR: u32 = crate::mem_map::D0_RAM_BASE + 0x7_0000;
// TODO: try this out; we may not be able to run from here
pub const LP_LOAD_ADDR: u32 = crate::mem_map::OCRAM_BASE + 0x8000;

const BOOT_MAGIC: &[u8; 4] = b"BFNP";
const FLASH_CONFIG_MAGIC: &[u8; 4] = b"FCFG";
const CLOCK_CONFIG_MAGIC: &[u8; 4] = b"PCFG";

const CRC32: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

#[derive(FromBytes, Immutable, IntoBytes, Clone, Copy, Debug)]
#[repr(C, packed)]
struct FlashConfig {
    magic: u32,
    io_mode: u8,
    continuous_read_support: u8,
    sfctrl_clock_delay: u8,
    sfctrl_clock_invert: u8,
    reset_en_command: u8,
    reset_command: u8,
    exit_continuousread_command: u8,
    exit_continuousread_command_size: u8,
    jedec_id_command: u8,
    jedec_id_command_dummy_clock: u8,
    enter_32bits_addr_command: u8,
    exit_32bits_addr_clock: u8,
    sector_size: u8,
    mfg_id: u8,
    page_size: u16,
    chip_erase_command: u8,
    sector_erase_command: u8,
    blk32k_erase_command: u8,
    blk64k_erase_command: u8,

    write_enable_command: u8,
    page_prog_command: u8,
    qpage_prog_command: u8,
    qual_page_prog_addr_mode: u8,

    fast_read_command: u8,
    fast_read_dummy_clock: u8,
    qpi_fast_read_command: u8,
    qpi_fast_read_dummy_clock: u8,

    fast_read_do_command: u8,
    fast_read_do_dummy_clock: u8,
    fast_read_dio_command: u8,
    fast_read_dio_dummy_clock: u8,

    fast_read_qo_command: u8,
    fast_read_qo_dummy_clock: u8,
    fast_read_qio_command: u8,
    fast_read_qio_dummy_clock: u8,

    qpi_fast_read_qio_command: u8,
    qpi_fast_read_qio_dummy_clock: u8,
    qpi_page_prog_command: u8,
    write_vreg_enable_command: u8,

    wel_reg_index: u8,
    qe_reg_index: u8,
    busy_reg_index: u8,
    wel_bit_pos: u8,

    qe_bit_pos: u8,
    busy_bit_pos: u8,
    wel_reg_write_len: u8,
    wel_reg_read_len: u8,

    qe_reg_write_len: u8,
    qe_reg_read_len: u8,
    release_power_down: u8,
    busy_reg_read_len: u8,

    reg_read_command0: u8,
    reg_read_command1: u8,
    reg_write_command0: u8,
    reg_write_command1: u8,

    enter_qpi_command: u8,
    exit_qpi_command: u8,
    continuous_read_code: u8,
    continuous_read_exit_code: u8,

    burst_wrap_command: u8,
    burst_wrap_dummy_clock: u8,
    burst_wrap_data_mode: u8,
    burst_wrap_code: u8,

    de_burst_wrap_command: u8,
    de_burst_wrap_command_dummy_clock: u8,
    de_burst_wrap_code_mode: u8,
    de_burst_wrap_code: u8,

    sector_erase_time: u16,
    blk32k_erase_time: u16,

    blk64k_erase_time: u16,
    page_prog_time: u16,

    chip_erase_time: u16,
    power_down_delay: u8,
    qe_data: u8,

    crc32: u32,
}

impl FlashConfig {
    // An empty config
    pub fn new() -> Self {
        Self {
            magic: 0,

            io_mode: 0,
            continuous_read_support: 0,
            sfctrl_clock_delay: 0,
            sfctrl_clock_invert: 0,

            reset_en_command: 0,
            reset_command: 0,
            exit_continuousread_command: 0,
            exit_continuousread_command_size: 0,

            jedec_id_command: 0,
            jedec_id_command_dummy_clock: 0,
            enter_32bits_addr_command: 0,
            exit_32bits_addr_clock: 0,

            sector_size: 0,
            mfg_id: 0,
            page_size: 0,

            chip_erase_command: 0,
            sector_erase_command: 0,
            blk32k_erase_command: 0,
            blk64k_erase_command: 0,

            write_enable_command: 0,
            page_prog_command: 0,
            qpage_prog_command: 0,
            qual_page_prog_addr_mode: 0,

            fast_read_command: 0,
            fast_read_dummy_clock: 0,
            qpi_fast_read_command: 0,
            qpi_fast_read_dummy_clock: 0,

            fast_read_do_command: 0,
            fast_read_do_dummy_clock: 0,
            fast_read_dio_command: 0,
            fast_read_dio_dummy_clock: 0,

            fast_read_qo_command: 0,
            fast_read_qo_dummy_clock: 0,
            fast_read_qio_command: 0,
            fast_read_qio_dummy_clock: 0,

            qpi_fast_read_qio_command: 0,
            qpi_fast_read_qio_dummy_clock: 0,
            qpi_page_prog_command: 0,
            write_vreg_enable_command: 0,

            wel_reg_index: 0,
            qe_reg_index: 0,
            busy_reg_index: 0,
            wel_bit_pos: 0,

            qe_bit_pos: 0,
            busy_bit_pos: 0,
            wel_reg_write_len: 0,
            wel_reg_read_len: 0,

            qe_reg_write_len: 0,
            qe_reg_read_len: 0,
            release_power_down: 0,
            busy_reg_read_len: 0,

            reg_read_command0: 0,
            reg_read_command1: 0,
            reg_write_command0: 0,
            reg_write_command1: 0,

            enter_qpi_command: 0,
            exit_qpi_command: 0,
            continuous_read_code: 0,
            continuous_read_exit_code: 0,

            burst_wrap_command: 0,
            burst_wrap_dummy_clock: 0,
            burst_wrap_data_mode: 0,
            burst_wrap_code: 0,

            de_burst_wrap_command: 0,
            de_burst_wrap_command_dummy_clock: 0,
            de_burst_wrap_code_mode: 0,
            de_burst_wrap_code: 0,

            sector_erase_time: 0,
            blk32k_erase_time: 0,

            blk64k_erase_time: 0,
            page_prog_time: 0,

            chip_erase_time: 0,
            power_down_delay: 0,
            qe_data: 0,

            crc32: 0x74ccea76,
        }
    }
}

#[derive(FromBytes, Immutable, IntoBytes, Clone, Copy, Debug)]
#[repr(C, packed)]
struct ClockConfig {
    magic: u32,

    xtal_type: u8,
    mcu_clock: u8,
    mcu_clock_divider: u8,
    mcu_bclock_divider: u8,

    mcu_pbclock_divider: u8,
    lp_divider: u8,
    dsp_clock: u8,
    dsp_clock_divider: u8,

    dsp_bclock_divider: u8,
    dsp_pbclock: u8,
    dsp_pbclock_divider: u8,
    emi_clock: u8,

    emi_clock_divider: u8,
    flash_clock_type: u8,
    flash_clock_divider: u8,
    wifi_pll_pu: u8,

    au_pll_pu: u8,
    cpu_pll_pu: u8,
    mipi_pll_pu: u8,
    uhs_pll_pu: u8,

    crc32: u32,
}

const CLOCK_CONFIG_SIZE: usize = size_of::<ClockConfig>();

impl ClockConfig {
    // An empty config
    pub fn new() -> Self {
        let mut c = Self {
            magic: 0,

            xtal_type: 0,
            mcu_clock: 0,
            mcu_clock_divider: 0,
            mcu_bclock_divider: 0,

            mcu_pbclock_divider: 0,
            lp_divider: 0,
            dsp_clock: 0,
            dsp_clock_divider: 0,

            dsp_bclock_divider: 0,
            dsp_pbclock: 0,
            dsp_pbclock_divider: 0,
            emi_clock: 0,

            emi_clock_divider: 0,
            flash_clock_type: 0,
            flash_clock_divider: 0,
            wifi_pll_pu: 0,

            au_pll_pu: 0,
            cpu_pll_pu: 0,
            mipi_pll_pu: 0,
            uhs_pll_pu: 0,

            crc32: 0,
        };
        let bytes = &c.as_bytes()[..CLOCK_CONFIG_SIZE - 4];
        c.crc32 = CRC32.checksum(bytes);
        c
    }
}
#[bitfield(u32)]
#[derive(FromBytes, Immutable, IntoBytes)]
pub struct BootConfigBits {
    #[bits(2)]
    sign: u8,
    #[bits(2)]
    encrypt_type: u8,
    #[bits(2)]
    key_selection: u8,
    xts_mode: bool,
    aes_region_lock: bool,
    no_segment: bool,
    boot2_enable: bool,
    boot2_rollback: bool,
    #[bits(4)]
    cpu_master_id: u8,
    notload_in_bootrom: bool,
    crc_ignore: bool,
    hash_ignore: bool,
    power_on_mm: bool,
    #[bits(3)]
    em_sel: u8,
    commands_en: bool,
    #[bits(2)]
    commands_wrap_mode: u8,
    #[bits(4)]
    commands_wrap_len: u8,
    icache_invalid: bool,
    dcache_invalid: bool,
    fpga_halt_release: bool,
}

#[derive(FromBytes, Immutable, IntoBytes, Clone, Copy, Debug)]
#[repr(C, packed)]
struct BootConfig {
    config: BootConfigBits,
    group_image_offset: u32,
    aes_region_length: u32,
    image_length_or_segment_count: u32,
    sha256: [u8; 32],
}

impl BootConfig {
    pub fn new(segments: &[Segment]) -> Self {
        let image_length_or_segment_count = segments.len() as u32;
        let mut hash = sha2::Sha256::new();
        for s in segments {
            hash.update(s.header.as_bytes());
            hash.update(s.data);
        }
        let sha256: [u8; 32] = hash.finalize().into();
        let mut config = BootConfigBits::new()
            .with_no_segment(true)
            // power on D0 (C096) aka MM aka MultiMedia core
            .with_power_on_mm(true)
            .with_em_sel(1)
            .with_commands_en(true)
            .with_commands_wrap_mode(2)
            .with_commands_wrap_len(2)
            .with_icache_invalid(true)
            .with_dcache_invalid(true);
        Self {
            config,
            group_image_offset: 0,
            aes_region_length: 0,
            image_length_or_segment_count,
            sha256,
        }
    }
}

#[bitfield(u32)]
#[derive(FromBytes, Immutable, IntoBytes)]
pub struct CpuEnableAndCache {
    #[bits(8)]
    config_enable: u8,
    #[bits(8)]
    halt_cpu: u8,
    cache_enable: bool,
    cache_wa: bool,
    cache_wb: bool,
    cache_wt: bool,
    #[bits(4)]
    cache_way_dis: u8,
    #[bits(8)]
    _reserved: u8,
}

#[derive(FromBytes, Immutable, IntoBytes, Clone, Copy, Debug)]
#[repr(C, packed)]
struct CpuConfig {
    cpu_enable_and_cache: CpuEnableAndCache,
    cache_range: u64,
    image_offset: u32,
    boot_entry: u32,
    msp_val: u32,
}

impl CpuConfig {
    pub fn new() -> Self {
        Self {
            cpu_enable_and_cache: CpuEnableAndCache::new(),
            cache_range: 0,
            image_offset: 0,
            boot_entry: 0,
            msp_val: 0,
        }
    }

    pub fn with_entry(boot_entry: u32) -> Self {
        let mut cpu_enable_and_cache = CpuEnableAndCache::new().with_config_enable(1);
        Self {
            cpu_enable_and_cache,
            cache_range: 0,
            image_offset: 0,
            boot_entry,
            msp_val: 0,
        }
    }
}

// NOTE: This is a shortcut. Instead of defining the hundreds of fields, just
// zero out all the irrelevant and be done with it.
#[derive(FromBytes, Immutable, IntoBytes, Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct BootHeader {
    magic: [u8; 4],
    revision: u32,
    flash_config: FlashConfig,
    clock_config: ClockConfig,
    boot_config: BootConfig,
    // E907, 32-bit, where the mask ROM starts
    m0_config: CpuConfig,
    // C906, 64-bit
    d0_config: CpuConfig,
    // Exxx, 32-bit, low-power
    lp_config: CpuConfig,
    boot2_partition_table_0: u32,
    boot2_partition_table_1: u32,
    flash_config_table_addr: u32,
    flash_config_table_size: u32,
    patch_config: [u32; 8],
    patch_jump: [u32; 8],
    _reserved: [u8; 20],
    crc32: u32,
}

const BOOT_HEADER_SIZE: usize = std::mem::size_of::<BootHeader>();

impl BootHeader {
    pub fn new(m0_seg: Option<Segment>, d0_seg: Option<Segment>, lp_seg: Option<Segment>) -> Self {
        let mut segments = Vec::<Segment>::new();
        if let Some(s) = m0_seg {
            segments.push(s);
        }
        if let Some(s) = d0_seg {
            segments.push(s);
        }
        if let Some(s) = lp_seg {
            segments.push(s);
        }
        let mut h = Self {
            magic: *BOOT_MAGIC,
            revision: 1,
            flash_config: FlashConfig::new(),
            clock_config: ClockConfig::new(),
            boot_config: BootConfig::new(&segments),
            m0_config: if m0_seg.is_some() {
                CpuConfig::with_entry(M0_LOAD_ADDR)
            } else {
                CpuConfig::new()
            },
            d0_config: if d0_seg.is_some() {
                CpuConfig::with_entry(D0_LOAD_ADDR)
            } else {
                CpuConfig::new()
            },
            lp_config: if lp_seg.is_some() {
                CpuConfig::with_entry(LP_LOAD_ADDR)
            } else {
                CpuConfig::new()
            },
            boot2_partition_table_0: 0,
            boot2_partition_table_1: 0,
            flash_config_table_addr: 0,
            flash_config_table_size: 0,
            patch_config: [0u32; 8],
            patch_jump: [0u32; 8],
            _reserved: [0u8; 20],
            crc32: 0,
        };
        let bytes = &h.as_bytes()[..BOOT_HEADER_SIZE - 4];
        h.crc32 = CRC32.checksum(bytes);
        h
    }
}

#[derive(FromBytes, Immutable, IntoBytes, Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct SegmentHeader {
    address: u32,
    size: u32,
    _reserved: u32,
    crc32: u32,
}

const SEGMENT_HEADER_SIZE: usize = std::mem::size_of::<SegmentHeader>();

impl SegmentHeader {
    pub fn new(address: u32, size: u32) -> Self {
        let mut h = Self {
            address,
            size,
            _reserved: 0,
            crc32: 0,
        };
        let bytes = &h.as_bytes()[..SEGMENT_HEADER_SIZE - 4];
        let crc32 = CRC32.checksum(bytes);
        h.crc32 = crc32;
        h
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Segment<'a> {
    pub header: SegmentHeader,
    pub data: &'a [u8],
}

impl<'a> Segment<'a> {
    pub fn new(address: u32, data: &'a [u8]) -> Self {
        let size = data.len() as u32;
        Self {
            header: SegmentHeader::new(address, size),
            data,
        }
    }
}
