// DeepSeek Harness app addon — detects dsh-tui terminal states.

function get_program_name() {
    return 'dsh';
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
    if (!state) return { status: 'none' };
    return { status: state === '0' ? 'idle' : 'busy' };
}

function detect_output(text) {
    return { status: 'none', message: '' };
}
