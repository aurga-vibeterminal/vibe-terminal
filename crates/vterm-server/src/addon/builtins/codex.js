// Codex app addon - detects terminal states for the Codex CLI

var BUSY_SPINNERS = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

var CONFIRMATION_PATTERNS = [
    'do you want to',
    'would you like to',
    'what would you like to',
    'how would you like to',
    'codex needs your approval',
    'codex needs your attention',
    'codex needs your permission',
    'codex wants to',
    'allow codex to',
    'allow command',
    'allow this command',
    'approve command',
    'run this command?',
    'apply this patch?',
    'proceed?',
    'continue?',
    'allow once',
    'yes, allow',
    'yes, and don',
    'no, and tell codex'
];

var SPECIAL_KEYWORDS = [
    'sudo', 'password', 'passphrase', 'secret', 'token', 'api key',
    'ssh key', 'private key', 'delete', 'destroy', 'erase', 'wipe',
    'format', 'drop database', 'remove all', 'rm -rf', 'git',
    'terminate', 'shutdown', 'kill process', 'chmod', 'chown',
    'registry', 'credential', 'keychain'
];

function get_program_name() {
    addon.log('get_program_name() -> codex');
    return 'codex';
}

function get_supported_commands() {
    var cmds = [
        { type: 'osc', code: '0' },
        { type: 'osc', code: '2' }
    ];
    addon.log('get_supported_commands() -> ' + JSON.stringify(cmds));
    return cmds;
}

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

function startsWithAny(line, patterns) {
    for (var i = 0; i < patterns.length; i++) {
        if (line.startsWith(patterns[i])) return true;
    }
    return false;
}

function containsAny(text, patterns) {
    for (var i = 0; i < patterns.length; i++) {
        if (text.indexOf(patterns[i]) !== -1) return true;
    }
    return false;
}

function equalsAny(value, patterns) {
    for (var i = 0; i < patterns.length; i++) {
        if (value === patterns[i]) return true;
    }
    return false;
}

function detect_command(os, type, code, data) {
    //addon.log('detect_command: os=' + os + ' type=' + type + ' code=' + code + ' data=' + JSON.stringify(data));
    if (type !== 'osc' || (code !== '0' && code !== '2')) {
        return { status: 'none' };
    }

    var title = bytesToUtf8(data);
    var trimmed = title.trimStart();
    //addon.log('detect_command: title=' + trimmed);
    if (!trimmed) {
        addon.log('detect_command: empty title, app stopped');
        return { status: 'stop' };
    }

    var prefix = trimmed.charAt(0);
    if (equalsAny(prefix, BUSY_SPINNERS)) {
        //addon.log('detect_command: busy detected (spinner ' + prefix + ')');
        return { status: 'busy' };
    }

    //addon.log('detect_command: non-empty title, idle detected');
    return { status: 'idle' };
}

function detect_output(text) {
    addon.log('detect_output: text length=' + (text ? text.length : 0));
    if (!text) return { status: 'idle', message: '' };

    var combined = text.trim().toLowerCase();
    var lines = combined.split('\n');
    for (var i = lines.length - 1; i >= 0; i--) {
        var line = lines[i].trim();
        if (!line) continue;
        if (
            !startsWithAny(line, CONFIRMATION_PATTERNS) &&
            !(line.indexOf('codex') !== -1 && containsAny(line, ['approval', 'permission', 'confirm', 'attention']))
        ) {
            continue;
        }

        if (containsAny(combined, SPECIAL_KEYWORDS)) {
            addon.log('detect_output: special confirmation detected');
            return { status: 'confirming_special', message: 'codex is waiting for a special confirmation' };
        }
        addon.log('detect_output: normal confirmation detected');
        return { status: 'confirming', message: 'codex is waiting for a normal confirmation' };
    }
    return { status: 'idle', message: '' };
}
