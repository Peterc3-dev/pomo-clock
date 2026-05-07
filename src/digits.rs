/// 5-line tall ASCII art digits for the countdown display.
/// Each digit is 6 characters wide, colon is 2 characters wide.

const DIGIT_HEIGHT: usize = 5;

pub fn digit_lines(ch: char) -> [&'static str; 5] {
    match ch {
        '0' => [
            "██████",
            "██  ██",
            "██  ██",
            "██  ██",
            "██████",
        ],
        '1' => [
            "    ██",
            "    ██",
            "    ██",
            "    ██",
            "    ██",
        ],
        '2' => [
            "██████",
            "    ██",
            "██████",
            "██    ",
            "██████",
        ],
        '3' => [
            "██████",
            "    ██",
            "██████",
            "    ██",
            "██████",
        ],
        '4' => [
            "██  ██",
            "██  ██",
            "██████",
            "    ██",
            "    ██",
        ],
        '5' => [
            "██████",
            "██    ",
            "██████",
            "    ██",
            "██████",
        ],
        '6' => [
            "██████",
            "██    ",
            "██████",
            "██  ██",
            "██████",
        ],
        '7' => [
            "██████",
            "    ██",
            "    ██",
            "    ██",
            "    ██",
        ],
        '8' => [
            "██████",
            "██  ██",
            "██████",
            "██  ██",
            "██████",
        ],
        '9' => [
            "██████",
            "██  ██",
            "██████",
            "    ██",
            "██████",
        ],
        ':' => [
            "  ",
            "██",
            "  ",
            "██",
            "  ",
        ],
        _ => [
            "      ",
            "      ",
            "      ",
            "      ",
            "      ",
        ],
    }
}

/// Render a time string like "25:00" into 5 lines of ASCII art.
pub fn render_time(minutes: u64, seconds: u64) -> Vec<String> {
    let time_str = format!("{:02}:{:02}", minutes, seconds);
    let mut lines = vec![String::new(); DIGIT_HEIGHT];

    for (i, ch) in time_str.chars().enumerate() {
        let glyph = digit_lines(ch);
        for row in 0..DIGIT_HEIGHT {
            if i > 0 {
                lines[row].push(' ');
            }
            lines[row].push_str(glyph[row]);
        }
    }

    lines
}
