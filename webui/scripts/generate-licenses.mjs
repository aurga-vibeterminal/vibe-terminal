#!/usr/bin/env node
/**
 * Third-party license manifest generator.
 *
 * Collects the license of every third-party component shipped in the
 * release binaries and writes the result into webui/public, where the
 * Vite build copies it into webui/dist and rust-embed then compiles it
 * into vterm-server.exe (see crates/vterm-server/src/config/wwwroot.rs):
 *
 *   public/licenses.json              index consumed by the web UI's
 *                                     Help > Open Source Licenses page
 *   public/licenses/<hash>.txt        full license texts, content-addressed
 *                                     (identical texts stored once)
 *   public/third-party-notices.txt    aggregated plain-text notices; copied
 *                                     next to the release binaries by
 *                                     build.bat / packaging/windows and also
 *                                     served at /third-party-notices.txt
 *
 * Sources:
 *   - Rust crates: `cargo metadata --locked` filtered to packages with a
 *     `source` (registry / git). Workspace members and path dependencies
 *     (incl. the git submodules) are this product itself and are skipped.
 *     Full texts come from the local cargo registry checkout next to each
 *     crate's Cargo.toml (manifest_path), so no network is needed.
 *   - npm packages: webui/package-lock.json, skipping dev-only entries and
 *     optional entries that are not installed (not shipped in the bundle).
 *     Full texts come from node_modules/<pkg>/LICENSE*.
 *   - Manual entries: webui/scripts/manual-licenses.json for components
 *     brought in outside the package managers (e.g. ConPTY).
 *
 * Wired into `npm run build`; can be run standalone for `npm run dev`:
 *
 *   npm run generate:licenses
 *
 * Failures degrade to a `warnings` entry in the manifest (surfaced by the
 * UI) instead of aborting the build, with two exceptions: a missing
 * package-lock.json, and a breach of the permissive-only license policy
 * below (set ALLOW_NONPERMISSIVE=1 to downgrade that to a warning while a
 * replacement dependency is being found).
 *
 * License policy — permissive-only:
 * All Vibe Terminal distributions share one dependency tree, so every
 * component must be usable under a permissive license. SPDX expressions
 * are parsed into OR
 * alternatives: strong copyleft (GPL/AGPL/LGPL/SSPL/...) is tolerated only
 * as a *dropped* alternative of a dual license — the permissive branch is
 * recorded as the applied license, only its files ship, and the copyleft
 * text is omitted from the notices. MPL-2.0 is accepted as weak,
 * file-level copyleft and flagged (modifications to those files must be
 * published under MPL-2.0). A component with no permissive alternative —
 * or an unrecognised expression — fails the build. Packages that declare
 * a known permissive license but ship no text get a canonical text filled
 * in (MIT/ISC/BSD synthesized; Apache-2.0 reused verbatim from a crate).
 */
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const WEBUI_DIR = dirname(dirname(fileURLToPath(import.meta.url)));
const REPO_ROOT = dirname(WEBUI_DIR);
const PUBLIC_DIR = join(WEBUI_DIR, 'public');
const TEXT_DIR = join(PUBLIC_DIR, 'licenses');

/** File names treated as license texts, ranked so plain LICENSE sorts first. */
const LICENSE_FILE_RE = /^(licen[cs]e|unlicen[cs]e|copying|copyright|notice)([._\- ].*)?$/i;

function licenseRank(name) {
  const n = name.toLowerCase();
  if (n.startsWith('license') || n.startsWith('licence')) return 0;
  if (n.startsWith('copying')) return 1;
  if (n.startsWith('notice')) return 2;
  return 3;
}

/** Read a file as UTF-8, falling back to latin1 for odd encodings. */
function readText(file) {
  const buf = readFileSync(file);
  const utf8 = buf.toString('utf8');
  return utf8.includes('\uFFFD') ? buf.toString('latin1') : utf8;
}

/**
 * License-ish files of a package directory as ordered { file, text } parts
 * (plain LICENSE sorts first), so the license policy below can filter the
 * files of dual-licensed components. Dual-licensed crates (LICENSE-MIT +
 * LICENSE-APACHE) keep every file until filtered.
 */
