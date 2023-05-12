use riscv::register::{
    mstatus,
    sstatus::{self, Sstatus, SPP},
};

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32], // 通用寄存器
    pub sstatus: Sstatus,
    pub sepc: usize, // 返回值 pc
}

impl TrapContext {
    /// set stack pointer to x_2 reg (sp)
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    /// init app context
    /// entry: app 入口, 即第一条指令地址
    /// sp: 用户栈指针
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        // CSR sstatus
        let sstatus = sstatus::read();
        //设置返回的特权级：User mode。换句话说返回后( sret )进入 User 态
        assert_eq!(sstatus.spp(), SPP::User);
        let mut ctx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry, // app 入口点
        };
        ctx.set_sp(sp); // app 的用户态栈指针
        ctx // return initial Trap Context of app
    }
}
