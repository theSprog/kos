#![no_std]
#![no_main]

extern crate alloc;
use user_lib::*;

struct ProgressBar<I> {
    iterator: I,
    total_items: usize,
    processed_items: usize,
}

impl<I> ProgressBar<I>
where
    I: Iterator,
{
    const BAR: &str = r"-\|/";

    fn new(iterator: I) -> Self {
        let total_items = iterator.size_hint().1.unwrap_or(0);
        ProgressBar {
            iterator,
            total_items,
            processed_items: 0,
        }
    }
}

impl<I> Iterator for ProgressBar<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iterator.next();

        if self.total_items > 0 {
            let progress = (self.processed_items * 100) / self.total_items;
            let spaces = (progress / 2) as usize;

            print!(
                "\r {} \u{1b}[42m{}\u{1b}[0m [ {}%, {}/{} ]",
                Self::BAR.chars().nth(progress % 4).unwrap(),
                " ".repeat(spaces),
                progress,
                self.processed_items,
                self.total_items
            );

            if progress == 100 {
                println!("");
            }
        }

        self.processed_items += 1;

        item
    }
}

#[no_mangle]
fn main() -> i32 {
    let mut buf = alloc::vec![0u8; 100];
    for _ in ProgressBar::new(buf.iter_mut()) {
        sleep(30)
    }

    0
}
