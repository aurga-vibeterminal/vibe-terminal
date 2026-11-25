use vterm_common::SessionAppStatus;
use vterm_core::emulator::Terminal;

pub const RECENT_OUTPUT_LIMIT: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptMatch {
    pub app_status: SessionAppStatus,
    pub summary: String,
    pub matched_line: String,
}

pub fn screen_text(terminal: &Terminal) -> String {
    terminal.grid_to_string()
}

pub fn strip_ansi_sequences(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if matches!(chars.peek(), Some('[')) {
                chars.next();
                while let Some(next) = chars.next() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            } else if matches!(chars.peek(), Some(']')) {
                chars.next();
                while let Some(next) = chars.next() {
                    if next == '\u{7}' {
                        break;
                    }
                    if next == '\u{1b}' && matches!(chars.peek(), Some('\\')) {
                        chars.next();
                        break;
                    }
                }
            }
            continue;
        }

        output.push(ch);
    }

    output
}

pub fn trim_to_recent_limit(buffer: &mut String, limit: usize) {
    if buffer.len() <= limit {
        return;
    }

    let mut start = buffer.len() - limit;
    while start < buffer.len() && !buffer.is_char_boundary(start) {
        start += 1;
    }

    buffer.drain(..start);
}

pub fn prompt_signature(prompt: &PromptMatch) -> String {
    format!("{:?}:{}", prompt.app_status, prompt.matched_line.trim())
}

pub fn should_auto_confirm(enabled: bool, prompt: &PromptMatch) -> bool {
    enabled && prompt.app_status == SessionAppStatus::ConfirmingNormal
}

#[cfg(test)]
mod tests {
    use super::*;
    use vterm_common::TerminalSize;

    #[test]
    fn trims_recent_output_without_breaking_utf8_boundaries() {
        let mut buffer = "界".repeat(RECENT_OUTPUT_LIMIT);
        buffer.push('界');

        trim_to_recent_limit(&mut buffer, RECENT_OUTPUT_LIMIT);

        assert!(buffer.is_char_boundary(buffer.len()));
        assert!(buffer.len() <= RECENT_OUTPUT_LIMIT);
        assert!(!buffer.is_empty());
    }
}
