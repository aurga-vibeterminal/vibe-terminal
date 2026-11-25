// Kimi Code app addon — detects terminal states for Kimi Code CLI
//
// Detection strategy:
//   - OSC 9 notification messages   -> idle / confirming
//   - OSC 9;4 ConEmu progress       -> busy / idle
//   - detect_output text scanning   -> confirming (secondary)
//
// Exit detection is handled by the pty layer (forked process exit),
// not by this addon.

var CONFIRMATION_MESSAGES = [
    'approval required',
    'needs your approval',
    'needs your answer',
    'needs your attention',
    'needs your permission',
    'do you want to',
    'would you like to',
    'what would you like to',
    'how would you like to',
];

var SPECIAL_KEYWORDS = [
    'sudo', 'password', 'passphrase', 'secret', 'token', 'api key',
    'ssh key', 'private key', 'delete', 'destroy', 'erase', 'wipe',
    'format', 'drop database', 'remove all', 'rm -rf', 'git',
    'terminate', 'shutdown', 'kill process', 'chmod', 'chown',
    'registry', 'credential', 'keychain', 'force',
];

// ── Addon entry points ────────────────────────────────────────────────────

function get_program_name() {
    addon.log('get_program_name() -> kimi');
    return 'kimi';
}

function get_supported_commands() {
    var cmds = [
        { type: 'osc', code: '9' }  // OSC 9 notifications + OSC 9;4 progress
    ];
    addon.log('get_supported_commands() -> ' + JSON.stringify(cmds));
    return cmds;
}

// ── Helpers ────────────────────────────────────────────────────────────────

function bytesToUtf8(bytes) {
    var str = '';
    if (!bytes) return str;
    for (var i = 0; i < bytes.length; i++) {
        str += String.fromCharCode(bytes[i]);
    }
    try {
        return decodeURIComponent(escape(str));
    } catch (e) {
        return str;
    }
}

function containsAny(text, patterns) {
    for (var i = 0; i < patterns.length; i++) {
        if (text.indexOf(patterns[i]) !== -1) return true;
    }
    return false;
}

// ── detect_command: called when a subscribed OSC sequence arrives ─────────

function detect_command(os, type, code, data) {
    //addon.log('detect_command: type=' + type + ' code=' + code);

    if (type !== 'osc' || code !== '9') {
        return { status: 'none' };
    }

    var payload = bytesToUtf8(data);
    //addon.log('detect_command: payload=' + payload);

    if (!payload) {
        return { status: 'none' };
    }

    // ── OSC 9;4 — ConEmu progress indicator ──────────────────────────────
    // Format: "4;{state};{value}". State 0 clears progress; non-zero states
    // mean active progress, including indeterminate progress such as "4;3".
    // Emitted by pi-tui's ProcessTerminal.setProgress().
    if (payload.indexOf('4;') === 0) {
        var parts = payload.split(';');
        var state = parts.length > 1 ? parts[1] : '';
        var value = parts.length > 2 ? parts[2] : '';
        var status = state !== '' && state !== '0' ? 'busy' : 'idle';
        //addon.log('detect_command: progress state=' + state + ' value=' + value + ' -> ' + status);
        return { status: status };
    }

    // ── OSC 9 — Desktop notification messages ─────────────────────────────
    // Kimi Code sends these at state transitions:
    //   "Kimi Code task complete"          -> idle
    //   "Kimi Code approval required: ..." -> confirming
    //   "Kimi Code needs your answer: .."  -> confirming

    if (payload.indexOf('task complete') !== -1) {
        addon.log('detect_command: task complete -> idle');
        return { status: 'idle' };
    }

    if (containsAny(payload.toLowerCase(), CONFIRMATION_MESSAGES)) {
        var isSpecial = containsAny(payload.toLowerCase(), SPECIAL_KEYWORDS);
        if (isSpecial) {
            addon.log('detect_command: special confirmation detected');
            return { status: 'confirming_special', message: payload };
        }
        addon.log('detect_command: confirmation detected');
        return { status: 'confirming', message: payload };
    }

    // Unknown OSC 9 payload — ignore
    return { status: 'none' };
}

// ── detect_output: called with terminal output text (secondary check) ─────

function detect_output(text) {
    //addon.log('detect_output: text length=' + (text ? text.length : 0));

    if (!text) return { status: 'idle', message: '' };

    var combined = text.trim().toLowerCase();
    var lines = combined.split('\n');

    // Walk lines in reverse to find the last matching confirmation
    for (var i = lines.length - 1; i >= 0; i--) {
        var line = lines[i].trim();
        if (!line) continue;

        // Look for confirmation patterns in output
        var matched = false;
        for (var j = 0; j < CONFIRMATION_MESSAGES.length; j++) {
            if (line.indexOf(CONFIRMATION_MESSAGES[j]) !== -1) {
                matched = true;
                break;
            }
        }
        if (!matched) continue;

        var isSpecial = containsAny(combined, SPECIAL_KEYWORDS);
        if (isSpecial) {
            addon.log('detect_output: special confirmation detected');
            return { status: 'confirming_special', message: 'kimi code is waiting for a special confirmation' };
        }
        addon.log('detect_output: confirmation detected');
        return { status: 'confirming', message: 'kimi code is waiting for a confirmation' };
    }

    return { status: 'idle', message: '' };
}
