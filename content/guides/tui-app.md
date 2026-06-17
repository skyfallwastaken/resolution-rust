---
title: "Building a TUI app"
description: "Make a fully-fledged terminal UI app using ratatui."
week: 5

prize:
  title: "Rust Plushie"
  description: "An adorable plushie of Ferris the Crab <3"
---

## This week is now closed

We need to batch order the plushies, so this week is now closed.

---

Welcome to week 5! If you haven't done weeks 1 through 4 yet, please go back and finish those off first.

This week, you'll be making a **terminal UI (TUI)** app using [ratatui](https://ratatui.rs). TUIs are apps that live entirely inside your terminal but feel like proper full-screen interfaces. Think `htop` or `lazygit`. They're great for developer tools because they boot up instantly, work over SSH (more useful than you may think), and, when done right, feel great to use.

When you're done, you'll receive this awesome Ferris plushie!

![Image of a plushie](https://cdn.mahadk.com/s/v3/4d544c239ce0d232_image.png)

## Using Hackatime (+ using AI)

Same as before: use [Lapse](https://lapse.hackclub.com) or the regular Hackatime plugin. No AI please. The [#resolution-rust Slack channel](https://hackclub.enterprise.slack.com/archives/C0AFY7A312P) is the spot for questions.

## Getting help

- In the [#resolution-rust Slack channel](https://hackclub.enterprise.slack.com/archives/C0AFY7A312P), or
- Via email: [mahad@hackclub.com](mailto:mahad@hackclub.com)

## What is ratatui?

Ratatui is a Rust crate for drawing TUIs in the terminal. You give it a tree of widgets every frame and it works out what characters and ANSI escape codes to spit out (basically, what characters + special "control codes" to print out to change colours, layout the UI, etc). If you've done the egui week, you'll recognise that this sounds pretty similar to immediate mode, so your draw function runs every frame and rebuilds the UI from scratch.

The mental model:

```rust
loop {
    terminal.draw(|frame| {
        // describe what the screen looks like right now
        frame.render_widget(some_header, top_area);
        frame.render_widget(some_list, middle_area);
    });

    // read a key press, update state, loop again
}
```

Ratatui itself only handles _drawing_. For reading key presses and putting the terminal into raw mode, you pair it with a backend like [crossterm](https://lib.rs/crossterm). That's the combo we'll use here because it works on macOS, Linux and Windows.

## What you need to build

Your submission this week is a TUI app of your choosing. Pick anything you'd actually find useful! Here are the requirements:

1. **Uses at least 4 different ratatui widget types.** Some options: `Paragraph`, `List`, `Block`, `Tabs`, `Gauge`, `Chart`, `Table`, `Sparkline`. The widget gallery linked at the bottom has loads more.
2. **Pulls in real data from somewhere.** This could be a shell command (like `gh`, `kubectl` or `git`), an HTTP API, the file system or even system info via a crate like [sysinfo](https://lib.rs/sysinfo). No "hello world" inputs please!
3. **Has proper keyboard navigation.** Scrolling, tab switching, list selection, modal dialogs. Whatever fits your app, so long as the user can actually drive the app with your keyboard. You _can_ use mouse inputs too (OpenCode is an example of a UI I like that uses them nicely)
4. **Uses a proper layout** with multiple regions on screen. A header at the top with the actual content below is the minimum. Footers, sidebars or popups all count too.
5. **Published as a binary** either on crates.io (via `cargo publish`) or on GitHub Releases (using the same workflow approach from week 2).

Try to make something you'll actually use when this stuff is done. A weather widget, a Kubernetes pod browser, a music player frontend for `mpd`, a Hacker News reader (yes, again ;P), an interactive `git log` browser. Pick whatever scratches an itch for you.

The example below walks through making a GitHub PR comment viewer that reads data using the `gh` CLI. Use it as a reference for ratatui patterns. **Do not just submit a variant of it.**

## Now go build!

If you'd rather figure out how to use Ratatui yourself (which I'd encourage doing! helps you learn), then you can stop reading here! Have fun <3

If you want to go through a guide to help you learn it though, keep reading.

---

## Example: building a GitHub PR comment viewer

The app we'll build lets you invoke `cargo run -- hackclub/hackatime 1218` to pop up a TUI that shows a pull request's title, status, author and full comment thread. You can scroll through the comments with the arrow keys (or `j`/`k` if you're a vim sort of person) and press `q` to quit.

It looks roughly like this when running:

```
┌Pull Request──────────────────────────────────────────────────────┐
│ #123 Fix flaky test in worker queue  [OPEN]  opened by @somebody │
└──────────────────────────────────────────────────────────────────┘
┌Comments──────────────────────────────────────────────────────────┐
│ @reviewer1                                                       │
│ Looks good but can we add a regression test?                     │
│                                                                  │
│ @somebody                                                        │
│ Added one in the latest push.                                    │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
 j/k or arrows to scroll, q to quit
```

### 1. Why use `gh` instead of hitting the API directly?

The [GitHub CLI](https://cli.github.com) is already authenticated on most developer machines, handles pagination, respects rate limits and lets you query specific fields with `--json`. Shelling out to it is way less code than handling OAuth tokens ourselves, so we're going to lean on it.

You'll need `gh` installed and logged in. On a Mac, use:

```bash
brew install gh
```

If you use Linux then you're probably smart enough to figure out how to install it on your own :P

Check with:

```bash
gh auth status
```

If that errors, run `gh auth login` first.

### 2. Setting up the project

```bash
cargo new pr_viewer
cd pr_viewer
cargo add ratatui@0.29 crossterm@0.28
cargo add serde --features derive
cargo add serde_json
```

Quick rundown of what each crate is doing:

| Crate | What it does |
|-------|-------------|
| **ratatui** | The TUI drawing library. |
| **crossterm** | Backend for ratatui. Reads key events, enables raw mode, swaps to the alternate screen. |
| **serde** | Parses the JSON we get from `gh` into our Rust structs. |
| **serde_json** | The JSON support for serde. |

### 3. The data model

`gh pr view` can return JSON if you ask for it with `--json field1,field2,...`. We want the title, state, number, author and comments. Add these structs to the top of `src/main.rs`:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Author {
    login: String,
}

#[derive(Deserialize)]
struct Comment {
    author: Author,
    body: String,
}

#[derive(Deserialize)]
struct PrData {
    title: String,
    state: String,
    number: u64,
    author: Author,
    comments: Vec<Comment>,
}
```

`#[derive(Deserialize)]` is what tells serde "here's how to fill this struct from JSON". The field names have to match the JSON keys exactly. Nice and clean.

### 4. Calling `gh` and parsing the output

We use `std::process::Command` to shell out:

```rust
use std::process::Command;

fn fetch_pr(repo: &str, number: &str) -> Result<PrData, Box<dyn std::error::Error>> {
    let output = Command::new("gh")
        .args([
            "pr",
            "view",
            number,
            "--repo",
            repo,
            "--json",
            "title,state,number,author,comments",
        ])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "gh failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let pr: PrData = serde_json::from_slice(&output.stdout)?;
    Ok(pr)
}
```

`.output()` runs the command and gives us back the stdout, stderr and exit status. If `gh` exits non-zero (the PR doesn't exist, you're not logged in, etc.) we bail out with the stderr. Otherwise we hand the stdout bytes to `serde_json::from_slice` which deserialises straight into our `PrData` struct.

`Box<dyn std::error::Error>` is a handy catch-all error type. It's saying "this function can return any error type that implements `std::error::Error`". Good enough for a small app like this. In a larger codebase you'd usually reach for a crate like [anyhow](https://lib.rs/anyhow) or define your own error enum.

### 5. Parsing CLI args and booting the terminal

Inside `main`, we read the two positional arguments and then put the terminal into a state where ratatui can take over:

```rust
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::env;
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <owner/repo> <pr_number>", args[0]);
        std::process::exit(1);
    }
    let repo = &args[1];
    let number = &args[2];

    let pr = fetch_pr(repo, number)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut scroll: u16 = 0;
    let result = run_app(&mut terminal, &pr, &mut scroll);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}
```

There's a fair amount happening here so let's break it down:

- **Raw mode** turns off the terminal's line buffering and echoing, so we can read individual key presses as they happen rather than waiting for the user to hit Enter.
- **The alternate screen** is a separate buffer your terminal can switch to. When the app exits, your old shell prompt is exactly where you left it. This is what `vim` and `less` use too.
- We call `fetch_pr` _before_ booting the terminal so that any error message from `gh` shows up normally instead of getting eaten by the alternate screen.
- The setup and teardown are kept symmetrical. The `result` variable holds the outcome of `run_app` but we deliberately tear down the terminal first before propagating it, otherwise a panic or error would leave your terminal in a broken state.

The `enable_raw_mode`, `EnterAlternateScreen` and friends are the bits of boilerplate every crossterm-based TUI starts with. Copy them once, never write them again.

### 6. The draw loop

Now the actual UI. This is `run_app`:

```rust
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::time::Duration;

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    pr: &PrData,
    scroll: &mut u16,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ])
                .split(area);

            let header_text = format!(
                "#{} {}  [{}]  opened by @{}",
                pr.number, pr.title, pr.state, pr.author.login
            );
            let header = Paragraph::new(header_text)
                .block(Block::default().borders(Borders::ALL).title("Pull Request"))
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                );
            f.render_widget(header, chunks[0]);

            let mut lines: Vec<Line> = Vec::new();
            if pr.comments.is_empty() {
                lines.push(Line::from(Span::styled(
                    "no comments yet",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            for c in &pr.comments {
                lines.push(Line::from(Span::styled(
                    format!("@{}", c.author.login),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
                for body_line in c.body.lines() {
                    lines.push(Line::from(body_line.to_string()));
                }
                lines.push(Line::from(""));
            }

            let comments = Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title("Comments"))
                .wrap(Wrap { trim: false })
                .scroll((*scroll, 0));
            f.render_widget(comments, chunks[1]);

            let help = Paragraph::new("j/k or arrows to scroll, q to quit")
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(help, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Down | KeyCode::Char('j') => {
                            *scroll = scroll.saturating_add(1)
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            *scroll = scroll.saturating_sub(1)
                        }
                        KeyCode::PageDown => *scroll = scroll.saturating_add(10),
                        KeyCode::PageUp => *scroll = scroll.saturating_sub(10),
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}
```

That's the whole UI. Let's pick it apart.

#### The layout

```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);
```

Layouts in ratatui are vector-based. You give it a region (the full window in this case) plus a list of constraints, and it slices it up for you. The constraints we used:

- `Length(3)` for the header. Exactly 3 rows tall, no more no less. Enough for the top border, content and bottom border of a bordered block.
- `Min(0)` for the comments. Take whatever space is left over.
- `Length(1)` for the help line at the bottom. Just one row.

`chunks` is now a `Vec<Rect>` you can index into. `chunks[0]` is the header area and so on.

There are other constraint types like `Constraint::Percentage(50)` and `Constraint::Ratio(1, 3)`. Mix and match as you need.

#### Building styled text

```rust
lines.push(Line::from(Span::styled(
    format!("@{}", c.author.login),
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD),
)));
```

Ratatui has a `Line` and `Span` model that's quite different from egui. A `Span` is a chunk of text with a single style. A `Line` is a horizontal sequence of spans. So if you want "@author: hello" where only the @author is bold, you make two spans and put them on the same line.

For the comment bodies we just shove each line in as plain text. Real GitHub comments use Markdown, but rendering Markdown in a TUI is a whole other rabbit hole. There's a [tui-markdown](https://lib.rs/tui-markdown) crate if you want to go there!

#### Scrolling

```rust
.scroll((*scroll, 0))
```

`Paragraph` has a built-in scroll offset. The tuple is `(row_offset, col_offset)`. We're only scrolling vertically, hence the `0` for the column.

The trade-off with `Paragraph`-based scrolling: it doesn't know how tall the content actually is, so we can scroll past the end. For something nicer (with a scrollbar widget and proper bounds checking) ratatui has the [`Scrollbar`](https://docs.rs/ratatui/latest/ratatui/widgets/struct.Scrollbar.html) widget and `ScrollbarState`. Worth a look if you want polish.

`saturating_add` and `saturating_sub` are how you do "add 1 but don't underflow past 0" without writing explicit if statements. They saturate at the type's min and max, so `0u16.saturating_sub(1)` is just `0` instead of panicking. Handy!

#### Event polling

```rust
if event::poll(Duration::from_millis(100))? {
    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            match key.code { ... }
        }
    }
}
```

`event::poll(timeout)` waits up to `timeout` for an event. If something arrives within that window, it returns `true` and you call `event::read()` to actually grab it. If nothing happens, it returns `false` and we loop around to redraw.

We check `key.kind == KeyEventKind::Press` because on Windows you get both Press and Release events for the same key. Without this check, every key would register twice.

### 7. Running it

```bash
cargo run -- hackclub/hackatime 1218
```

(Pick a PR number that actually exists on a repo you have access to.) You should see a TUI pop up with the PR details. Arrow keys to scroll, `q` to quit.

If you see "gh failed" in your terminal, double check `gh auth status` and make sure the PR number actually exists on the repo you specified.

### Wrapping up the exemplar

What this code shows off:

- The standard ratatui + crossterm boot sequence (raw mode, alternate screen, restore on exit)
- Layouts with mixed constraint types
- The `Line` / `Span` text model for styled text
- `Paragraph` scrolling with `saturating_add`
- Polling for events without blocking the draw loop
- Shelling out to a CLI and parsing its JSON output with serde

That's a pretty solid base to build off of for your own project.

## Now build your own!

The PR viewer was a guided exercise. For your submission, build something that's actually yours. Some ideas:

- An interactive `git log` browser with a commit list on the left and the diff on the right
- A Hacker News reader that fetches stories via the Algolia API
- A music player frontend for `mpd` or `spotifyd`
- A Docker container dashboard (resurrect the bollard knowledge from week 3!)
- A system monitor using the [sysinfo](https://lib.rs/sysinfo) crate. CPU, memory, disk, processes
- A reddit reader, an RSS reader or an email client
- A package manager TUI (browse, search, install)

Try to avoid:

- A clone of the PR viewer above with one field swapped out
- A static dashboard that doesn't actually let the user _do_ anything
- Anything you won't use again after submitting

### Useful links

- [ratatui website](https://ratatui.rs) with tutorials and concept docs
- [ratatui widget gallery](https://ratatui.rs/showcase/widgets/) to see what's in the box
- [ratatui examples repo](https://github.com/ratatui/ratatui/tree/main/examples) for working code samples of every widget
- [Awesome Ratatui](https://github.com/ratatui/awesome-ratatui) for crates and apps built with ratatui
- [crossterm docs](https://docs.rs/crossterm/latest/crossterm/) for event handling and terminal manipulation

## Bonus challenges

If you've got time to spare:

- Add syntax highlighting using Syntect + proper Markdown support!
- Tables and Mermaid diagrams (`mermaid` code blocks) don't render properly - can you fix that?
- Add a way for me to merge PRs (incl. squash merges, merge merges (yes), rebase merges) and close them too
- Add a proper [`Scrollbar`](https://docs.rs/ratatui/latest/ratatui/widgets/struct.Scrollbar.html) widget that tracks position and content length properly
- Add a help popup (press `?` to toggle) using a centered floating rect
- Support multiple tabs with the `Tabs` widget
- Add mouse support by handling `Event::Mouse`
- Render Markdown comments using [tui-markdown](https://lib.rs/tui-markdown)
- Throw your app on [crates.io](https://crates.io) so others can `cargo install` it!
- Try something like [dioxus-tui](https://docs.rs/dioxus-tui/latest/dioxus_tui/)

## Submitting your project

Push the code to GitHub, publish a release (or a crates.io version) and submit the link. Double check [Hackatime](https://hackatime.hackclub.com) to make sure your time tracked.

See you next week!
