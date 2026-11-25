// Qwen Code app addon.
//
// Qwen emits OSC 9;4;3 while a tool is executing and OSC 9;4;0 when idle.
// Model-only response streaming is detected through its braille spinner.

var BUSY_SPINNERS = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

function get_program_name() {
    return 'qwen';
}

function get_supported_commands() {
    return [{ type: 'osc', code: '9' }];
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

function detect_command(os, type, code, data) {
    if (type !== 'osc' || code !== '9') {
        return { status: 'none' };
    }

    var payload = bytesToUtf8(data);
    if (payload.indexOf('4;') !== 0) {
        return { status: 'none' };
    }

    var state = payload.split(';')[1] || '';
    if (state === '0') {
        return { status: 'idle' };
    }
    if (state === '1' || state === '2' || state === '3' || state === '4') {
        return { status: 'busy' };
    }
    return { status: 'none' };
}

// Spinner output supplements OSC 9;4 for model-only responses. Qwen may draw
// the input box and footer below the spinner in the same output frame.
function detect_output(text) {
    if (!text) return { status: 'none', message: '' };

    var lines = text.split(/[\r\n]/);
    for (var i = lines.length - 1; i >= 0; i--) {
        var line = lines[i].trim();
        if (!line) continue;
        var prefix = line.charAt(0);
        for (var j = 0; j < BUSY_SPINNERS.length; j++) {
            if (prefix === BUSY_SPINNERS[j]) {
                return { status: 'busy', message: '', idle_after_ms: 500 };
            }
        }
    }
    return { status: 'none', message: '' };
}
