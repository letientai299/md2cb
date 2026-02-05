# md2cb - Markdown to clipboard converter.

Converts GitHub Flavored Markdown (GFM) to rich HTML and copies it to the system
clipboard for pasting into Word, Google Docs, Pages, Teams, etc.

## Usage

```bash
cat file.md | md2cb
```

Then, paste the copied clipboard content to the target app.

Add `--edit/-e` flag to edit the content in `$EDITOR` before converting. `-e`
would open an empty markdown file if run without any input (file or stdin).

## Demo

> TODO (tai): prepare a demo image for several apps.

## Development

See `mise tasks` for list of common tasks. Use `mise dev` to start the 2 web
servers:

- http://localhost:9091/demo.md: Markdown preview rendered by [markserv][]:
- http://localhost:9090: [Froala][] editor for pasting the converted content

[markserv]: https://github.com/markserv/markserv
[froala]: https://froala.com

## Requirements

- https://mise.jdx.dev for manage dev toosl and tasks runner.
- Docker for running dev servers

## Notes

Most of the code was written by Claude Code, with some code review from Copilot!

At work I need to use Teams. It supports a few makdown features, but the editing
experience for long message isn't smooth. So, I often write in NVim and use the
below shell script to convert them before paste to Teams.

```bash
pandoc --from gfm --to html |
    textutil -convert rtf -stdin -stdout -format html |
    pbcopy -Prefer
```

The script work well for typical bullet list, but:

- Doesn't support image, mermaid.
- Mac only.

Hence, I build this.
