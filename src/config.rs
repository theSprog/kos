// 用户栈大小, 64K
pub const USER_STACK_SIZE: usize = 4096 * 16;
// 内核栈大小, 32K
pub const KERNEL_STACK_SIZE: usize = 4096 * 8;
// 最多允许 8 个 app
pub const MAX_APP_NUM: usize = 8;
// 起始基地址
pub const BASE_ADDRESS: usize = 0x82000000;
// 每个 app 的 size 上限, 128K
pub const APP_SIZE_LIMIT: usize = 0x20000;
