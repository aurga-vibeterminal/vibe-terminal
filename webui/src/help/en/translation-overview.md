Translation turns text you meet in the client into the language of your choice: AI chat replies, commit messages, code comments, or any selection in the code editor or a diff. A translation model configured on the server does the work — the client sends the text, renders the result, and caches it so you can flip between the original and the translation.

## Where translation is available

- **Chat panel** — every message header (your messages, model replies, and `Thinking…` blocks) carries a translate icon. One click replaces the message body with its translation; another click restores the original;
- **Commit message draft** — the commit message box in the Git panel grows a translate button once your draft spans several lines. It translates the whole draft in place, and one more click brings your original wording back;
- **Commit details popup** — hovering a commit in the Git history opens a details popup; the translate icon next to the hash swaps the commit message to your language;
- **Code editor & diff view** — select anything (block selections count too), right-click and pick `Translate`. A small globe badge marks the translated lines in the gutter, and a card with the translation opens next to the selection.

## How it works

1. The client sends the text to the server — plus the file name when the text was selected in the editor or a diff, so the model knows which comment syntax the file uses;
2. The server's translation LLM translates it into the target language configured in the Web UI. Markdown formatting (headings, lists, tables, code fences) is preserved; in file selections, comment markers such as `//` or `#` are recognized as decoration and stripped, while code and identifiers stay untouched;
3. The client renders the result as Markdown wherever the request came from — in place for chat messages, or in a scrollable card for editor and diff selections.

## What you need

- A server with the translation feature enabled — see *Server Setup* for the one-time configuration in the Web UI's `LLM` section;
- A desktop client connected to that server. Nothing is configured client-side: every surface picks up the same server settings automatically.

If translation is not configured, the client surfaces the server's error (for example in the translation card) instead of a result.
