#![no_main]
#![no_std]

/// 时钟频率, 机器每秒执行 CLOCK_FREQ 这么多 cycle
/// 因此 CLOCK_FREQ 可以理解为一秒
pub const CLOCK_FREQ: usize = 10000000;

// 微秒单位
pub const MICRO_UNIT: usize = CLOCK_FREQ / 1_000_000;
// 毫秒单位
pub const MILLI_UNIT: usize = CLOCK_FREQ / 1_000;
// 秒单位
pub const SECOND_UNIT: usize = CLOCK_FREQ;

/// 内存映射 I/O (MMIO, Memory-Mapped I/O)
/// 外设的设备寄存器可以通过特定的物理内存地址来访问，
/// 每个外设的设备寄存器都分布在没有交集的一个或数个物理地址区间中，
/// 不同外设的设备寄存器所占的物理地址空间也不会产生交集，
/// 且这些外设物理地址区间也不会和 RAM 的物理内存所在的区间存在交集
///
/// VirtIO 外设总线的 MMIO 物理地址区间为从 0x10001000 开头的 4KiB
/// 为了能够在内核中访问 VirtIO 外设总线，我们就必须在内核地址空间中对特定内存区域提前进行映射
/// qemu riscv 的源码如下
/// static const MemMapEntry virt_memmap[] = {
///     [VIRT_DEBUG] =        {        0x0,         0x100 },
///     [VIRT_MROM] =         {     0x1000,        0xf000 },
///     [VIRT_TEST] =         {   0x100000,        0x1000 },
///     [VIRT_RTC] =          {   0x101000,        0x1000 },
///     [VIRT_CLINT] =        {  0x2000000,       0x10000 },
///     [VIRT_ACLINT_SSWI] =  {  0x2F00000,        0x4000 },
///     [VIRT_PCIE_PIO] =     {  0x3000000,       0x10000 },
///     [VIRT_PLATFORM_BUS] = {  0x4000000,     0x2000000 },
///     [VIRT_PLIC] =         {  0xc000000, VIRT_PLIC_SIZE(VIRT_CPUS_MAX * 2) },
///     [VIRT_APLIC_M] =      {  0xc000000, APLIC_SIZE(VIRT_CPUS_MAX) },
///     [VIRT_APLIC_S] =      {  0xd000000, APLIC_SIZE(VIRT_CPUS_MAX) },
///     [VIRT_UART0] =        { 0x10000000,         0x100 },
///     [VIRT_VIRTIO] =       { 0x10001000,        0x1000 },
///     [VIRT_FW_CFG] =       { 0x10100000,          0x18 },
///     [VIRT_FLASH] =        { 0x20000000,     0x4000000 },
///     [VIRT_IMSIC_M] =      { 0x24000000, VIRT_IMSIC_MAX_SIZE },
///     [VIRT_IMSIC_S] =      { 0x28000000, VIRT_IMSIC_MAX_SIZE },
///     [VIRT_PCIE_ECAM] =    { 0x30000000,    0x10000000 },
///     [VIRT_PCIE_MMIO] =    { 0x40000000,    0x40000000 },
///     [VIRT_DRAM] =         { 0x80000000,           0x0 },
/// };
///
// pub const MMIO: &[(usize, usize)] = &[(0x10001000, 0x9000)];

pub const MMIO: &[(usize, usize)] = &[
    // (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    // (0x2000000, 0x10000),
    // (0xc000000, 0x210000), // VIRT_PLIC in virt machine
    (0x10000000, 0x1000), // UART 0
    (0x10001000, 0x8000), // VIRTIO0~7
];