function collectLicenseParts(dir) {
  if (!existsSync(dir)) return [];
  let files;
  try {
    files = readdirSync(dir, { withFileTypes: true })
      .filter((e) => e.isFile() && LICENSE_FILE_RE.test(e.name))
      .sort(
        (a, b) => licenseRank(a.name) - licenseRank(b.name) || a.name.localeCompare(b.name),
      )
      .map((e) => e.name);
  } catch {
    return [];
  }
  const parts = [];
  for (const name of files) {
    try {
      const text = readText(join(dir, name)).trim();
      if (text) parts.push({ file: name, text });
    } catch {
      /* unreadable file: skip */
    }
  }
  return parts;
}

/** Concatenate parts back into one text, naming each file when > 1. */
function joinParts(parts) {
  if (!parts || parts.length === 0) return null;
  return parts
    .map((p) => (parts.length > 1 ? `----- ${p.file} -----\n\n${p.text}` : p.text))
    .join('\n\n\n');
}

/** Kept for callers that only sniff the combined text. */
function collectLicenseText(dir) {
  return joinParts(collectLicenseParts(dir));
}

/** Best-effort SPDX guess from the license text itself. */
function sniffLicense(text) {
  if (!text) return null;
  const head = text.slice(0, 4000);
  if (/Apache License\s+Version\s+2\.0/i.test(head)) return 'Apache-2.0';
  if (/GNU Affero General Public License/i.test(head)) return 'AGPL-3.0-or-later';
  if (/GNU Lesser General Public License/i.test(head)) return 'LGPL-2.1-or-later';
  if (/GNU General Public License/i.test(head)) return 'GPL-2.0-or-later';
  if (/Mozilla Public License\s+2\.0/i.test(head)) return 'MPL-2.0';
  // Boost before MIT: the BSL preamble also says "Permission is hereby
  // granted, free of charge".
  if (/Boost Software License/i.test(head)) return 'BSL-1.0';
  if (/Permission is hereby granted, free of charge/i.test(head)) return 'MIT';
  if (/ISC License/i.test(head)) return 'ISC';
  if (/BSD 3-Clause License/i.test(head)) return 'BSD-3-Clause';
  if (
    /Redistribution and use in source and binary forms/i.test(head) &&
    /may not be used to endorse or promote/i.test(head)
  ) {
    return 'BSD-3-Clause';
  }
  if (/Redistribution and use in source and binary forms/i.test(head)) return 'BSD-2-Clause';
  if (/unlicense\.org/i.test(head)) return 'Unlicense';
  if (/zlib\.(net|org)/i.test(head)) return 'Zlib';
  if (/Unicode License v3/i.test(head)) return 'Unicode-3.0';
  if (/Creative Commons Zero|CC0\s+1\.0/i.test(head)) return 'CC0-1.0';
  return null;
}

// ---------------------------------------------------------------------------
// License policy — permissive-only
// ---------------------------------------------------------------------------
// Terms (normalised SPDX ids) this product may rely on.
const PERMISSIVE_TERMS = new Set([
  'mit',
  'mit-0',
  'apache-2.0',
  'apache-2.0 with llvm-exception',
  'bsd-2-clause',
  'bsd-3-clause',
  'bsd-1-clause',
  'isc',
  'zlib',
  '0bsd',
  'unlicense',
  'cc0-1.0',
  'bsl-1.0', // Boost Software License (permissive; NOT BUSL)
  'unicode-3.0',
  'unicode-dfs-2016',
  'cdla-permissive-2.0',
  'upl-1.0',
  'blueoak-1.0.0',
  'ofl-1.1', // SIL Open Font License: permissive, for fonts bundled with the Web UI
]);
/** Weak, file-level copyleft: allowed, but modifications must be shared. */
const WEAK_COPYLEFT_TERMS = new Set(['mpl-2.0']);
/** Strong copyleft / source-available / non-commercial identifiers. */
const COPYLEFT_RE =
  /(^|[^a-z0-9])(agpl|gpl|lgpl|sspl|gfdl|osl|eupl|sleepycat|qpl|apsl|ibm-pl|cddl|cpl|mpl-1|busl|polyform|nc|nd)([^a-z0-9]|$)/;

