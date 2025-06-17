pub const MAX_HARTS: usize = 8; // 最大支持的 hart 数量
pub const PLIC_MAX_IRQ: usize = 511;

/// 每个 hart 拥有 2 个 context：M-mode + S-mode
pub const CONTEXT_PER_HART: usize = 2;

/// 每个 context 的 enable 区域占用 0x80 字节（= 32 个 u32，1024 bit）
pub const CONTEXT_ENABLE_STRIDE: usize = 0x80; // 每个 context enable 区域大小

/// 每个 context 的 threshold + claim 寄存器 block 大小为 4K（0x1000）
pub const CONTEXT_STRIDE: usize = 0x1000; // 每个 context threshold/claim block 的偏移跨度

pub const PLIC_PRIO_BEGIN: usize = 0x0000;
pub const PLIC_PRIO_END: usize = 0x0FFF;

pub const PLIC_PENDING_BEGIN: usize = 0x1000;
pub const PLIC_PENDING_END: usize = 0x1FFF;

pub const PLIC_ENABLE_BEGIN: usize = 0x2000;
pub const PLIC_ENABLE_END: usize = 0x1f_ffff;

pub const PLIC_THRESHOLD_CLAIM_BEGIN: usize = 0x20_0000;
pub const PLIC_THRESHOLD_CLAIM_END: usize = 0x3f_ffff;


