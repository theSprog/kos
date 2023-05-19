pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;

// 单页大小
pub const PAGE_SIZE: usize = 4 * KB;
// 单页页宽
pub const PAGE_SIZE_BITS: usize = 12;

// 用户栈大小, 8MB, 由于有了虚拟内存, 可以开大一点
pub const USER_STACK_SIZE: usize = 8 * MB;
// 内核栈大小, 512K, 应该开大一点，因为内核栈有时候会爆栈
// 比如下面的栈经过测试 3KB 会提示内核栈溢出 (canary 机制, 以及分页后的 guard page 机制)
pub const KERNEL_STACK_SIZE: usize = 512 * KB;
