use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
    QueueableCommand,
};
use std::io::{stdout, Write};
use vterm_common::TerminalProfile;

/// Let user select a profile interactively
pub fn select_profile(profiles: &[TerminalProfile]) -> anyhow::Result<String> {
    if profiles.is_empty() {
        return Err(anyhow::anyhow!("No terminal profiles available"));
    }

    // Sort profiles: default first, then by title
    let mut sorted: Vec<_> = profiles.iter().collect();
    sorted.sort_by(|a, b| match (b.is_default, a.is_default) {
        (true, false) => std::cmp::Ordering::Greater,
        (false, true) => std::cmp::Ordering::Less,
        _ => a.title.cmp(&b.title),
    });

    let mut selected = 0usize;
    let start_row = get_start_row(&sorted)?;

    terminal::enable_raw_mode()?;

    loop {
        draw_profiles(&sorted, selected, start_row)?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected < sorted.len() - 1 {
                        selected += 1;
                    }
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    terminal::disable_raw_mode()?;
                    clear_menu(start_row, sorted.len())?;
                    return Ok(sorted[selected].id.clone());
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    terminal::disable_raw_mode()?;
                    clear_menu(start_row, sorted.len())?;
                    return Err(anyhow::anyhow!("Profile selection cancelled"));
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    let digit = c as usize - '0' as usize;
                    if digit > 0 && digit <= sorted.len() {
                        terminal::disable_raw_mode()?;
                        clear_menu(start_row, sorted.len())?;
                        return Ok(sorted[digit - 1].id.clone());
                    }
                }
                _ => {}
            }
        }
    }
}

fn get_start_row(profiles: &[&TerminalProfile]) -> anyhow::Result<u16> {
    let mut stdout = stdout();
    let (cols, rows) = terminal::size()?;

    // Calculate how many rows we need
    let needed = 4 + profiles.len() as u16; // Header + separator + profiles + prompt

    // If we don't have enough space, start from top
    if needed > rows {
        return Ok(0);
    }

    // Print header first in normal mode
    writeln!(stdout, "")?;
    writeln!(
        stdout,
        "  Select a terminal profile (use arrow keys or number):"
    )?;
    writeln!(
        stdout,
        "  {}",
        "─".repeat((cols as usize).saturating_sub(4))
    )?;

    stdout.flush()?;

    // Return row where profiles start
    Ok(terminal::size()?.1 - needed + 3)
}

fn draw_profiles(
    profiles: &[&TerminalProfile],
    selected: usize,
    start_row: u16,
) -> anyhow::Result<()> {
    let mut stdout = stdout();

    for (i, profile) in profiles.iter().enumerate() {
        let row = start_row + i as u16;

        stdout.queue(cursor::MoveTo(0, row))?;
        stdout.queue(terminal::Clear(ClearType::CurrentLine))?;
        stdout.queue(cursor::MoveTo(2, row))?;

        if i == selected {
            stdout.queue(SetForegroundColor(Color::Cyan))?;
            stdout.queue(Print("> "))?;
            stdout.queue(SetForegroundColor(Color::White))?;
        } else {
            stdout.queue(Print("  "))?;
            stdout.queue(ResetColor)?;
        }

        // Number
        stdout.queue(Print(format!("{}.", i + 1)))?;
        stdout.queue(cursor::MoveToColumn(6))?;

        // Profile name (highlighted if selected)
        if i == selected {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(Print(&profile.title))?;
            stdout.queue(ResetColor)?;
        } else {
            stdout.queue(Print(&profile.title))?;
        }

        // Default marker
        if profile.is_default {
            stdout.queue(cursor::MoveToColumn(30))?;
            stdout.queue(SetForegroundColor(Color::DarkGrey))?;
            stdout.queue(Print("(default)"))?;
            stdout.queue(ResetColor)?;
        }
    }

    // Print prompt at bottom
    let prompt_row = start_row + profiles.len() as u16;
    stdout.queue(cursor::MoveTo(0, prompt_row))?;
    stdout.queue(terminal::Clear(ClearType::CurrentLine))?;
    stdout.queue(cursor::MoveTo(2, prompt_row))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("↑/↓ or j/k to move, Enter to select, q to quit"))?;
    stdout.queue(ResetColor)?;

    stdout.flush()?;
    Ok(())
}

fn clear_menu(start_row: u16, profile_count: usize) -> anyhow::Result<()> {
    let mut stdout = stdout();
    let total_rows = profile_count as u16 + 1;

    for i in 0..total_rows {
        stdout.queue(cursor::MoveTo(0, start_row + i))?;
        stdout.queue(terminal::Clear(ClearType::CurrentLine))?;
    }

    stdout.flush()?;
    Ok(())
}

/// Simple non-interactive profile listing
pub fn list_profiles(profiles: &[TerminalProfile]) -> anyhow::Result<()> {
    let mut sorted: Vec<_> = profiles.iter().collect();
    sorted.sort_by(|a, b| match (b.is_default, a.is_default) {
        (true, false) => std::cmp::Ordering::Greater,
        (false, true) => std::cmp::Ordering::Less,
        _ => a.title.cmp(&b.title),
    });

    println!("Available terminal profiles:");
    println!("");

    for (i, profile) in sorted.iter().enumerate() {
        let default_marker = if profile.is_default { " (default)" } else { "" };
        println!("  {}. {}{}", i + 1, profile.title, default_marker);
    }

    println!("");
    println!("Use 'vterm --profile <id>' to select a profile");

    Ok(())
}
