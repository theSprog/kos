// SPDX-License-Identifier: Apache-2.0

use logger::{debug, info};

use super::*;

use core::marker::PhantomData;
use core::mem::{align_of, size_of};

type Result<T> = core::result::Result<T, OutOfSpace>;

// Internal use only
trait Serializable {
    fn into_buf(self, dst: &mut [u8]) -> Result<usize>;
}

impl Serializable for usize {
    #[inline]
    fn into_buf(self, dst: &mut [u8]) -> Result<usize> {
        let (_prefix, dst, suffix) = unsafe { dst.align_to_mut::<usize>() };
        dst[dst.len().checked_sub(1).ok_or(OutOfSpace)?] = self;
        let len = suffix.len();
        let len = len.checked_add(size_of::<usize>()).ok_or(OutOfSpace)?;
        Ok(len)
    }
}

impl Serializable for u8 {
    #[inline]
    fn into_buf(self, dst: &mut [u8]) -> Result<usize> {
        dst[dst.len().checked_sub(1).ok_or(OutOfSpace)?] = self;
        Ok(1)
    }
}

impl Serializable for &[u8] {
    #[inline]
    fn into_buf(self, dst: &mut [u8]) -> Result<usize> {
        let start = dst.len().checked_sub(self.len()).ok_or(OutOfSpace)?;
        let end = dst.len();
        dst[start..end].copy_from_slice(self);
        Ok(self.len())
    }
}

/// Handle for the stack
///
/// Retains the immutability and immovability of the stack buffer.
///
/// For a reference to the stack suitable for use before execution
/// of a Linux ELF binary, simply `Deref` the `Handle`.
pub struct Handle<'a>(&'a mut [u8], usize);

impl<'a> core::ops::Deref for Handle<'a> {
    type Target = Stack;

    fn deref(&self) -> &Stack {
        #[repr(C, align(16))]
        struct Aligned(u128);

        let (pre, body, _) = unsafe { self.0[self.1..].align_to::<Aligned>() };
        assert!(pre.is_empty());

        unsafe { &*(body.as_ptr() as *const _ as *const _) }
    }
}

/// Builder for the initial stack of a Linux ELF binary
///
/// # Examples
///
/// ```rust
/// use crt0stack::{Builder, Entry};
///
/// let mut stack = [1u8; 512];
/// let stack = stack.as_mut();
///
/// let mut builder = Builder::new(stack);
///
/// builder.push("/init").unwrap();
/// let mut builder = builder.done().unwrap();
///
/// builder.push("HOME=/root").unwrap();
/// let mut builder = builder.done().unwrap();
///
/// let auxv = [
///     Entry::Gid(1000),
///     Entry::Uid(1000),
///     Entry::Platform("x86_64"),
///     Entry::ExecFilename("/init"),
/// ];
/// auxv.iter().for_each(|e| builder.push(e).unwrap());
///
/// let handle = builder.done().unwrap();
/// ```
pub struct Builder<'a, T> {
    stack: &'a mut [u8],
    stack_vbase: usize, // 用户态栈帧的基地址
    data: usize,        // 相对栈底数据偏移
    items: usize,       // Index to the top of the items section
    state: PhantomData<T>,
}

impl<'a, T> Builder<'a, T> {
    // Serializes the input and saves it in the data section.
    // Returns a reference to the serialized input within the data section.
    // 将 val 放在栈的高位
    #[inline]
    fn push_data(&mut self, val: impl Serializable) -> Result<*const ()> {
        let val_len = val.into_buf(&mut self.stack[..self.data])?;
        // 预留空间
        self.data = self.data.checked_sub(val_len).ok_or(OutOfSpace)?;
        if self.data <= self.items {
            Err(OutOfSpace)
        } else {
            Ok((self.stack_vbase + self.data) as *const ())
        }
    }

    // Serializes the input and saves it in the item section.
    #[inline]
    fn push_item(&mut self, val: usize) -> Result<()> {
        let (prefix, dst, _suffix) = {
            let start = self.items;
            let end = self.data;
            unsafe { self.stack[start..end].align_to_mut::<usize>() }
        };
        if dst.is_empty() {
            return Err(OutOfSpace);
        }
        dst[0] = val;
        let len = prefix.len();
        let len = len.checked_add(size_of::<usize>()).ok_or(OutOfSpace)?;
        self.items = self.items.checked_add(len).ok_or(OutOfSpace)?;

        if self.data <= self.items {
            Err(OutOfSpace)
        } else {
            Ok(())
        }
    }
}

impl<'a> Builder<'a, Arg> {
    /// Create a new Builder for the stack
    ///
    /// Needs a sufficiently large byte slice.
    #[inline]
    pub fn new(stack: &'a mut [u8], stack_vbase: usize) -> Self {
        let len = stack.len();
        Self {
            stack,
            data: len,
            stack_vbase,
            items: size_of::<usize>(),
            state: PhantomData,
        }
    }

    /// Push a new `argv` argument
    #[inline]
    pub fn push(&mut self, arg: &str) -> Result<()> {
        self.push_data(0u8)?; // c-str zero byte
        let p = self.push_data(arg.as_bytes())?;
        self.push_item(p as usize)
    }

