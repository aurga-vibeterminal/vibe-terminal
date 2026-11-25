// OpenCode app addon.
//
// OpenCode does not emit terminal activity OSC. Its classic TUI uses braille
// spinners; the Mini footer uses a one-cell spinner beside its running status.

var BRAILLE_SPINNERS = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
var MINI_SPINNERS = ['▪', '▫', '□', '■', '▘', '▝', '▗', '▖', '▧', '▩', '▨', '░', '▒', '▓', '█', '*', '-', '\\', '|', '/'];

function get_program_name() {
    return 'opencode';
}

function get_supported_commands() {
    return [];
}

function detect_command(os, type, code, data) {
    return { status: 'none' };
}

function busyResult(refreshOnly) {
    return {
        status: 'busy',
        message: '',
        idle_after_ms: 1000,
        refresh_only: refreshOnly === true,
    };
}

function isMiniSpinner(symbol) {
    if (MINI_SPINNERS.indexOf(symbol) !== -1) return true;
    if (!symbol) return false;
    var point = symbol.codePointAt(0);
    return (point >= 0x1cd00 && point <= 0x1ceaf) ||
        (point >= 0x1fb00 && point <= 0x1fbff);
}

function isMiniBusyStatus(rest) {
    var status = rest.trim();
    return /^Running(?:\s|$)/.test(status) ||
        /^reconnecting(?:\s|$)/.test(status) ||
        /^sending prompt(?:\s|$)/.test(status) ||
        /^\S+\s+stop(?:\s|$)/.test(status);
}

function detect_output(text) {
    if (!text) return { status: 'none', message: '' };

    var lines = text.split(/[\r\n]/);
    for (var i = 0; i < lines.length; i++) {
        var line = lines[i].trim();
        if (!line) continue;
        var symbol = Array.from(line)[0] || '';
        var rest = line.slice(symbol.length);
        if (BRAILLE_SPINNERS.indexOf(symbol) !== -1 && (!rest || /^\s/.test(rest))) {
            return busyResult(false);
        }
        if (isMiniSpinner(symbol) && (!rest.trim() || isMiniBusyStatus(rest))) {
            return busyResult(!rest.trim());
        }
    }
    return { status: 'none', message: '' };
}