function normalizeTerm(term) {
  return term.toLowerCase().replace(/\s+/g, ' ').trim();
}

/** Canonical SPDX casing for display after normalisation. */
const DISPLAY_NAMES = {
  'mit': 'MIT',
  'mit-0': 'MIT-0',
  'apache-2.0': 'Apache-2.0',
  'apache-2.0 with llvm-exception': 'Apache-2.0 WITH LLVM-exception',
  'bsd-2-clause': 'BSD-2-Clause',
  'bsd-3-clause': 'BSD-3-Clause',
  'bsd-1-clause': 'BSD-1-Clause',
  'isc': 'ISC',
  'zlib': 'Zlib',
  '0bsd': '0BSD',
  'unlicense': 'Unlicense',
  'cc0-1.0': 'CC0-1.0',
  'bsl-1.0': 'BSL-1.0',
  'unicode-3.0': 'Unicode-3.0',
  'unicode-dfs-2016': 'Unicode-DFS-2016',
  'cdla-permissive-2.0': 'CDLA-Permissive-2.0',
  'upl-1.0': 'UPL-1.0',
  'blueoak-1.0.0': 'BlueOak-1.0.0',
  'ofl-1.1': 'OFL-1.1',
  'mpl-2.0': 'MPL-2.0',
};

const displayTerm = (term) => DISPLAY_NAMES[term] ?? term;

/**
 * Split an SPDX-ish expression into OR-alternatives of AND-terms. The sloppy
 * `MIT/Apache-2.0` slash form and parentheses are normalised first; the
 * parsing is conservative in the direction this policy needs: a GPL term
 * sitting in an AND chain keeps its alternative copyleft-classified.
 */
function parseLicenseExpression(license) {
  if (!license || typeof license !== 'string') return [];
  const flat = license.replace(/[()]/g, ' ');
  return flat
    .split(/\s+or\s+|\/|\|\|/i)
    .map((alt) =>
      alt
        .split(/\s+and\s+/i)
        .map((t) => normalizeTerm(t))
        .filter(Boolean),
    )
    .filter((alt) => alt.length > 0);
}

function classifyTerm(term) {
  if (PERMISSIVE_TERMS.has(term)) return 'permissive';
  if (WEAK_COPYLEFT_TERMS.has(term)) return 'weak';
  if (COPYLEFT_RE.test(term)) return 'copyleft';
  return 'unknown';
}

/** Collected while generating; reported in the manifest, notices and console. */
const policy = {
  violations: [], // hard errors -> non-zero exit unless ALLOW_NONPERMISSIVE=1
  reviews: [], // unknown expressions -> also fail until clarified
  selections: [], // dual licenses applied via their permissive branch
  weak: [], // MPL-2.0 components: share modifications to these files
  synthesized: [], // canonical texts filled in for missing license files
};

/** Pool of verbatim canonical texts seen while collecting (lowercased spdx -> text). */
const canonicalTextPool = new Map();

function poolKey(spdx) {
  return spdx.toLowerCase();
}

/** Synthesized canonical texts for licenses with a copyright holder slot. */
const HOLDER_TEMPLATES = {
  mit: (holder) => `Copyright (c) ${holder}

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.`,
  isc: (holder) => `Copyright (c) ${holder}

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.`,
  'bsd-2-clause': (holder) => `Copyright (c) ${holder}

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.`,
  'bsd-3-clause': (holder) => `Copyright (c) ${holder}

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its contributors
   may be used to endorse or promote products derived from this software
   without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.`,
};

/** A canonical text for `term`, or null when none is available. */
function canonicalLicenseText(term, holder) {
  if (HOLDER_TEMPLATES[term]) {
    return HOLDER_TEMPLATES[term](holder || 'The package authors');
  }
  if (term === 'apache-2.0' && canonicalTextPool.has(poolKey('Apache-2.0'))) {
    return canonicalTextPool.get(poolKey('Apache-2.0')); // byte-identical to what crates ship
  }
  // Other pooled texts (BSL-1.0, Zlib, CC0-1.0, ...) serve packages that
  // declare the same license but bundle no file.
  if (canonicalTextPool.has(poolKey(term))) {
    return canonicalTextPool.get(poolKey(term));
  }
  return null;
}

