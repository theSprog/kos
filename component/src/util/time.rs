use core::fmt::Display;

pub struct TimeUnit;

impl TimeUnit {
    const SECONDS_PER_DAY: u32 = 24 * 60 * 60;
    const SECONDS_PER_HOUR: u32 = 60 * 60;
    const SECONDS_PER_MINUTE: u32 = 60;

    const MINUTES_PER_HOUR: u32 = 60;
    const MINUTES_PER_DAY: u32 = 24 * 60;

    const HOURS_PER_DAY: u32 = 24;
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => panic!("Invalid month: {}", month),
    }
}

pub struct PosixTime {
    inner_time: u64,
}

impl PosixTime {
    pub fn new(posix_time: u64) -> Self {
        Self {
            inner_time: posix_time,
        }
    }

    pub fn parse(&self) -> (u32, u32, u32, u32, u32, u32) {
        // 起始时间是 1970 年 1 月 1 日 00:00:00
        let timestamp = self.inner_time;
        let mut days = (timestamp / TimeUnit::SECONDS_PER_DAY as u64) as u32;
        let mut seconds = (timestamp % TimeUnit::SECONDS_PER_DAY as u64) as u32;

        let mut year = 1970;

        // 计算年份
        loop {
            let days_in_year = if is_leap_year(year) { 366 } else { 365 };

            if days < days_in_year {
                break;
            }

            days -= days_in_year;
            year += 1;
        }

        // 计算月份和日期
        let mut month = 1;
        let mut day = 1;

        loop {
            let days_in_month = days_in_month(year, month);

            if days < days_in_month {
                day += days as u32;
                break;
            }

            days -= days_in_month;
            month += 1;
        }

        // 计算时、分、秒
        let hour = (seconds / TimeUnit::SECONDS_PER_HOUR) as u32;
        seconds %= TimeUnit::SECONDS_PER_HOUR;

        let minute = (seconds / TimeUnit::SECONDS_PER_MINUTE) as u32;
        seconds %= TimeUnit::SECONDS_PER_MINUTE;

        (year, month, day, hour, minute, seconds)
    }
}

pub struct UTC {
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    seconds: u32,
}

impl UTC {
    pub fn from_posix(posix_time: u64) -> Self {
        let (year, month, day, hour, minute, seconds) = PosixTime::new(posix_time).parse();
        Self {
            year,
            month,
            day,
            hour,
            minute,
            seconds,
        }
    }
}

impl Display for UTC {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // 格式化输出
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.seconds
        )
    }
}

pub struct LocalTime {
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    seconds: u32,
}

impl LocalTime {
    // 中国时区 utc+8
    pub fn from_posix(posix_time: u64) -> Self {
        let (year, month, day, hour, minute, seconds) =
            PosixTime::new(posix_time + 8 * TimeUnit::SECONDS_PER_HOUR as u64).parse();

        Self {
            year,
            month,
            day,
            hour,
            minute,
            seconds,
        }
    }
}

impl Display for LocalTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.seconds
        )
    }
}
