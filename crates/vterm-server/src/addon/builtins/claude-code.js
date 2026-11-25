// Claude Code app addon — detects terminal states for Claude Code CLI

var IDLE_PREFIX = '\u2733';       // ✳
// Claude Code uses braille pattern characters (U+2800–U+28FF) as spinner frames
var BRAILLE_START = 0x2800;
var BRAILLE_END = 0x28FF;

var CONFIRMATION_PATTERNS = [
    'do you want to',
    'would you like to',
    'what would you like to',
    'how would you like to',
    'exit plan mode?',
    'ready to code?',
    'answer questions?',
    'claude code needs your approval for the plan',
    'claude code wants to enter plan mode',
    'claude code needs your attention',
    'claude needs your approval for a review artifact',
    'claude needs your permission to use',
    'claude code needs your permission to use'
];

var SPECIAL_KEYWORDS = [
    'sudo', 'password', 'passphrase', 'secret', 'token', 'api key',
    'ssh key', 'private key', 'delete', 'destroy', 'erase', 'wipe',
    'format', 'drop database', 'remove all', 'rm -rf', 'git',
    'terminate', 'shutdown', 'kill process'
];

function get_program_name() {
    addon.log('get_program_name() -> claude');
    return 'claude';
}

function get_supported_commands() {
    var cmds = [
        { type: 'osc', code: '0' },
        { type: 'osc', code: '2' }
    ];
    addon.log('get_supported_commands() -> ' + JSON.stringify(cmds));
    return cmds;
}

// Decode an array of byte values to a UTF-8 string.
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

// type: sequence type string (e.g. 'osc')
// code: code string (e.g. '0' for OSC 0)
// data: byte array (e.g. [226,160,130,32,67,108,97,117,100,101,...])
function detect_command(os, type, code, data) {
    //addon.log('detect_command: type=' + type + ' code=' + code + ' data=' + JSON.stringify(data));
    if (type === 'osc' && code === '0') {
        var title = bytesToUtf8(data);
        var trimmed = title.trimStart();
        //addon.log('detect_command: title=' + trimmed);
        if (!trimmed) return { status: 'none' };
        var prefix = trimmed.charAt(0);
        if (prefix === IDLE_PREFIX) {
            //addon.log('detect_command: idle detected');
            return { status: 'idle' };
        }
        var cp = prefix.codePointAt(0);
        if (cp >= BRAILLE_START && cp <= BRAILLE_END) {
            //addon.log('detect_command: busy detected (braille U+' + cp.toString(16).toUpperCase() + ')');
            return { status: 'busy' };
        }
        // title is exactly 'claude' — app name only, not a real title
        if (trimmed === 'claude') {
            //addon.log('detect_command: title is "claude", ignoring');
            return { status: 'none' };
        }
        // Title contains "claude" (case insensitive) — still the app
        if (trimmed.toLowerCase().indexOf('claude') !== -1) {
            //addon.log('detect_command: title contains "claude", ignoring');
            return { status: 'none' };
        }
        // Only treat as "stop" if the title looks like a shell prompt
        // Common shell titles: user@host:path, ~/path, /home/user/..., etc.
        var looksLikeShell = trimmed.match(/^[^ ]+@[^ ]*:/)   // user@host:path
            || trimmed.match(/^~\//)                           // ~/path
            || trimmed.match(/^\/(home|Users|tmp|var|etc)\//)  // Unix absolute paths
            || trimmed.match(/[$#%]\s*$/);                     // ends with prompt char
        if (looksLikeShell) {
            addon.log('detect_command: shell-like title detected, app stopped, title=' + trimmed);
            return { status: 'stop' };
        }
        addon.log('detect_command: unknown title, ignoring');
        return { status: 'none' };
    }
    return { status: 'none' };
}

function detect_output(text) {
    addon.log('detect_output: text length=' + (text ? text.length : 0));
    var combined = text.trim().toLowerCase();
    var lines = combined.split('\n');
    // Walk lines in reverse to find the last matching confirmation pattern
    for (var i = lines.length - 1; i >= 0; i--) {
        var line = lines[i].trim();
        if (!line) continue;
        var matched = false;
        for (var j = 0; j < CONFIRMATION_PATTERNS.length; j++) {
            if (line.startsWith(CONFIRMATION_PATTERNS[j])) {
                matched = true;
                break;
            }
        }
        if (!matched) continue;
        // Classify confirmation kind
        var isSpecial = false;
        for (var k = 0; k < SPECIAL_KEYWORDS.length; k++) {
            if (combined.indexOf(SPECIAL_KEYWORDS[k]) !== -1) {
                isSpecial = true;
                break;
            }
        }
        if (isSpecial) {
            addon.log('detect_output: special confirmation detected');
            return { status: 'confirming_special', message: 'claude code is waiting for a special confirmation' };
        }
        addon.log('detect_output: normal confirmation detected');
        return { status: 'confirming', message: 'claude code is waiting for a normal confirmation' };
    }
    return { status: 'idle', message: '' };
}