/**
 * Apply the permissive-only policy to one component. `parts` are the bundled
 * license files (possibly empty); `holder` names the copyright holder used
 * when synthesizing a canonical text; `fallbackText` carries a text obtained
 * outside the package directory (manual entries), which is shipped as-is and
 * never synthesized over. Returns the fields to merge into the manifest
 * entry — `text` is what gets stored and shipped.
 */
function applyLicensePolicy(entry, parts, holder, fallbackText = null) {
  const original = entry.license;
  const alts = parseLicenseExpression(original);
  const verdicts = alts.map((alt) => {
    const classes = alt.map(classifyTerm);
    if (classes.includes('copyleft')) return 'copyleft';
    if (classes.includes('unknown')) return 'unknown';
    return classes.includes('weak') ? 'weak' : 'permissive';
  });

  if (alts.length === 0) {
    policy.reviews.push(`${entry.name} ${entry.version}: no usable license expression`);
    return { text: joinParts(parts) };
  }
  const keep = alts.filter((_, i) => verdicts[i] === 'permissive' || verdicts[i] === 'weak');
  const dropped = alts.filter((_, i) => verdicts[i] !== 'permissive' && verdicts[i] !== 'weak');

  if (keep.length === 0) {
    const id = `${entry.name} ${entry.version} (${entry.origin})`;
    if (verdicts.includes('copyleft')) {
      policy.violations.push(`${id}: "${original}" offers no permissive alternative`);
    } else {
      policy.reviews.push(`${id}: unrecognised license "${original}"`);
    }
    return { text: joinParts(parts) };
  }

  const chosenExpr = keep.map((a) => a.map(displayTerm).join(' AND ')).join(' OR ');
  const out = { text: fallbackText ?? joinParts(parts) };
  if (chosenExpr.toUpperCase() !== original.toUpperCase()) {
    out.licenseOriginal = original;
    out.license = chosenExpr;
    out.licenseNote = `Applied under its permissive alternative "${chosenExpr}" of "${original}"; the other alternative is not relied upon.`;
  }

  // Omit the license files of the dropped copyleft alternative (e.g. the
  // GPL text of a dual-licensed crate), matched by file name or by content
  // — some crates hide another copy under a generic name like LICENSE.txt.
  // Files that match no alternative (pointers, third-party notices) stay.
  if (dropped.length > 0 && parts.length > 1) {
    const droppedGplFamily = dropped.flat().some((t) => /gpl|sspl/.test(t));
    const dropRes = [];
    for (const t of dropped.flat()) {
      if (/lgpl/.test(t)) dropRes.push(/lgpl/i);
      else if (/agpl/.test(t)) dropRes.push(/agpl/i);
      else if (/^gpl/.test(t)) dropRes.push(/gpl/i);
      else if (/sspl/.test(t)) dropRes.push(/sspl/i);
    }
    const isDroppedFile = (p) => {
      if (dropRes.some((re) => re.test(p.file))) return true;
      if (droppedGplFamily) {
        const sniffed = sniffLicense(p.text);
        if (sniffed && /(GPL|LGPL|AGPL)/.test(sniffed)) return true;
      }
      return false;
    };
    const omitted = parts.filter(isDroppedFile).map((p) => p.file);
    if (omitted.length > 0) {
      const kept = parts.filter((p) => !isDroppedFile(p));
      policy.selections.push(
        `${entry.name} ${entry.version}: applying ${chosenExpr} ` +
          `(alternative ${dropped.map((a) => a.map(displayTerm).join(' AND ')).join(' OR ')} dropped; ` +
          `${omitted.join(', ')} omitted)`,
      );
      out.licenseNote = [
        out.licenseNote,
        `License file(s) ${omitted.join(', ')} of the dropped alternative are not shipped.`,
      ]
        .filter(Boolean)
        .join(' ');
      out.text = joinParts(kept);
    } else {
      policy.selections.push(
        `${entry.name} ${entry.version}: applying ${chosenExpr} ` +
          `(alternative ${dropped.map((a) => a.map(displayTerm).join(' AND ')).join(' OR ')} dropped)`,
      );
    }
  } else if (dropped.length > 0) {
    policy.selections.push(
      `${entry.name} ${entry.version}: applying ${chosenExpr} ` +
        `(alternative ${dropped.map((a) => a.map(displayTerm).join(' AND ')).join(' OR ')} dropped)`,
    );
  }

  if (keep.flat().some((t) => classifyTerm(t) === 'weak')) {
    policy.weak.push(
      `${entry.name} ${entry.version} (${original}): MPL-2.0 file-level copyleft — ` +
        `modifications to these files must be published under MPL-2.0`,
    );
  }

  // Fill in a canonical text when the package bundles none. Try every term
  // of the applied alternatives: a package licensed "Zlib OR Apache-2.0 OR
  // MIT" without files may ship any of those texts to be compliant.
  if (!out.text && !fallbackText) {
    const candidates = [...new Set(keep.flat())];
    let canonical = null;
    let usedTerm = null;
    for (const term of candidates) {
      canonical = canonicalLicenseText(term, holder ?? `The ${entry.name} Authors`);
      if (canonical) {
        usedTerm = term;
        break;
      }
    }
    if (canonical) {
      const label = HOLDER_TEMPLATES[usedTerm]
        ? usedTerm.toUpperCase()
        : /apache/.test(usedTerm)
          ? 'Apache-2.0'
          : usedTerm;
      out.text = `${canonical}\n\n(The upstream package ships no license file; this canonical ${label} text was supplied by the packager.)`;
      out.licenseNote = [
        out.licenseNote,
        'No license file ships with this package; the canonical text is supplied by the packager.',
      ]
        .filter(Boolean)
        .join(' ');
      policy.synthesized.push(`${entry.name} ${entry.version} (${usedTerm})`);
    }
  }
  return out;
}

