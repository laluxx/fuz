use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal::{self, disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    ExecutableCommand,
};
use std::io::{self, stdout, BufRead, Stdout, Write};

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    let (_, start_row) = cursor::position()?;

    let stdin = io::stdin();
    let lines: Vec<String> = stdin.lock().lines().filter_map(Result::ok).collect();
    let mut query = String::new();
    let mut selected_line = 0;

    loop {
        display(&mut stdout, &lines, &query, selected_line, start_row)?;

        if let Event::Key(key_event) = event::read()? {
            match key_event {
                KeyEvent {
                    code: KeyCode::Char('j'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    selected_line = (selected_line + 1).min(lines.len().saturating_sub(1));
                }
                KeyEvent {
                    code: KeyCode::Char('k'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    if selected_line > 0 {
                        selected_line -= 1;
                    }
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    query.push(c);
                }
                KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    query.pop();
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                } => {
                    if let Some(selected) = lines.get(selected_line) {
                        println!("{}", selected);
                        break;
                    }
                }
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => {
                    break;
                }
                _ => {}
            }
        }
    }

    cleanup_terminal(&mut stdout)?;
    Ok(())
}

fn display(
    stdout: &mut Stdout,
    lines: &[String],
    query: &str,
    selected_line: usize,
    start_row: u16,
) -> Result<(), io::Error> {
    let (cols, _) = size()?;
    let filtered_lines: Vec<&String> = lines
        .iter()
        .filter(|line| fuzzy_match(query, line))
        .collect();

    // Display the query input line
    stdout.execute(MoveTo(0, start_row))?;
    stdout.execute(Print(format!("{:width$}", query, width = cols as usize)))?;

    // Display the filtered list
    for (i, line) in filtered_lines.iter().enumerate() {
        let row = start_row + i as u16 + 2; // Start from two lines below the current cursor position
        stdout.execute(MoveTo(0, row))?;
        if i == selected_line {
            stdout.execute(Print("-> "))?;
        } else {
            stdout.execute(Print("   "))?;
        }
        let display_line = if line.len() > cols as usize - 3 {
            &line[..cols as usize - 3] // Truncate line to fit the screen width
        } else {
            line
        };
        stdout.execute(Print(display_line))?;
    }

    // Place cursor back at the end of the query input line
    stdout.execute(MoveTo(query.len() as u16, start_row))?;

    stdout.flush()?;
    Ok(())
}

fn cleanup_terminal(stdout: &mut Stdout) -> Result<(), io::Error> {
    disable_raw_mode()?;
    stdout.flush()
}

fn fuzzy_match(query: &str, line: &str) -> bool {
    line.to_lowercase().contains(&query.to_lowercase())
}
