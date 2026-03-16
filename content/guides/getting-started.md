---
title: "Getting started"
description: "Install Rust, learn how projects are structured, and make a basic HN top story viewer."
week: 1
---

Hello and welcome to the first week of the Resolution Rust pathway! By the end of the pathway, you'll be able to make projects in Rust, understand the "Rust way of thinking" and the ecosystem behind it, and make something useful by yourself.

The below is an example of a "Hacker News" viewer tool. You should follow the tutorial and then **remix** it for a different API. Some examples:

- [Alpha Vantage](https://www.alphavantage.co) allows you to find trading and forex data. Perfect for Monitoring the Situation™
- [PokeAPI](https://pokeapi.co) for Pokemon!
- Or maybe use the [GitHub API](https://docs.github.com/en/rest) to view repo details from the terminal?

Note that this first week is intentionally fairly simple. **If the below task is too simple, please feel free to submit your own Rust project to the pathway instead**, bearing in mind that you'll still need to use Hackatime (see below).

## What this workshop isn't

This workshop isn't going to go over syntax minutiae or the like - this pathway is more of a hintsheet than a complete tutorial. I would highly, highly recommend you read this in conjunction with the [official Rust book](https://doc.rust-lang.org/book/), which is free online and super easy to follow.

We also won't be going over basic programming concepts (e.g. what variables and functions are). If you want to learn this stuff, then that's not a problem - the "General Coding" pathway can be done in tandem with this one and it'll help you understand what the guides are talking about.

Finally, this week isn't meant to restrict what projects you can submit. If you're confident enough to make a more advanced project, shoot me a message (see below) and I can work with you to get it submitted.

## Getting help

You might find yourself getting stuck as you work through the workshop. *That is totally okay* and we're more than happy to help you along the way.

There are two ways to do so:

- In the [#resolution-rust Slack channel](https://hackclub.enterprise.slack.com/archives/C0AFY7A312P), or:
- Via email: [hey@mahadk.com](mailto:hey@mahadk.com)

## Using Hackatime (+ using AI)

You need to use Hackatime whilst working on Resolution pathways!

For the first week, [**please use Lapse.**](https://lapse.hackclub.com) This is a pretty simple workshop and Lapse ensures that the time isn't undercounted! Future weeks will allow you to use either.

Furthermore, please don't use AI whilst working on these workshops - it gives you a false sense of security and you end up learning nothing.

## Prerequisites

### Setting up Rust

We'll be using the *rustup* tool to install Rust. It installs Rust (and its auxiliary tools like *Cargo* and *Clippy,* which we'll learn about later), keeps them up to date, and allows for switching versions.

If you are using macOS or Linux, then open the Terminal app and paste in the following, then hit Enter:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

If you're on Windows, visit [rustup.rs](http://rustup.rs/) and download and run the right installer for your computer.

### Setting up Hackatime

*You can skip this step if you have Hackatime already installed.*

First up, make an account! Make a visit to the [Hackatime sign-in page](https://hackatime.hackclub.com/signin) and sign up with your Hack Club account. After that, [visit the setup page](https://hackatime.hackclub.com/my/wakatime_setup) and follow the instructions to install Hackatime's plugins for your code editor.

## Creating a new project

Now let's get onto the fun part! You'll want to start up your code editor, then open the "Terminal" tab and run:

```bash
cargo new hn_cli
```

Let's see what we're doing here:

- *Cargo* is the package management tool for Rust. It's the CLI you use when you want to create project, manage dependencies, run formatting and linting checks, and more.
- We're creating a new *binary package* called "hn_cli". This means that we're making an application rather than a library. Rust uses *snake_case* for package names, hence the "hn_cli" name.

Now let's open the newly created "hn_cli" folder and see its structure:

```
❯ tree
.
├── Cargo.toml
└── src
    └── main.rs
```

Let's look at each file here:

- *Cargo.toml* is like *package.json* or *pyproject.toml* but for Rust projects. It includes details about your package like its name, license, version and dependencies.
- *main.rs* is the entry point of your program. When you run `cargo run`, Rust compiles and executes the code starting from the `main` function inside this file.

If you open `main.rs`, you'll see this:

```rust
fn main() {
    println!("Hello, world!");
}
```

Let's try it out! Run `cargo run` in your terminal:

```
❯ cargo run
   Compiling hn_cli v0.1.0
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.50s
   Running `target/debug/hn_cli`
Hello, world!
```

Awesome - you just ran your first Rust program! Now let's learn some Rust and build something real.

## Rust basics

### Variables

In Rust, you create variables with `let`:

```rust
let name = "Hacker News";
let score = 42;
```

By default, variables in Rust are _immutable_ by default — you can't change them after they're set. This is one of Rust's safety features. If you want a variable you can change, add `mut`:

```rust
let mut count = 0;
count = count + 1; // this is fine

let count_two = 0;
count_two = count_two + 1; // but this isn't, and it won't compile
```

### Types

Rust is a *statically typed* language, meaning every value has a type known at compile time. The compiler can usually figure out the type for you (this is called *type inference*), but you can also be explicit:

```rust
let name: &str = "Hacker News";
let score: u32 = 42;
let pi: f64 = 3.14;
let is_cool: bool = true;
```

Some common types you'll see:

| Type | What it is |
|------|-----------|
| `i32`, `i64` | Signed integers (can be negative) |
| `u32`, `u64` | Unsigned integers (zero or positive) |
| `f64` | Floating-point number |
| `bool` | `true` or `false` |
| `String` | An owned, growable string |
| `&str` | A string reference/slice (borrowed) |
| `Vec<T>` | A growable list (like an array) |
| `Option<T>` | A value that might or might not exist |

_"What the heck is a <T>?"_ you might ask. Good question! This is showing a _generic_, meaning that a `Vec` isn't just restricted to holding one type - for instance you can have a `Vec<String>` or `Vec<i32>` or even a `Vec<Vec<Vec<Option<i32>>>>`! Of course, you can't put an `i32` or some other non-`String` type into a `Vec<String>` - _all_ values in the `Vec` need to have the same type as `T`.

### Functions

Functions are declared with `fn`. The return type comes after `->`:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Notice there's no `return` keyword and no semicolon on the last line — in Rust, the last expression in a function is automatically its return value. You *can* use `return` for early returns, but leaving off the semicolon is the idiomatic way.

### Structs

Structs are how you define custom data types in Rust (like a class without methods, or a TypeScript interface):

```rust
struct Story {
    title: String,
    url: Option<String>,
    score: u32,
    by: String,
}
```

The `Option<String>` type means the `url` field might not have a value — it could be `Some("https://...")` or `None`. This is how Rust handles nullable values without null pointers.

### Printing

You've already seen `println!`. The `!` means it's a *macro*, not a regular function. You can embed variables using `{}`:

```rust
let name = "world";
println!("Hello, {}!", name);
```

## Adding dependencies

Now let's actually build our HN CLI! We need two *crates* (Rust's name for packages/libraries):

- **reqwest** — an HTTP client for making web requests
- **serde** — a serialization framework for parsing JSON

Open your `Cargo.toml` and add these under `[dependencies]`:

```toml
[package]
name = "hn_cli"
version = "0.1.0"
edition = "2024"

[dependencies]
reqwest = { version = "0.12", features = ["json", "blocking"] }
serde = { version = "1", features = ["derive"] }
```

A few things to note here:

- The `features` field enables optional functionality. For example, `reqwest`'s `"json"` feature lets us parse JSON responses directly, and `"blocking"` gives us a simple, synchronous HTTP client so we don't need to deal with async code.
- `serde`'s `"derive"` feature lets us use `#[derive(Deserialize)]` to automatically generate JSON parsing code for our structs — no manual parsing needed!

## Building the HN CLI

### The Hacker News API

Hacker News has a free, public API. Here's how it works:

1. First, get the story IDs. `https://hacker-news.firebaseio.com/v0/topstories.json` returns an array of up to 500 story IDs
2. Then, get the story info for each ID. `https://hacker-news.firebaseio.com/v0/item/{id}.json` returns the details for a specific story

A story object looks like this:

```json
{
  "by": "dhouston",
  "descendants": 71,
  "id": 8863,
  "score": 111,
  "time": 1175714200,
  "title": "My YC app: Dropbox - Throw away your USB drive",
  "type": "story",
  "url": "http://www.getdropbox.com/u/2/screencast.html"
}
```

### Writing the code

Now replace everything in `src/main.rs` with this:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Story {
    title: String,
    url: Option<String>,
    score: u32,
    by: String,
}

fn main() {
    println!("🔶 Top 10 Hacker News Stories\n");

    let client = reqwest::blocking::Client::new();

    let top_ids: Vec<u64> = client
        .get("https://hacker-news.firebaseio.com/v0/topstories.json")
        .send()
        .expect("Failed to fetch top stories")
        .json()
        .expect("Failed to parse story IDs");

    for (i, id) in top_ids.iter().take(10).enumerate() {
        let url = format!("https://hacker-news.firebaseio.com/v0/item/{id}.json");

        let story: Story = client
            .get(&url)
            .send()
            .expect("Failed to fetch story")
            .json()
            .expect("Failed to parse story");

        let link = story.url.as_deref().unwrap_or("(no URL)");
        println!("{}. {} ({} points by {})", i + 1, story.title, story.score, story.by);
        println!("   {}\n", link);
    }
}
```

Whew, that was a lot! Let's break down what's happening here, section by section.

### Imports and the Story struct

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Story {
    title: String,
    url: Option<String>,
    score: u32,
    by: String,
}
```

- `use serde::Deserialize;` brings in the `Deserialize` trait from the serde crate. For our intents and purposes, this means that we want to be able to take in, say, a JSON response, and convert (deserialize!) it into a struct.
- `#[derive(Deserialize)]` is a *macro* that tells Rust to automatically generate code that can parse JSON into our `Story` struct. Without this, we'd have to write all the parsing logic by hand.
- We only include the fields we care about - serde will simply ignore any extra fields in the JSON (like `descendants`, `time`, etc.).
- `url` is `Option<String>` because not every story has a URL (e.g. "Ask HN" posts have text instead).

### The main function

```rust
fn main() {
```

Just a regular `fn main()` - nothing special needed! We're using reqwest's *blocking* client, which means our HTTP requests run synchronously. This keeps the code simple and straightforward. Rust does have async, and it's something we'll be using in future weeks, but to keep things simple, we won't be using it for now.

### Fetching the top story IDs

```rust
let client = reqwest::blocking::Client::new();

let top_ids: Vec<u64> = client
    .get("https://hacker-news.firebaseio.com/v0/topstories.json")
    .send()
    .expect("Failed to fetch top stories")
    .json()
    .expect("Failed to parse story IDs");
```

- We create a reusable HTTP client with `reqwest::blocking::Client::new()`. The `blocking` module gives us a synchronous client — it sends a request and waits for the response, just like you'd expect.
- `.get(url)` builds a GET request, `.send()` sends it, and `.json()` parses the response body as JSON.
- `.expect("message")` is how you handle errors quickly — if something goes wrong, the program will crash with that message. In production code you'd use proper error handling, but this is fine for a learning project.
- `Vec<u64>` is a vector (dynamic array) of unsigned 64-bit integers — the story IDs.

### Looping through stories

```rust
for (i, id) in top_ids.iter().take(10).enumerate() {
```

- `.iter()` creates an iterator over the vector.
- `.take(10)` limits it to the first 10 items.
- `.enumerate()` gives us both the index (`i`) and the value (`id`) — like `enumerate()` in Python or `.forEach((item, index) => ...)` in JavaScript.

### Fetching and displaying each story

```rust
let url = format!("https://hacker-news.firebaseio.com/v0/item/{id}.json");

let story: Story = client
    .get(&url)
    .send()
    .expect("Failed to fetch story")
    .json()
    .expect("Failed to parse story");

let link = story.url.as_deref().unwrap_or("(no URL)");
println!("{}. {} ({} points by {})", i + 1, story.title, story.score, story.by);
println!("   {}\n", link);
```

- `format!` is like `println!` but returns a `String` instead of printing it.
- `{id}` inside the format string directly inserts the variable (this is called *inline formatting*).
- `.as_deref()` converts `Option<String>` to `Option<&str>`, and `.unwrap_or("(no URL)")` provides a fallback if the URL is `None`.

## Running it

Save the file and run:

```bash
cargo run
```

You should see something like this:

```
🔶 Top 10 Hacker News Stories

1. Some Cool Article (mass points by someuser)
   https://example.com/article

2. Show HN: My Cool Project (100 points by builder)
   https://github.com/example/project

...
```

Congratulations — you just built a working CLI in Rust that fetches live data from the internet! 🎉

## What you learned

Let's recap what you covered this week:

- Setting up a new project: Using `cargo new` to create a project, and `Cargo.toml` to manage dependencies
- How variables work in Rust: `let` for immutable, `let mut` for mutable
- We used Rust types: `String`, `&str`, `u32`, `Vec<T>`, `Option<T>`
- We used structs! Use the `struct` keyword to create one. A struct is basically a data type (e.g. `Vehicle`, `Comment`, `Post`)
- Functions! Use the `fn` keyword to define one, add return types with `->`, and how Rust uses implicit returns
- Making synchronous HTTP requests with `reqwest`
- Then automatically parsing JSON into Rust structs with `#[derive(Deserialize)]` and `serde`
- And last but not least, iterators: `.iter()`, `.take()`, `.enumerate()`. Use these when you're, well, _iterating_ over an array or collection!

## Challenges

Want to keep going? Try extending your CLI with these:

1. Show the comment count (`descendants` field) in the output.
2. Let the user choose between top, new, best, ask, or show stories by passing a command-line argument. Check out [`std::env::args`](https://doc.rust-lang.org/std/env/fn.args.html) for reading arguments. Or better yet, [the `clap` library!](https://lib.rs/clap) We'll be using it in future weeks so getting a handle on it now would work wonders.
3. Replace `.expect()` with proper error handling using `Result` and the `?` operator. The [Error Handling chapter](https://doc.rust-lang.org/book/ch09-00-error-handling.html) of the Rust book is a great resource. You can also use libraries like [color_eyre](http://lib.rs/color_eyre) for formatted errors.

## Submitting your project

Once you're done, push your code to a GitHub repository. You'll need the repo link for the submission form! You should also double check the [Hackatime site](https://hackatime.hackclub.com) to make sure that your time's been tracked.

See you next week! 🦀