/** Content-addressed store: identical license texts are written once. */
const textStore = new Map(); // sha1 -> { fileName, text }
const textByFile = new Map(); // 'licenses/<hash>.txt' -> text
function storeText(text) {
  if (!text) return null;
  const hash = createHash('sha1').update(text, 'utf8').digest('hex').slice(0, 12);
  const fileName = `${hash}.txt`;
  textStore.set(hash, { fileName, text });
  const path = `licenses/${fileName}`;
  textByFile.set(path, text);
  return path;
}

/** Extract an https URL from a package.json `repository` field. */
function repoUrl(pkgJson) {
  const repo = pkgJson.repository;
  if (!repo) return null;
  const url = typeof repo === 'string' ? repo : repo.url;
  if (typeof url !== 'string') return null;
  const https = url.replace(/^git\+/, '').replace(/^git:/, 'https:').replace(/\.git$/, '');
  return /^https:\/\//.test(https) ? https : null;
}

// ---------------------------------------------------------------------------
// Rust crates
// ---------------------------------------------------------------------------
function collectRust(warnings) {
  const run = (args) =>
    spawnSync('cargo', ['metadata', '--format-version', '1', ...args], {
      cwd: REPO_ROOT,
      encoding: 'utf8',
      maxBuffer: 64 * 1024 * 1024,
    });
  // --locked first so the manifest matches Cargo.lock exactly; fall back to
  // unlocked (with a warning) when the lock file is mid-update.
  let res = run(['--locked']);
  if (res.error || res.status !== 0) {
    warnings.push(
      `cargo metadata --locked failed (${res.error?.message ?? `exit ${res.status}`}); retrying unlocked`,
    );
    res = run([]);
  }
  if (res.error || res.status !== 0) {
    warnings.push(
      `cargo metadata unavailable (${res.error?.message ?? `exit ${res.status}`}); Rust crates omitted`,
    );
    return [];
  }
  const metadata = JSON.parse(res.stdout);
  // First pass: collect parts and pool canonical texts, so pooled lookups
  // below do not depend on the package iteration order.
  const rows = [];
  for (const pkg of metadata.packages) {
    // Packages without a `source` are workspace members / path deps
    // (including the git submodules) — this product itself, not a
    // third-party component.
    if (!pkg.source) continue;
    const parts = collectLicenseParts(dirname(pkg.manifest_path));
    // Pool verbatim canonical texts license-by-license (e.g. the standalone
    // LICENSE-APACHE / LICENSE-BOOST files of dual-licensed crates), so
    // packages that ship no text at all can reuse them byte-identically.
    for (const part of parts) {
      const sniffed = sniffLicense(part.text);
      if (sniffed && !canonicalTextPool.has(poolKey(sniffed))) {
        canonicalTextPool.set(poolKey(sniffed), part.text);
      }
    }
    rows.push({ pkg, parts });
  }
  const entries = [];
  for (const { pkg, parts } of rows) {
    const base = {
      origin: 'rust',
      name: pkg.name,
      version: pkg.version,
      license: pkg.license || sniffLicense(joinParts(parts)) || 'UNKNOWN',
      repository: pkg.repository || pkg.homepage || null,
    };
    const applied = applyLicensePolicy(base, parts, (pkg.authors ?? []).join(', ') || undefined);
    const { text, ...fields } = applied;
    entries.push({ ...base, ...fields, textFile: storeText(text) });
  }
  entries.sort((a, b) => a.name.localeCompare(b.name));
  return entries;
}

