use alloc::{borrow::ToOwned, collections::BTreeMap, string::String, vec::Vec};
use component::crt0::{Entry, Reader};

use crate::start::CRT0_SP;

lazy_static! {
    pub(crate) static ref ENV: Env = Env::init();
}

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
                let env: Vec<String> = env.split("=").map(|s| s.to_owned()).collect();
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

    // 其他调用 new 函数
    pub fn new() -> &'static Env {
        &ENV
    }

    pub fn args(&self) -> &Vec<String> {
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

    pub fn auxs(&self) -> &Vec<Entry> {
        &self.auxs
    }

    pub fn auxs_mut(&mut self) -> &mut Vec<Entry> {
        &mut self.auxs
    }
}