    /// Advance the Builder to the next step
    #[inline]
    pub fn done(mut self) -> Result<Builder<'a, Env>> {
        // last argv is NULL
        self.push_item(0usize)?;

        // Store argc at the beginning
        let (prefix, dst, _suffix) = {
            let start = 0;
            let end = self.data;
            unsafe { self.stack[start..end].align_to_mut::<usize>() }
        };
        if dst.is_empty() {
            return Err(OutOfSpace);
        }

        // Calculate argc = (self.items - prefix.len()) / size_of::<usize> - 2
        dst[0] = self.items.checked_sub(prefix.len()).ok_or(OutOfSpace)?;
        dst[0] = dst[0].checked_div(size_of::<usize>()).ok_or(OutOfSpace)?;
        dst[0] = dst[0].checked_sub(2).ok_or(OutOfSpace)?;

        Ok(Builder {
            stack: self.stack,
            stack_vbase: self.stack_vbase,
            data: self.data,
            items: self.items,
            state: PhantomData,
        })
    }
}

impl<'a> Builder<'a, Env> {
    /// Add a new environment variable string
    #[inline]
    pub fn push(&mut self, env: &str) -> Result<()> {
        self.push_data(0u8)?; // c-str zero byte
        let p = self.push_data(env.as_bytes())?;
        self.push_item(p as usize)
    }

    /// Advance the Build to the next step
    #[inline]
    pub fn done(mut self) -> Result<Builder<'a, Aux>> {
        // last environ is NULL
        self.push_item(0usize)?;
        Ok(Builder {
            stack: self.stack,
            stack_vbase: self.stack_vbase,
            data: self.data,
            items: self.items,
            state: PhantomData,
        })
    }
}

impl<'a> Builder<'a, Aux> {
    /// Add a new Entry
    #[inline]
    pub fn push(&mut self, entry: &Entry) -> Result<()> {
        let (key, value): (usize, usize) = match entry {
            Entry::Platform(x) => {
                self.push_data(0u8)?;
                (AT_PLATFORM, self.push_data(x.as_bytes())? as _)
            }
            Entry::BasePlatform(x) => {
                self.push_data(0u8)?;
                (AT_BASE_PLATFORM, self.push_data(x.as_bytes())? as _)
            }
            Entry::ExecFilename(x) => {
                self.push_data(0u8)?;
                (AT_EXECFN, self.push_data(x.as_bytes())? as _)
            }
            Entry::Random(x) => (AT_RANDOM, self.push_data(&x[..])? as _),
            Entry::ExecFd(v) => (AT_EXECFD, *v),
            Entry::PHdr(v) => (AT_PHDR, *v),
            Entry::PHent(v) => (AT_PHENT, *v),
            Entry::PHnum(v) => (AT_PHNUM, *v),
            Entry::PageSize(v) => (AT_PAGESZ, *v),
            Entry::Base(v) => (AT_BASE, *v),
            Entry::Flags(v) => (AT_FLAGS, *v),
            Entry::Entry(v) => (AT_ENTRY, *v),
            Entry::NotElf(v) => (AT_NOTELF, *v as usize),
            Entry::Uid(v) => (AT_UID, *v),
            Entry::EUid(v) => (AT_EUID, *v),
            Entry::Gid(v) => (AT_GID, *v),
            Entry::EGid(v) => (AT_EGID, *v),
            Entry::HwCap(v) => (AT_HWCAP, *v),
            Entry::ClockTick(v) => (AT_CLKTCK, *v),
            Entry::Secure(v) => (AT_SECURE, *v as usize),
            Entry::HwCap2(v) => (AT_HWCAP2, *v),
            Entry::SysInfo(v) => (AT_SYSINFO, *v),
            Entry::SysInfoEHdr(v) => (AT_SYSINFO_EHDR, *v),
        };
        self.push_item(key)?;
        self.push_item(value)?;
        Ok(())
    }

    /// Finish the Builder and get the `Handle`
    #[inline]
    pub fn done(mut self) -> Result<Handle<'a>> {
        self.push_item(AT_NULL)?;
        self.push_item(0)?;

        let start_idx = {
            // at the end, copy the items of the item section from the bottom to the top of the stack

            /*

            +------------------------+  len           +------------------------+  len
            |                        |                |                        |
            |          data          |                |          data          |
            |                        |                |                        |
            +------------------------+                +------------------------+
            |                        |                |                        |
            |                        |                |         items          |
            |                        |  +---------->  |                        |
            |                        |                +------------------------+ <---+ stack pointer
            +------------------------+                |                        |
            |                        |                |                        |
            |         items          |                |                        |
            |                        |                |                        |
            +------------------------+  0             +------------------------+  0

            */

            // align down the destination pointer
            let dst_idx = self.data.checked_sub(self.items).ok_or(OutOfSpace)?;

            #[allow(clippy::integer_arithmetic)]
            let align_offset = (&self.stack[dst_idx] as *const _ as usize) % align_of::<Stack>();

            let dst_idx = dst_idx.checked_sub(align_offset).ok_or(OutOfSpace)?;

            // Align the source start index
            #[allow(clippy::integer_arithmetic)]
            let src_start_idx = self.items % size_of::<usize>();

            self.stack.copy_within(src_start_idx..self.items, dst_idx);

            dst_idx
        };
        Ok(Handle(self.stack, start_idx))
    }
}
