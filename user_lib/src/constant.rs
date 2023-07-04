// 在不同的操作系统和编程环境中，换行符和回车符的使用方式可能会有所不同
// 我们的操作系统把 \n 和 \r 都视为两者的结合: 即回车加换行
// 换行 \n, 光标下移
pub const LF: u8 = 0x0au8;
// 回车符 \r, 光标回到开头
pub const CR: u8 = 0x0du8;

// 退格
pub const DL: u8 = 0x7fu8;
pub const BS: u8 = 0x08u8;

pub const WS: u8 = 0x20u8;
pub const WAVES: u8 = 0x7eu8;

pub const ESC: u8 = 0x1bu8;

// ctrl + l 换页
pub const FF: u8 = 0x0cu8;
// ctrl + u 拒绝接受, 清除这一行
pub const NAK: u8 = 0x15u8;
// ctrl + d
pub const EOT: u8 = 0x04u8;

// tab
pub const TAB: u8 = 0x09u8;
