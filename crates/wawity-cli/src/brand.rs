use std::io::{stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub const RESET: &str = "\x1b[0m";

pub const VIOLET: (u8, u8, u8) = (168, 120, 255);
pub const CYAN: (u8, u8, u8) = (86, 204, 242);
pub const GREEN: (u8, u8, u8) = (86, 220, 150);
pub const RED: (u8, u8, u8) = (255, 107, 107);
pub const AMBER: (u8, u8, u8) = (255, 193, 94);
pub const GREY: (u8, u8, u8) = (128, 130, 150);
pub const WHITE: (u8, u8, u8) = (232, 232, 242);

const GRAD: [(u8, u8, u8); 6] = [
    (176, 118, 255),
    (156, 132, 255),
    (134, 152, 255),
    (112, 174, 252),
    (94, 194, 246),
    (78, 212, 238),
];

const ROWS: [&str; 6] = [
    "\u{2588}\u{2588}\u{2557}    \u{2588}\u{2588}\u{2557} \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557} \u{2588}\u{2588}\u{2557}    \u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2557}   \u{2588}\u{2588}\u{2557}",
    "\u{2588}\u{2588}\u{2551}    \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2551}    \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2551}\u{255a}\u{2550}\u{2550}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{255d}\u{255a}\u{2588}\u{2588}\u{2557} \u{2588}\u{2588}\u{2554}\u{255d}",
    "\u{2588}\u{2588}\u{2551} \u{2588}\u{2557} \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2551} \u{2588}\u{2557} \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}    \u{255a}\u{2588}\u{2588}\u{2588}\u{2588}\u{2554}\u{255d} ",
    "\u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}     \u{255a}\u{2588}\u{2588}\u{2554}\u{255d}  ",
    "\u{255a}\u{2588}\u{2588}\u{2588}\u{2554}\u{2588}\u{2588}\u{2588}\u{2554}\u{255d}\u{2588}\u{2588}\u{2551}  \u{2588}\u{2588}\u{2551}\u{255a}\u{2588}\u{2588}\u{2588}\u{2554}\u{2588}\u{2588}\u{2588}\u{2554}\u{255d}\u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}      \u{2588}\u{2588}\u{2551}   ",
    " \u{255a}\u{2550}\u{2550}\u{255d}\u{255a}\u{2550}\u{2550}\u{255d} \u{255a}\u{2550}\u{255d}  \u{255a}\u{2550}\u{255d} \u{255a}\u{2550}\u{2550}\u{255d}\u{255a}\u{2550}\u{2550}\u{255d} \u{255a}\u{2550}\u{255d}   \u{255a}\u{2550}\u{255d}      \u{255a}\u{2550}\u{255d}   ",
];

const SPARKS: [&str; 4] = [
    "\u{b7}  \u{2726}  \u{2da}",
    "\u{2da}  \u{b7}  \u{2726}",
    "\u{2726}  \u{2da}  \u{b7}",
    "\u{2da}  \u{2726}  \u{b7}",
];

const FACES: [&str; 4] = [
    "( \u{25d5}\u{203f}\u{25d5} )",
    "( \u{25d5}\u{203f}\u{25d5} )",
    "( \u{2d8}\u{203f}\u{2d8} )",
    "( \u{25d5}\u{203f}\u{25d5} )",
];

const TAILS: [&str; 4] = [
    " \u{2570}\u{2500}\u{256f} ",
    "  \u{2570}\u{2500}\u{256f}",
    " \u{2570}\u{2500}\u{256f} ",
    "\u{2570}\u{2500}\u{256f}  ",
];

const SPIN: [&str; 10] = [
    "\u{280b}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283c}", "\u{2834}", "\u{2826}", "\u{2827}",
    "\u{2807}", "\u{280f}",
];

pub fn init() {
    enable_vt();
}

#[cfg(windows)]
fn enable_vt() {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetStdHandle(handle: u32) -> isize;
        fn GetConsoleMode(handle: isize, mode: *mut u32) -> i32;
        fn SetConsoleMode(handle: isize, mode: u32) -> i32;
        fn SetConsoleOutputCP(page: u32) -> i32;
        fn SetConsoleCP(page: u32) -> i32;
    }
    unsafe {
        SetConsoleOutputCP(65001);
        SetConsoleCP(65001);
        let handle = GetStdHandle(0xFFFF_FFF5);
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) != 0 {
            SetConsoleMode(handle, mode | 0x0004);
        }
    }
}

#[cfg(not(windows))]
fn enable_vt() {}

pub fn fg(color: (u8, u8, u8), text: &str) -> String {
    format!(
        "\x1b[38;2;{};{};{}m{}{}",
        color.0, color.1, color.2, text, RESET
    )
}

pub fn bold(text: &str) -> String {
    format!("\x1b[1m{}{}", text, RESET)
}

pub fn dim(text: &str) -> String {
    format!("\x1b[2m{}{}", text, RESET)
}

