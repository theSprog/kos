use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};
use component::crt0::{Entry, Reader};

use crate::start::CRT0_SP;

lazy_static! {
    pub static ref PROC_ENV: Env = Env::init();
}

#[derive(Debug)]
pub struct Env {
    args: Vec<String>,
    envs: BTreeMap<String, String>,
    auxs: Vec<Entry>,
}

impl Env {
    fn init() -> Self {
        // // 测试是否得到内核放在栈上的数据
        let reader = unsafe { Reader::from_ptr(CRT0_SP) };

        let mut reader_arg = reader.done();
        let args: Vec<String> = reader_arg.by_ref().collect();

        let mut reader_env = reader_arg.done();
        let envs: BTreeMap<String, String> = reader_env
            .by_ref()
            .map(|env| {
                let env: Vec<String> = env.split("=").map(|s| s.to_string()).collect();
                assert_eq!(env.len(), 2);
                (env[0].clone(), env[1].clone())
            })
            .collect();

        let mut reader_aux = reader_env.done();
        let auxs: Vec<Entry> = reader_aux.by_ref().collect();
        Self { args, envs, auxs }
    }

    pub fn from(&self) -> Self {
        Self {
            args: self.args.clone(),
            envs: self.envs.clone(),
            auxs: self.auxs.clone(),
        }
    }

    // 外部接口其他调用 new 函数
    pub fn new() -> &'static Env {
        &PROC_ENV
    }

    pub(crate) fn build_env(args: &str, new_env: Option<Env>) -> Env {
        let args = shell_words::split(args).expect("failed to parse args");

        match new_env {
            // args 参数替换
            Some(mut new_env) => {
                new_env.args_mut().clear();
                new_env.args_mut().extend(args);
                new_env
            }
            None => {
                // 否则新建一个
                let mut new_env = Env::from(Env::new());
                new_env.args_mut().clear();
                new_env.args_mut().extend(args);
                new_env
            }
        }
    }

    pub(crate) fn build_c_args(&self) -> (Vec<String>, Vec<*const u8>) {
        // 将末尾加上 \0
        let args_vec = self
            .args()
            .iter()
            .map(|arg| alloc::format!("{}\0", arg))
            .collect::<Vec<_>>();

        // 收集各个字符串的指针
        let mut args_ptr_vec = args_vec
            .iter()
            .map(|arg| (*arg).as_ptr())
            .collect::<Vec<_>>();
        // 最后一个指针设为 null 表示数组结束
        args_ptr_vec.push(core::ptr::null());

        // move 移出作用域延长 args_vec 生命周期
        (args_vec, args_ptr_vec)
    }

    pub(crate) fn build_c_envs(&self) -> (Vec<String>, Vec<*const u8>) {
        let envs_vec: Vec<String> = self
            .envs()
            .iter()
            .map(|(k, v)| alloc::format!("{}={}\0", k, v))
            .collect();
        let mut envs_ptr_vec = envs_vec
            .iter()
            .map(|arg| (*arg).as_ptr())
            .collect::<Vec<_>>();
        envs_ptr_vec.push(core::ptr::null());

        (envs_vec, envs_ptr_vec)
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn args_mut(&mut self) -> &mut Vec<String> {
        &mut self.args
    }

    pub fn envs(&self) -> &BTreeMap<String, String> {
        &self.envs
    }

    pub fn envs_mut(&mut self) -> &mut BTreeMap<String, String> {
        &mut self.envs
    }

    pub fn auxs(&self) -> &[Entry] {
        &self.auxs
    }

    pub fn auxs_mut(&mut self) -> &mut Vec<Entry> {
        &mut self.auxs
    }
}
