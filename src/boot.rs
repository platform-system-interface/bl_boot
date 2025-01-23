use bitfield_struct::bitfield;
use zerocopy::{FromBytes, IntoBytes};
use zerocopy_derive::{FromBytes, Immutable, IntoBytes};
use sha2::Digest;

pub const M0_LOAD_ADDR: u32 = crate::mem_map::OCRAM_BASE;
// TODO: at the moment, we can only boot from this offset; not sure yet why
pub const D0_LOAD_ADDR: u32 = crate::mem_map::D0_RAM_BASE + 0x7_0000;

const BOOT_M0: bool = true;
const BOOT_D0: bool = false;

const BOOT_MAGIC: &[u8; 4] = b"BFNP";
const FLASH_CONFIG_MAGIC: &[u8; 4] = b"FCFG";
const CLOCK_CONFIG_MAGIC: &[u8; 4] = b"PCFG";

const CRC32: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

#[derive(FromBytes, Immutable, IntoBytes, Clone, Copy, Debug)]
#[repr(C, packed)]
struct FlashConfig {
    magic: u32,
    _0: [u8; 20],
    _1: [u8; 32],
    _2: [u8; 32],
    crc32: u32,
}

impl FlashConfig {
    // An empty config
    pub fn new() -> Self {
        Self {
            magic: 0,
            _0: [0; 20],
            _1: [0; 32],
            _2: [0; 32],
            crc32: 0x74ccea76,
        }
    }
}

#[derive(FromBytes, Immutable, IntoBytes, Clone, Copy, Debug)]
#[repr(C, packed)]
struct ClockConfig {
    magic: u32,
    _0: [u8; 20],
    crc32: u32,
}

impl ClockConfig {
    // An empty config
    pub fn new() -> Self {
        Self {
            magic: 0,
            _0: [0; 20],
            crc32: 0x0fd59b8d,
        }
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
    cmds_en: bool,
    #[bits(2)]
    cmds_wrap_mode: u8,
    #[bits(4)]
    cmds_wrap_len: u8,
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
            .with_cmds_en(true)
            .with_cmds_wrap_mode(2)
            .with_cmds_wrap_len(2)
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
    boot2_partition_table: u64,
    flash_config_table_addr: u32,
    flash_config_table_size: u32,
    patch_config: [u8; 32],
    patch_jump: [u8; 32],
    _reserved: [u8; 20],
    crc32: u32,
}

const BOOT_HEADER_SIZE: usize = std::mem::size_of::<BootHeader>();

impl BootHeader {
    pub fn new(segments: &[Segment]) -> Self {
        let mut h = Self {
            magic: *BOOT_MAGIC,
            revision: 1,
            flash_config: FlashConfig::new(),
            clock_config: ClockConfig::new(),
            boot_config: BootConfig::new(segments),
            m0_config: if BOOT_M0 {
                CpuConfig::with_entry(M0_LOAD_ADDR)
            } else {
                CpuConfig::new()
            },
            d0_config: if BOOT_D0 {
                CpuConfig::with_entry(D0_LOAD_ADDR)
            } else {
                CpuConfig::new()
            },
            lp_config: CpuConfig::new(),
            boot2_partition_table: 0,
            flash_config_table_addr: 0,
            flash_config_table_size: 0,
            patch_config: [0u8; 32],
            patch_jump: [0u8; 32],
            _reserved: [0u8; 20],
            crc32: 0,
        };
        let bytes = &h.as_bytes()[..BOOT_HEADER_SIZE - 4];
        let crc32 = CRC32.checksum(bytes);
        h.crc32 = crc32;
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
