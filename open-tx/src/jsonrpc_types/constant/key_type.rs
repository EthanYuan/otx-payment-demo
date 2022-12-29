/// Meta Map Keys
pub const OTX_META_VERSION: u32 = 0x01;

/// Cell Dep Keys
pub const OTX_OUTPOINT: u32 = 0x02;
pub const OTX_CELL_DEP_TYPE: u32 = 0x03;

/// Header Dep Keys
pub const OTX_HEADER_DEP_HASH: u32 = 0x04;

/// Input Keys
pub const OTX_INPUT_SINCE: u32 = 0x100;

/// Witness Keys
pub const OTX_WITNESS_RAW: u32 = 0x200;
pub const OTX_WITNESS_ARGS: u32 = 0x201;

/// Previous Output or Output Keys
pub const OTX_OUTPUT_CAPACITY: u32 = 0x300;
pub const OTX_OUTPUT_LOCK_CODE_HASH: u32 = 0x301;
pub const OTX_OUTPUT_LOCK_HASH_TYPE: u32 = 0x302;
pub const OTX_OUTPUT_LOCK_ARGS: u32 = 0x303;
pub const OTX_OUTPUT_TYPE_CODE_HASH: u32 = 0x304;
pub const OTX_OUTPUT_TYPE_HASH_TYPE: u32 = 0x305;
pub const OTX_OUTPUT_TYPE_ARGS: u32 = 0x306;
pub const OTX_OUTPUT_DATA: u32 = 0x307;