pub fn accent(text: &str) -> String {
    bold(&fg(VIOLET, text))
}

pub fn width() -> usize {
    crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(90)
}

pub fn clear() {
    print!("\x1b[2J\x1b[3J\x1b[H");
    let _ = stdout().flush();
}

pub fn hide_cursor() {
    print!("\x1b[?25l");
    let _ = stdout().flush();
}

pub fn show_cursor() {
    print!("\x1b[?25h");
    let _ = stdout().flush();
}

pub fn rule() {
    let w = width().min(74).saturating_sub(4).max(20);
    println!("  {}", fg(GREY, &"\u{2500}".repeat(w)));
}

pub fn heading(text: &str) {
    println!();
    println!("  {}", bold(&fg(WHITE, text)));
    rule();
}

pub fn banner() {
    println!();
    if width() < 62 {
        println!(
            "  {}",
            bold(&fg(VIOLET, "W A W I T Y"))
        );
    } else {
        for (i, row) in ROWS.iter().enumerate() {
            println!("  {}", fg(GRAD[i], row));
        }
    }
    println!(
        "  {}  {}",
        fg(CYAN, "secure tunnel console"),
        dim(&format!("v{}", env!("CARGO_PKG_VERSION")))
    );
    println!();
}

fn mascot_frame(i: usize) -> [String; 3] {
    let s = SPARKS[i % SPARKS.len()];
    let f = FACES[i % FACES.len()];
    let t = TAILS[i % TAILS.len()];
    [
        format!("   {}", fg(AMBER, s)),
        format!("  {}", bold(&fg(CYAN, f))),
        format!("   {}", fg(VIOLET, t)),
    ]
}

pub fn intro() {
    hide_cursor();
    println!();
    if width() >= 62 {
        for (i, row) in ROWS.iter().enumerate() {
            println!("  {}", fg(GRAD[i], row));
            let _ = stdout().flush();
            thread::sleep(Duration::from_millis(38));
        }
    } else {
        println!("  {}", bold(&fg(VIOLET, "W A W I T Y")));
    }
    println!(
        "  {}  {}",
        fg(CYAN, "secure tunnel console"),
        dim(&format!("v{}", env!("CARGO_PKG_VERSION")))
    );
    println!();
    println!();
    println!();
    println!();
    for i in 0..10 {
        let frame = mascot_frame(i);
        print!("\x1b[3A");
        for line in &frame {
            print!("\r\x1b[2K{}\n", line);
        }
        let _ = stdout().flush();
        thread::sleep(Duration::from_millis(85));
    }
    print!("\x1b[3A");
    let frame = mascot_frame(0);
    for line in &frame {
        print!("\r\x1b[2K{}\n", line);
    }
    println!(
        "  {} {}",
        fg(GREY, "\u{2570}\u{2500}"),
        dim("\u{412}\u{438}\u{441}\u{43f} \u{433}\u{43e}\u{442}\u{43e}\u{432} \u{43a} \u{440}\u{430}\u{431}\u{43e}\u{442}\u{435}")
    );
    show_cursor();
}

pub fn outro() {
    println!();
    let frame = mascot_frame(2);
    for line in &frame {
        println!("{}", line);
    }
    println!(
        "  {}",
        dim("\u{441}\u{435}\u{430}\u{43d}\u{441} \u{437}\u{430}\u{432}\u{435}\u{440}\u{448}\u{451}\u{43d}")
    );
    println!();
    show_cursor();
}

pub fn ok(text: &str) {
    println!("  {} {}", fg(GREEN, "\u{2714}"), text);
}

pub fn err(text: &str) {
    println!("  {} {}", fg(RED, "\u{2716}"), fg(RED, text));
}

pub fn info(text: &str) {
    println!("  {} {}", fg(CYAN, "\u{203a}"), text);
}

pub fn warn(text: &str) {
    println!("  {} {}", fg(AMBER, "!"), fg(AMBER, text));
}

pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Spinner {
    pub fn start(label: &str) -> Spinner {
        let running = Arc::new(AtomicBool::new(true));
        let flag = running.clone();
        let text = label.to_string();
        let handle = thread::spawn(move || {
            let mut i = 0usize;
            print!("\x1b[?25l");
            while flag.load(Ordering::Relaxed) {
                print!(
                    "\r\x1b[2K  {} {}",
                    fg(VIOLET, SPIN[i % SPIN.len()]),
                    dim(&text)
                );
                let _ = stdout().flush();
                i += 1;
                thread::sleep(Duration::from_millis(80));
            }
            print!("\r\x1b[2K\x1b[?25h");
            let _ = stdout().flush();
        });
        Spinner {
            running,
            handle: Some(handle),
        }
    }

    fn halt(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }

    pub fn done(mut self, text: &str) {
        self.halt();
        ok(text);
    }

    pub fn fail(mut self, text: &str) {
        self.halt();
        err(text);
    }

    pub fn stop(mut self) {
        self.halt();
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if self.handle.is_some() {
            self.halt();
        }
    }
}
