//! Generic full-screen interactive picker used by `vtermc -s`.

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
    QueueableCommand,
};
use std::io::{stdout, Write};

/// Outcome of [`select_index_deletable`]: an item was picked, or an item
/// was confirmed for deletion (the `d` → confirm flow already ran inside
/// the picker).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    Pick(usize),
    Delete(usize),
}

/// Clear the screen and let the user pick one of `items` with the arrow keys.
/// Returns the selected index.
pub fn select_index(title: &str, items: &[String]) -> anyhow::Result<usize> {
    match select_impl(title, items, None)? {
        Selection::Pick(index) => Ok(index),
        // Unreachable: without a `deletable` slice `d` is never handled.
        Selection::Delete(_) => Err(anyhow::anyhow!("Selection cancelled")),
    }
}

/// Like [`select_index`], but rows where `deletable[i]` is true can be
/// flagged for deletion: `d` asks for confirmation on the hint line
/// (`y`/Enter confirms, any other key aborts) and [`Selection::Delete`] is
/// returned once confirmed. `deletable` must have the same length as
/// `items`.
pub fn select_index_deletable(
    title: &str,
    items: &[String],
    deletable: &[bool],
) -> anyhow::Result<Selection> {
    if deletable.len() != items.len() {
        return Err(anyhow::anyhow!("deletable flags must match the item count"));
    }
    select_impl(title, items, Some(deletable))
}

fn select_impl(
    title: &str,
    items: &[String],
    deletable: Option<&[bool]>,
) -> anyhow::Result<Selection> {
    if items.is_empty() {
        return Err(anyhow::anyhow!("Nothing to select"));
    }

    let (cols, _) = terminal::size()?;
    let separator = "─".repeat((cols as usize).saturating_sub(4).max(4));
    let plain_hint = if deletable.is_some() {
        "↑/↓ or j/k to move, Enter to select, d to delete, q to cancel"
    } else {
        "↑/↓ or j/k to move, Enter to select, q to cancel"
    };
    let confirm_hint = "Delete this server? y to confirm, any other key to cancel";

    let mut selected = 0usize;
    let mut confirm = false;
    let mut stdout = stdout();

    terminal::enable_raw_mode()?;
    let result = (|| -> anyhow::Result<Selection> {
        loop {
            draw(
                title,
                &separator,
                items,
                selected,
                if confirm { confirm_hint } else { plain_hint },
                confirm,
            )?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if confirm {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                            return Ok(Selection::Delete(selected));
                        }
                        // Any other key aborts the delete confirmation;
                        // the picker keeps running.
                        _ => confirm = false,
                    }
                    continue;
                }
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        selected = if selected == 0 {
                            items.len() - 1
                        } else {
                            selected - 1
                        };
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        selected = (selected + 1) % items.len();
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        return Ok(Selection::Pick(selected));
                    }
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                        return Err(anyhow::anyhow!("Selection cancelled"));
                    }
                    KeyCode::Char('d') | KeyCode::Char('D')
                        if deletable
                            .is_some_and(|flags| flags.get(selected).copied().unwrap_or(false)) =>
                    {
                        confirm = true;
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        let digit = c as usize - '0' as usize;
                        if digit > 0 && digit <= items.len() {
                            return Ok(Selection::Pick(digit - 1));
                        }
                    }
                    _ => {}
                }
            }
        }
    })();

    terminal::disable_raw_mode()?;
    // Clear the menu so the picked action starts on a fresh screen.
    let _ = stdout.queue(terminal::Clear(ClearType::All));
    let _ = stdout.queue(cursor::MoveTo(0, 0));
    let _ = stdout.flush();
    result
}

fn draw(
    title: &str,
    separator: &str,
    items: &[String],
    selected: usize,
    hint: &str,
    warn: bool,
) -> anyhow::Result<()> {
    let mut stdout = stdout();
    let rows = terminal::size()?.1 as usize;

    stdout.queue(cursor::MoveTo(0, 0))?;
    stdout.queue(terminal::Clear(ClearType::All))?;

    write_line(&mut stdout, "")?;
    write_line(&mut stdout, &format!("  {title}"))?;
    write_line(&mut stdout, &format!("  {separator}"))?;

    let header_rows = 3usize;
    // Leave one row for the key hint at the bottom.
    let visible = (rows as usize).saturating_sub(header_rows + 2).max(1);
    let scroll_top = selected.saturating_sub(visible - 1);
    let scroll_top = scroll_top.min(items.len().saturating_sub(visible).max(0));

    for (i, item) in items.iter().enumerate().skip(scroll_top).take(visible) {
        stdout.queue(cursor::MoveTo(0, (header_rows + (i - scroll_top)) as u16))?;
        stdout.queue(terminal::Clear(ClearType::CurrentLine))?;
        stdout.queue(cursor::MoveTo(2, (header_rows + (i - scroll_top)) as u16))?;
        if i == selected {
            stdout.queue(SetForegroundColor(if warn {
                Color::Red
            } else {
                Color::Cyan
            }))?;
            stdout.queue(Print("> "))?;
            stdout.queue(SetForegroundColor(Color::White))?;
        } else {
            stdout.queue(Print("  "))?;
        }
        stdout.queue(Print(format!("{}.", i + 1)))?;
        stdout.queue(cursor::MoveToColumn(6))?;
        if i == selected {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(Print(truncate(item, cols_available())))?;
            stdout.queue(ResetColor)?;
        } else {
            stdout.queue(Print(truncate(item, cols_available())))?;
            stdout.queue(ResetColor)?;
        }
    }

    // Key hint on the last row.
    let hint_row = rows.saturating_sub(1) as u16;
    stdout.queue(cursor::MoveTo(0, hint_row))?;
    stdout.queue(terminal::Clear(ClearType::CurrentLine))?;
    stdout.queue(cursor::MoveTo(2, hint_row))?;
    stdout.queue(SetForegroundColor(if warn {
        Color::Yellow
    } else {
        Color::DarkGrey
    }))?;
    stdout.queue(Print(truncate(hint, hint_cols_available())))?;
    stdout.queue(ResetColor)?;

    stdout.flush()?;
    Ok(())
}

fn cols_available() -> usize {
    terminal::size()
        .map(|(cols, _)| cols as usize)
        .unwrap_or(80)
        .saturating_sub(8)
        .max(10)
}

fn hint_cols_available() -> usize {
    terminal::size()
        .map(|(cols, _)| cols as usize)
        .unwrap_or(80)
        .saturating_sub(4)
        .max(10)
}

fn truncate(text: &str, max_cols: usize) -> String {
    if text.chars().count() <= max_cols {
        text.to_string()
    } else {
        let cut: String = text.chars().take(max_cols.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}

fn write_line(stdout: &mut std::io::Stdout, line: &str) -> anyhow::Result<()> {
    stdout.queue(Print(line))?;
    stdout.queue(Print("\r\n"))?;
    Ok(())
}
