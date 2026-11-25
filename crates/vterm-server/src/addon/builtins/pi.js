// Pi Agent app addon.
//
// OSC 9;4 and OSC 7501 report authoritative busy/idle transitions. Pi disables
// OSC 9;4 by default, so its braille working indicator also detects Busy.

var BUSY_SPINNERS = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

function get_program_name() {
    return 'pi';
}

function get_supported_commands() {
    return [
        { type: 'osc', code: '9' },
        { type: 'osc', code: '7501' },
    ];
}

function bytesToUtf8(bytes) {
    var value = '';
    if (!bytes) return value;
    for (var i = 0; i < bytes.length; i++) {
        value += String.fromCharCode(bytes[i]);
    }
    try {
        return decodeURIComponent(escape(value));
    } catch (e) {
        return value;
    }
}

function isBusySpinner(value) {
    for (var i = 0; i < BUSY_SPINNERS.length; i++) {
        if (value === BUSY_SPINNERS[i]) return true;
    }
    return false;
}

function detect_command(os, type, code, data) {
    if (type !== 'osc' || (code !== '9' && code !== '7501')) {
        return { status: 'none' };
    }

    var payload = bytesToUtf8(data);
    if (code === '7501') {
        var pairs = payload.split(':');
        for (var i = 0; i < pairs.length; i++) {
            if (pairs[i].indexOf('state=') === 0) {
                var programState = pairs[i].slice(6);
                if (programState === 'working') return { status: 'busy' };
                if (programState === 'idle' || programState === 'done' ||
                    programState === 'error' || programState === 'clear') {
                    return { status: 'idle' };
                }
                return { status: 'none' };
            }
        }
        return { status: 'none' };
    }

    if (payload.indexOf('4;') !== 0) {
        return { status: 'none' };
    }

    var state = payload.split(';')[1] || '';
    if (state === '0') return { status: 'idle' };
    if (state === '1' || state === '2' || state === '3' || state === '4') {
        return { status: 'busy' };
    }
    return { status: 'none' };
}

// Pi draws its indicator either at the start of a line or inside the editor's
// top border, for example "── ⠋ Working ─────" or "───⠋──" at narrow widths.
function detect_output(text) {
    if (!text) return { status: 'none', message: '' };

    var lines = text.split(/[\r\n]/);
    for (var i = 0; i < lines.length; i++) {
        var line = lines[i].trim();
        var start = 0;
        while (start < line.length && line.charAt(start) === '─') start++;
        while (start < line.length && /\s/.test(line.charAt(start))) start++;
        if (!isBusySpinner(line.charAt(start))) continue;
        var next = line.charAt(start + 1);
        if (!next || next === '─' || /\s/.test(next)) {
            return { status: 'busy', message: '', idle_after_ms: 500 };
        }
    }
    return { status: 'none', message: '' };
}