// ---------------------------------------------------------------------------
// npm packages
// ---------------------------------------------------------------------------
function collectNpm(warnings) {
  const lockFile = join(WEBUI_DIR, 'package-lock.json');
  if (!existsSync(lockFile)) throw new Error('webui/package-lock.json not found');
  const lock = JSON.parse(readFileSync(lockFile, 'utf8'));
  const packages = lock.packages ?? {};
  const seen = new Set();
  const entries = [];
  for (const [key, info] of Object.entries(packages)) {
    if (key === '') continue; // the webui itself
    if (info.dev) continue; // dev-only subtree, not shipped in the bundle
    if (info.link) continue; // workspace link, not a real package
    // The lock lists optional deps for every platform; ones not installed
    // here never end up in this build's bundle.
    if (info.optional && !existsSync(join(WEBUI_DIR, key))) continue;

    const name = info.name ?? key.split('node_modules/').pop();
    const dedupeKey = `${name}@${info.version}`;
    if (seen.has(dedupeKey)) continue;
    seen.add(dedupeKey);

    const dir = join(WEBUI_DIR, key);
    // Best-effort package.json read: license fallback plus the real
    // repository link (lock entries carry only the tarball URL).
    let pkgJson = {};
    try {
      pkgJson = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf8'));
    } catch {
      /* missing or unreadable — metadata from the lock entry only */
    }
    let license = info.license ?? pkgJson.license ?? null;
    let parts = collectLicenseParts(dir);
    if (!license || /^SEE LICENSE IN /i.test(license)) {
      // "SEE LICENSE IN <file>" points at a file inside the package.
      const ref = license?.replace(/^SEE LICENSE IN /i, '');
      if (ref && existsSync(join(dir, ref))) {
        try {
          const refText = readText(join(dir, ref)).trim();
          if (refText && parts.length === 0) parts = [{ file: ref, text: refText }];
        } catch {
          /* fall through */
        }
      }
      license = sniffLicense(joinParts(parts)) ?? license ?? 'UNKNOWN';
    }
    if (!existsSync(dir)) {
      warnings.push(`npm package ${dedupeKey} is in the lock but not installed (no text)`);
    }
    const base = {
      origin: 'npm',
      name,
      version: info.version ?? '',
      license,
      repository: repoUrl(pkgJson) ?? pkgJson.homepage ?? info.resolved ?? null,
    };
    const holder =
      typeof pkgJson.author === 'string'
        ? pkgJson.author
        : typeof pkgJson.author?.name === 'string'
          ? pkgJson.author.name
          : undefined;
    const applied = applyLicensePolicy(base, parts, holder);
    const { text, ...fields } = applied;
    entries.push({ ...base, ...fields, textFile: storeText(text) });
  }
  entries.sort((a, b) => a.name.localeCompare(b.name));
  return entries;
}

