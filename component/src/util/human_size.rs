use core::{
    cmp,
    fmt::{self, Display},
    marker::PhantomData,
};

use num::{rational::Ratio, FromPrimitive};
use sys_interface::config::{GB, KB, MB};

pub fn debug_size(size: usize) -> DebugSizeFormatter {
    DebugSizeFormatter::new(size)
}

pub fn dec_size(size: usize) -> SizeFormatter<DecSuffix> {
    SizeFormatter::new(size)
}

pub fn bin_size(size: usize) -> SizeFormatter<BinSuffix> {
    SizeFormatter::new(size)
}

pub struct DebugSizeFormatter {
    size: usize,
}
impl DebugSizeFormatter {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl Display for DebugSizeFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = self.size;
        if size < KB {
            write!(f, "{}B", size)
        } else if size < MB {
            let kbs = size / KB;
            let rest = size % KB;
            if rest == 0 {
                write!(f, "{}KiB", kbs)
            } else {
                write!(f, "{}KiB+{}", kbs, debug_size(rest))
            }
        } else if size < GB {
            let mbs = size / MB;
            let rest = size % MB;
            if rest == 0 {
                write!(f, "{}MiB", mbs)
            } else {
                write!(f, "{}MiB+{}", mbs, debug_size(rest))
            }
        } else {
            write!(f, "Too large size for {}", size)
        }
    }
}

const DEFAULT_PRECISION: usize = 2;

pub trait SuffixType {
    const MOD_SIZE: usize;
    fn suffixes() -> [&'static str; 4];
}

pub struct DecSuffix;

impl SuffixType for DecSuffix {
    const MOD_SIZE: usize = 1000;

    fn suffixes() -> [&'static str; 4] {
        ["", "K", "M", "G"]
    }
}

/// Represents the prefixes used for display file sizes using powers of 1024.
pub struct BinSuffix;

impl SuffixType for BinSuffix {
    const MOD_SIZE: usize = 1024;

    fn suffixes() -> [&'static str; 4] {
        ["", "Ki", "Mi", "Gi"]
    }
}

fn int_log(mut num: usize, base: usize) -> usize {
    let mut divisions = 0;

    while num >= base {
        num = num / base;
        divisions += 1;
    }

    divisions
}

struct FormatRatio {
    size: Ratio<usize>,
}

impl FormatRatio {
    /// Creates a new format ratio from the number.
    fn new(size: Ratio<usize>) -> FormatRatio {
        FormatRatio { size }
    }
}

impl Display for FormatRatio {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.size.trunc())?;
        let precision = f.precision().unwrap_or(DEFAULT_PRECISION);

        if precision > 0 {
            write!(f, ".")?;
            let mut frac = self.size.fract();

            for _ in 0..precision {
                if frac.is_integer() {
                    // If the fractional part is an integer, we're done and just need more zeroes.
                    write!(f, "0")?;
                } else {
                    // Otherwise print every digit separately.
                    frac = frac * Ratio::from_u64(10).unwrap();
                    write!(f, "{}", frac.trunc())?;
                    frac = frac.fract();
                }
            }
        }

        Ok(())
    }
}

pub struct SizeFormatter<Suffix: SuffixType> {
    size: usize,
    _marker: PhantomData<Suffix>,
}
impl<Suffix: SuffixType> SizeFormatter<Suffix> {
    fn new(size: usize) -> SizeFormatter<Suffix> {
        Self {
            size,
            _marker: PhantomData,
        }
    }
}
impl<Suffix: SuffixType> Display for SizeFormatter<Suffix> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let max_prefix = Suffix::suffixes().len() - 1;
        let precision = f.precision().unwrap_or(DEFAULT_PRECISION);
        let mod_size = Suffix::MOD_SIZE;

        // Find the right prefix.
        let divisions = cmp::min(int_log(self.size.clone(), mod_size.clone()), max_prefix);

        // Cap the precision to what makes sense.
        let precision = cmp::min(precision, divisions * 3);

        let ratio = Ratio::<usize>::new(self.size.clone(), mod_size.pow(divisions as u32));

        let format_number = FormatRatio::new(ratio);

        write!(
            f,
            "{:.*}{}",
            precision,
            format_number,
            Suffix::suffixes()[divisions]
        )
    }
}