// ---------------------------------------------------------------------------
// Manual entries (components outside cargo/npm, e.g. ConPTY)
// ---------------------------------------------------------------------------
function collectManual(warnings) {
  const file = join(WEBUI_DIR, 'scripts', 'manual-licenses.json');
  if (!existsSync(file)) return [];
  let list;
  try {
    list = JSON.parse(readFileSync(file, 'utf8'));
  } catch (e) {
    warnings.push(`manual-licenses.json is invalid: ${e.message}`);
    return [];
  }
  return list.map((entry) => {
    let text = null;
    if (entry.licenseFile) {
      const path = join(REPO_ROOT, entry.licenseFile);
      if (existsSync(path)) text = readText(path).trim() || null;
      else warnings.push(`manual entry ${entry.name}: licenseFile missing (${entry.licenseFile})`);
    }
    if (!text && entry.text) text = entry.text;
    const base = {
      origin: 'manual',
      name: entry.name,
      version: entry.version ?? '',
      license: entry.license ?? sniffLicense(text) ?? 'UNKNOWN',
      repository: entry.repository ?? null,
    };
    const applied = applyLicensePolicy(base, [], null, text);
    const { text: _alreadyStored, ...fields } = applied;
    return { ...base, ...fields, textFile: storeText(text) };
  });
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------
const warnings = [];
const rust = collectRust(warnings);
const npm = collectNpm(warnings);
const manual = collectManual(warnings);

const generatedAt = new Date().toISOString();

// Rebuild public/licenses from scratch so stale hash files never linger.
rmSync(TEXT_DIR, { recursive: true, force: true });
mkdirSync(TEXT_DIR, { recursive: true });
for (const { fileName, text } of textStore.values()) {
  writeFileSync(join(TEXT_DIR, fileName), `${text}\n`, 'utf8');
}

const manifest = {
  version: 1,
  generatedAt,
  counts: { rust: rust.length, npm: npm.length, manual: manual.length },
  warnings,
  licensePolicy: {
    permissiveOnly: true,
    selections: policy.selections,
    weakCopyleft: policy.weak,
    synthesizedTexts: policy.synthesized,
    violations: policy.violations,
    needsReview: policy.reviews,
  },
  entries: [...rust, ...npm, ...manual],
};
writeFileSync(join(PUBLIC_DIR, 'licenses.json'), `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');

// Aggregated plain-text notices shipped next to the binaries. The full
// texts repeat once per unique text (not once per package), clustered
// under the list of packages sharing them — otherwise 700+ crates with
// the Apache-2.0 text alone would bloat the file to many megabytes.
const groups = [
  { origin: 'rust', title: 'RUST CRATES', entries: rust },
  { origin: 'npm', title: 'NPM PACKAGES', entries: npm },
  { origin: 'manual', title: 'BUNDLED COMPONENTS', entries: manual },
];
const bar = '='.repeat(78);
const rule = '-'.repeat(78);

/** Human-readable summary of the permissive-only policy for the notices. */
function policyLines() {
  const out = [];
  const section = (title, items) => {
    if (items.length === 0) return;
    out.push(title);
    for (const item of items) out.push(`  - ${item}`);
    out.push('');
  };
  if (policy.selections.length > 0) {
    out.push(
      'Components dual-licensed with a copyleft alternative are used here',
      'strictly under their permissive alternative; the copyleft text is not',
      'shipped or relied upon:',
    );
    for (const s of policy.selections) out.push(`  - ${s}`);
    out.push('');
  }
  section(
    'Weak copyleft (MPL-2.0): shipped unmodified; any modification to these' +
      '\nfiles must itself be published under MPL-2.0:',
    policy.weak,
  );
  section(
    'Packages shipping no license file: a canonical text is supplied by the' + '\npackager:',
    policy.synthesized,
  );
  out.push(
    'crossbeam-channel embeds a Go-derived matching algorithm under CC-BY-3.0',
    '(its LICENSE-THIRD-PARTY text, reproduced below, preserves the required',
    'attribution).',
    '',
  );
  return out;
}

const lines = [
  'AURGA Vibe Terminal',
  'THIRD-PARTY SOFTWARE NOTICES AND LICENSE TERMS',
  '',
  `Generated: ${generatedAt}`,
  'by webui/scripts/generate-licenses.mjs from Cargo.lock and',
  'webui/package-lock.json. The same information is shown in the web UI',
  'under Help > Open Source Licenses.',
  '',
  'LICENSE POLICY: every third-party component in Vibe Terminal must be',
  'available under a permissive license (MIT, Apache-2.0, BSD, ISC, Zlib, ...)',
  'or weak file-level copyleft (MPL-2.0, shipped unmodified). No component',
  'is used under GPL/LGPL/AGPL/SSPL terms.',
  '',
  ...policyLines(),
];
for (const group of groups) {
  if (group.entries.length === 0) continue;
  lines.push(bar, `${group.title} (${group.entries.length})`, bar, '');
  for (const e of group.entries) {
    const license = e.licenseOriginal
      ? `${e.license} (applied from: ${e.licenseOriginal})`
      : e.license;
    const bits = [`${e.name} ${e.version}`.trim(), license];
    if (e.repository) bits.push(e.repository);
    lines.push(bits.join(' | '));
  }
  lines.push('');
  // Cluster entries by identical license text; each text prints once.
  const byText = new Map(); // textFile -> entries
  const noText = [];
  for (const e of group.entries) {
    if (e.textFile) {
      const list = byText.get(e.textFile) ?? [];
      list.push(e);
      byText.set(e.textFile, list);
    } else {
      noText.push(e);
    }
  }
  lines.push(bar, `${group.title} — LICENSE TEXTS`, bar, '');
  for (const [, list] of [...byText.entries()].sort((a, b) =>
    a[1][0].name.localeCompare(b[1][0].name),
  )) {
    const notes = [...new Set(list.map((e) => e.licenseNote).filter(Boolean))];
    lines.push(
      `Applies to: ${list.map((e) => `${e.name} ${e.version}`.trim()).join(', ')}`,
      rule,
      ...notes.flatMap((n) => [`Note: ${n}`]),
      ...(notes.length > 0 ? [''] : []),
      textByFile.get(list[0].textFile) ?? '',
      '',
    );
  }
  if (noText.length > 0) {
    lines.push(
      `Ships without a bundled license file (see each project page): ${noText
        .map((e) => `${e.name} ${e.version} (${e.license})`.trim())
        .join(', ')}`,
      '',
    );
  }
}
writeFileSync(join(PUBLIC_DIR, 'third-party-notices.txt'), `${lines.join('\n')}\n`, 'utf8');

console.log(
  `[licenses] ${rust.length} rust crates, ${npm.length} npm packages, ` +
    `${manual.length} manual entries, ${textStore.size} unique license texts -> public/licenses.json`,
);
for (const s of policy.selections) console.log(`[licenses] selection: ${s}`);
for (const w of policy.weak) console.warn(`[licenses] weak-copyleft: ${w}`);
for (const s of policy.synthesized) console.log(`[licenses] synthesized text: ${s}`);
for (const w of warnings) console.warn(`[licenses] warning: ${w}`);

if (policy.violations.length > 0 || policy.reviews.length > 0) {
  for (const v of policy.violations) console.error(`[licenses] POLICY VIOLATION: ${v}`);
  for (const r of policy.reviews) console.error(`[licenses] POLICY REVIEW: ${r}`);
  if (process.env.ALLOW_NONPERMISSIVE === '1') {
    console.error('[licenses] continuing despite policy findings: ALLOW_NONPERMISSIVE=1 is set');
  } else {
    console.error(
      '[licenses] permissive-only license policy failed; replace the dependency above ' +
        '(or set ALLOW_NONPERMISSIVE=1 to temporarily downgrade this to a warning)',
    );
    process.exit(1);
  }
}
