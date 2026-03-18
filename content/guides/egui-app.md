---
title: "Building a GUI app"
description: "Make a desktop app with egui, then ship it on GitHub Releases using a workflow."
week: 2

prize:
  title: "Ferris Shirt"
  description: "A super comfy Ferris the Crab shirt! Unlikely to have customs fees, but you should still be prepared."
---

Welcome back! If you haven't done the first week yet, [go do that first](/guides/getting-started).

This week, you'll be making a **desktop GUI application** using [egui](https://www.egui.rs) and publishing it via GitHub Releases. We'll walk through building a to-do list app together to teach you the fundamentals, and then you'll go off and build something of your own.

Unlike last week, this guide is a checklist rather than a step-by-step tutorial. You'll need to read docs, explore examples and figure things out yourself. That's the point!

![Image of Rerun](https://cdn.mahadk.com/s/v3/a1ddf3058341e134_image.png)

## Using Hackatime (+ using AI)

For this week, please use [Lapse](https://lapse.hackclub.com) or the regular Hackatime plugin for your editor. Please don't use AI whilst working on these workshops - this includes for research. The [#resolution-rust Slack channel](https://hackclub.enterprise.slack.com/archives/C0AFY7A312P) is a great place to ask for help!

## Getting help

Same as last week:

- In the [#resolution-rust Slack channel](https://hackclub.enterprise.slack.com/archives/C0AFY7A312P), or
- Via email: [mahad@hackclub.com](mailto:mahad@hackclub.com)

## What is egui?

Before we start writing code, let's talk about what egui actually is.

You might be used to *retained mode* UI frameworks. In React, for example, you describe your UI once and then update it by changing state. The framework keeps track of the UI tree and works out what to re-render for you.

egui works completely differently. It uses an *immediate mode* paradigm, which means you **redraw the entire UI every single frame**. That means no virtual DOM or diffing behind the scenes! Every single frame, your code runs top to bottom and says "put a button here, put a label there". If the button was clicked, you find out right then and there. It's similar to how games work!

This sounds mad, but it's actually quite nice to work with. Your UI code ends up being very straightforward: it's just a function that draws widgets and responds to interactions, which means you don't need to worry about things like callbacks!

Here's the mental model:

```
loop {
    // This runs ~60 times per second
    draw_heading("My App");
    if draw_button("Click me").was_clicked() {
        do_something();
    }
}
```

That's basically it. egui handles the actual rendering, input and windowing for you. If you're still a bit confused, egui's ["Why immediate mode?"](https://github.com/emilk/egui?tab=readme-ov-file#why-immediate-mode) documentation explains it well.

**eframe** is the framework that wraps egui and handles creating a native window for you. You'll be adding eframe as a dependency, and it re-exports egui so you don't need to add egui separately.

## Building a to-do list app

Let's build a to-do list app together. It won't be anything fancy, and you _will_ need to **make a new app** at the end rather than just modifying the finished product like in week 1, but it'll teach you how eframe apps are structured, how widgets work and how to manage state. After this, you'll take what you've learnt and build your own project.

### 1. Setting up the project

Create a new project and add eframe:

```bash
cargo new todo_app
cd todo_app
cargo add eframe
```

`cargo add` is a handy command that adds a dependency to your `Cargo.toml` and fetches the latest version for you. If you open `Cargo.toml`, you'll see it's added something like this:

```toml
[dependencies]
eframe = "0.33.3"
```

Your version number might be slightly different, and that's fine.

### 2. The bare minimum eframe app

Before we build the to-do list, let's get the absolute simplest eframe app running. Replace everything in `src/main.rs` with:

```rust
use eframe::egui;

struct TodoApp {}

impl TodoApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {}
    }
}

impl eframe::App for TodoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My To-Do List");
        });
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "To-Do List",
        native_options,
        Box::new(|cc| Ok(Box::new(TodoApp::new(cc)))),
    )
}
```

Run it with `cargo run`. You should see a window pop up with "My To-Do List" as a heading. Brilliant! Let's break this down.

#### The app struct

```rust
struct TodoApp {}

impl TodoApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {}
    }
}
```

Every eframe app needs a struct that holds your app's state. Right now ours is empty, but we'll add fields to it shortly. The `new` function takes a `CreationContext` (which you can use to customise fonts, visuals and so on) but we don't need it yet, so the parameter name is prefixed with `_` to tell Rust we're intentionally not using it.

#### The `App` trait

```rust
impl eframe::App for TodoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My To-Do List");
        });
    }
}
```

The `eframe::App` trait has one required method: `update`. This is where all your UI code goes. Remember the immediate mode model? This function is called **every frame**. Each time it runs, you draw your entire UI from scratch.

A *trait* in Rust is a bit like an interface in other languages. By implementing `eframe::App` for our struct, we're telling eframe "here's how to draw my app". If you want to learn a bit more about traits, check out the [Rust book](https://doc.rust-lang.org/book/ch10-02-traits.html) or this snazzy [explainer](/reference/lazy-generics-and-raii) (which I would highly recommend reading! It'll make your life a lot easier)

Inside `update`, we use `egui::CentralPanel` to create a panel that fills the whole window. The `.show()` method takes a closure (an inline function) that receives a `ui` handle. You use this `ui` handle to add widgets.

_"What's a closure?"_ It's the `|ui| { ... }` bit. Think of it as an anonymous function. The `|ui|` part is the parameter list (like `(ui)` in a regular function), and the `{ ... }` is the body. Closures are used heavily in egui because each container widget (panels, layouts, scroll areas) needs to know what to draw inside it.

#### The main function

```rust
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "To-Do List",
        native_options,
        Box::new(|cc| Ok(Box::new(TodoApp::new(cc)))),
    )
}
```

`eframe::run_native` is what actually opens the window and starts the event loop. It takes three arguments:

1. **The app name** (`"To-Do List"`) - this sets the window title and is used for persistence
2. **Native options** - window configuration like size, position and so on. We're using the defaults for now
3. **An app creator** - a closure that creates your app. The `Box::new(...)` stuff is boilerplate you'll see in every eframe app. Don't worry too much about what `Box` is for now; we'll cover it in a future week

Notice that `main` returns `eframe::Result`. This is because `run_native` can fail (e.g. if it can't create a window), and returning the result lets Rust handle that for us.

### 3. Adding state

A to-do list needs to keep track of, well, to-dos. Let's define what a to-do item looks like and add some state to our app. Update the top of your file:

```rust
use eframe::egui;

struct Todo {
    text: String,
    done: bool,
}

struct TodoApp {
    todos: Vec<Todo>,
    new_todo_text: String,
}

impl TodoApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            todos: Vec::new(),
            new_todo_text: String::new(),
        }
    }
}
```

We've added two structs:

- `Todo` holds a single to-do item with its text and whether it's been completed
- `TodoApp` now has a `Vec<Todo>` (a list of to-dos) and a `String` to hold what the user is currently typing into the input field

`Vec::new()` creates an empty vector and `String::new()` creates an empty string. These are the initial values when the app launches.

### 4. Building the UI

Now for the fun part. Let's update the `update` method to draw our actual to-do list interface. Replace the entire `impl eframe::App for TodoApp` block:

```rust
impl eframe::App for TodoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Let's add a heading for our app!
            ui.heading("My To-Do List");

            ui.add_space(10.0);

            // Input area for adding new to-dos.
            // We use a horizontal layout to keep the input and button together.
            // Otherwise, the button would be on a separate line, which looks weird!
            ui.horizontal(|ui| {
                // The text input field for the new to-do. It's a single-line input.
                let text_input = ui.text_edit_singleline(&mut self.new_todo_text);

                // The "Add" button. It adds the new to-do to the list when clicked or Enter is pressed.
                // We also check that the input is not empty before adding - no point adding an empty to-do!
                if (ui.button("Add").clicked()
                    || (text_input.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                    && !self.new_todo_text.is_empty()
                {
                    // Okay, so let's add the new to-do to the list.
                    self.todos.push(Todo {
                        text: self.new_todo_text.clone(),
                        done: false,
                    });
                    // And clear the input field for the next to-do.
                    self.new_todo_text.clear();
                    // And add focus back to the input field so the user can add something else!
                    text_input.request_focus();
                }
            });

            // Let's add some space and a separator before the stats.
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);

            // Now, we're going to add a label with the number of completed todos.
            // Because egui is an Immediate Mode GUI, this label will automatically update whenever the todos change.
            let total = self.todos.len();
            let done_count = self.todos.iter().filter(|t| t.done).count();
            ui.label(format!("{done_count} of {total} tasks completed"));

            ui.add_space(5.0);

            // Let's show a list of todos!
            let mut to_remove: Option<usize> = None;

            for (i, todo) in self.todos.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    // A checkbox to mark the todo as done.
                    // We use a &mut (mutable reference) so that the checkbox can update the todo in place.
                    ui.checkbox(&mut todo.done, "");

                    if todo.done {
                        ui.label(
                            egui::RichText::new(&todo.text).strikethrough(),
                        );
                    } else {
                        ui.label(&todo.text);
                    }

                    if ui.button("X").clicked() {
                        to_remove = Some(i);
                    }
                });
            }

            // Remove the item outside the loop to avoid borrow issues.
            if let Some(index) = to_remove {
                self.todos.remove(index);
            }

            // And finally, a "clear completed" button.
            // Let's check if we have any completed todos in the first place!
            // The `any` method returns `true` if the closure returns `true` for _any_ element in the iterator.
            if self.todos.iter().any(|t| t.done) {
                // Let's add some space before the button.
                ui.add_space(10.0);
                if ui.button("Clear completed").clicked() {
                    // Here, we're modifying the todos vector to only retain the todos that are `!t.done` (i.e., not done).
                    // The rest get removed!
                    self.todos.retain(|t| !t.done);
                }
            }
        });
    }
}
```

That's a lot of new code! Let's go through it section by section.

#### The input area

```rust
ui.horizontal(|ui| {
    let text_input = ui.text_edit_singleline(&mut self.new_todo_text);

    if (ui.button("Add").clicked()
        || (text_input.lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter))))
        && !self.new_todo_text.is_empty()
    {
        self.todos.push(Todo {
            text: self.new_todo_text.clone(),
            done: false,
        });
        self.new_todo_text.clear();
        text_input.request_focus();
    }
});
```

`ui.horizontal()` lays out its children in a row rather than stacked vertically (which is the default). Inside, we have:

- `ui.text_edit_singleline(&mut self.new_todo_text)` draws a text input field. The `&mut` means we're giving egui a *mutable reference* to our string so it can update it as the user types. The function returns a `Response` that we store in `text_input` so we can check whether the user pressed Enter.
- `ui.button("Add").clicked()` draws a button and returns `true` on the frame it was clicked. This is the immediate mode magic: drawing the button and checking for clicks happen in the same expression.
- We also check if the user pressed Enter whilst the text input was focused. `text_input.lost_focus()` is `true` when the input *just* lost focus (which happens when you press Enter), and `ui.input(|i| i.key_pressed(egui::Key::Enter))` confirms it was specifically the Enter key.
- `.clone()` makes a copy of the string. We need this because we want to move the text into the new `Todo` whilst keeping `new_todo_text` around to clear it. If you've done week 1, you might remember that Rust is strict about ownership; `.clone()` is how you say "make a copy rather than moving the original".
- `.clear()` empties the input field, and `.request_focus()` puts the cursor back in the text box so the user can immediately type another item.

#### The stats line

```rust
let total = self.todos.len();
let done_count = self.todos.iter().filter(|t| t.done).count();
ui.label(format!("{done_count} of {total} tasks completed"));
```

Nothing too wild here. `.len()` gives us the number of items in the vector. `.filter(|t| t.done)` keeps only the items where `done` is `true`, and `.count()` counts them up. `ui.label()` draws some non-interactive text.

#### The to-do list

```rust
let mut to_remove: Option<usize> = None;

for (i, todo) in self.todos.iter_mut().enumerate() {
    ui.horizontal(|ui| {
        ui.checkbox(&mut todo.done, "");

        if todo.done {
            ui.label(
                egui::RichText::new(&todo.text).strikethrough(),
            );
        } else {
            ui.label(&todo.text);
        }

        if ui.button("X").clicked() {
            to_remove = Some(i);
        }
    });
}

if let Some(index) = to_remove {
    self.todos.remove(index);
}
```

We use `.iter_mut()` here instead of `.iter()` because we need to *modify* each to-do's `done` field when the checkbox is toggled. `.enumerate()` gives us both the index and the item, just like we used in the HN CLI last week.

For each to-do, we draw a row with:

- A **checkbox** bound to `todo.done`. When the user clicks it, egui flips the boolean for us automatically because we passed `&mut todo.done`.
- A **label** showing the text. If the to-do is done, we use `egui::RichText` to add a strikethrough effect. `RichText` lets you style text with things like bold, italics, colour and size.
- An **"X" button** to delete the item.

The deletion pattern is worth understanding. We can't remove an item from `self.todos` whilst we're iterating over it (Rust won't let you, and for good reason). So instead, we note *which* index to remove using `Option<usize>`, and do the actual removal after the loop finishes.

`if let Some(index) = to_remove` is a pattern match. It says "if `to_remove` is `Some(value)`, extract the value into `index` and run this block". If it's `None` (meaning nothing was deleted), the block is skipped.

#### The "clear completed" button

```rust
if self.todos.iter().any(|t| t.done) {
    ui.add_space(10.0);
    if ui.button("Clear completed").clicked() {
        self.todos.retain(|t| !t.done);
    }
}
```

This button only appears when there are completed items. `.any()` returns `true` if at least one item matches the condition. `.retain()` is the opposite of `.filter()` for vectors: it keeps only the items where the closure returns `true`, removing the rest *in place*.

### 5. Running it

Run `cargo run` and you should have a working to-do list. Try it out:

- Type some tasks and press Enter or click "Add"
- Tick the checkboxes to mark tasks as done (they'll get a strikethrough)
- Click "X" to delete individual items
- Click "Clear completed" to remove all finished tasks

### The full code

Here's the complete `src/main.rs` for reference (excluding comments):

```rust
use eframe::egui;

struct Todo {
    text: String,
    done: bool,
}

struct TodoApp {
    todos: Vec<Todo>,
    new_todo_text: String,
}

impl TodoApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            todos: Vec::new(),
            new_todo_text: String::new(),
        }
    }
}

impl eframe::App for TodoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My To-Do List");

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let text_input = ui.text_edit_singleline(&mut self.new_todo_text);

                if (ui.button("Add").clicked()
                    || (text_input.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                    && !self.new_todo_text.is_empty()
                {
                    self.todos.push(Todo {
                        text: self.new_todo_text.clone(),
                        done: false,
                    });
                    self.new_todo_text.clear();
                    text_input.request_focus();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);

            let total = self.todos.len();
            let done_count = self.todos.iter().filter(|t| t.done).count();
            ui.label(format!("{done_count} of {total} tasks completed"));

            ui.add_space(5.0);

            let mut to_remove: Option<usize> = None;

            for (i, todo) in self.todos.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut todo.done, "");

                    if todo.done {
                        ui.label(
                            egui::RichText::new(&todo.text).strikethrough(),
                        );
                    } else {
                        ui.label(&todo.text);
                    }

                    if ui.button("X").clicked() {
                        to_remove = Some(i);
                    }
                });
            }

            if let Some(index) = to_remove {
                self.todos.remove(index);
            }

            if self.todos.iter().any(|t| t.done) {
                ui.add_space(10.0);
                if ui.button("Clear completed").clicked() {
                    self.todos.retain(|t| !t.done);
                }
            }
        });
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "To-Do List",
        native_options,
        Box::new(|cc| Ok(Box::new(TodoApp::new(cc)))),
    )
}
```

## What you learnt

Let's recap what was covered:

- **Immediate mode UI**: your `update` function runs every frame and draws the entire UI from scratch. No retained widget tree, no callbacks.
- **The eframe app structure**: a struct for state, `impl eframe::App` with an `update` method, and `eframe::run_native` to launch it.
- **egui widgets**: `heading`, `label`, `button`, `text_edit_singleline`, `checkbox` and `RichText` for styled text.
- **Layouts**: `ui.horizontal()` to lay out widgets in a row, `ui.add_space()` for spacing and `ui.separator()` for visual dividers.
- **Mutable references in UI**: passing `&mut` to widgets like `text_edit_singleline` and `checkbox` so egui can read and write your state directly.
- **The `.clicked()` pattern**: drawing a widget and checking for interaction in one expression.
- **Iterator methods**: `.filter()`, `.count()`, `.any()` and `.retain()` for working with collections.

## Now build your own!

The to-do list was a guided exercise. For your actual submission, you need to build something **different** and more interesting. The idea is yours, but please do try to make something that you'll enjoy making and I'll enjoy reviewing! Some suggestions to get you going:

- A Pomodoro timer
- A Markdown previewer
- An RSS reader
- A native app for Hackatime
  - If you want help making this, send me a DM! I work on Hackatime and would love to help you out :)
  - [The docs](https://hackatime.hackclub.com/docs) may be useful too.
- A HTTP client app, like Postman
- Or perhaps build upon the HN CLI project we made last week and make a GUI for listing and reading HN stories?

And some suggestions as to what NOT to make:

- A unit converter
- A calculator
- A simple to-do list - add some pizzazz!
- Basically, anything you'll be making for this pathway and then will never use again :P

Pick something that interests you and that you can reasonably finish in 2-3 hours. There is no strict rule here, and hour counts are not really going to be taken into consideration at all, but it should be something that's better than hello world or our todo list! This is super subjective so please do ask if you're unsure.

Your app should use **at least three different widget types**. We used six in the tutorial above, but there are loads more to explore:

- `ui.slider()` for numeric ranges
- `ui.radio_value()` for single-select options
- `ui.combo_box()` for dropdowns
- `egui::TopBottomPanel` and `egui::SidePanel` for app-level layout
- `egui::ScrollArea` for scrollable content
- `ui.collapsing()` for expandable sections
- `ui.color_edit_button_rgb()` for colour pickers

Check out the interactive demo to see everything that's available:

- 📖 [egui widget gallery (live demo)](https://www.egui.rs/#demo)
- 📖 [egui docs on docs.rs](https://docs.rs/egui/latest/egui/)

### Publish on GitHub Releases

This is the shipping step. You need to:

- Push your code to a GitHub repository
- Create a GitHub Actions workflow that builds your app and uploads the binary to a GitHub Release
- Your workflow should ideally build for at least your own platform (macOS, Linux or Windows), but bonus points for cross-compilation

Useful links:

- 📖 [GitHub Actions quickstart](https://docs.github.com/en/actions/writing-workflows/quickstart)
- 📖 [`actions/upload-artifact`](https://github.com/actions/upload-artifact) for uploading build outputs
- 📖 [`softprops/action-gh-release`](https://github.com/softprops/action-gh-release) for creating releases with attached binaries
- 📖 [Rust CI with GitHub Actions](https://blog.logrocket.com/optimizing-ci-cd-pipelines-rust-projects/) for general Rust CI tips
- 📖 [Some optimization tips for Rust CI](https://blog.logrocket.com/optimizing-ci-cd-pipelines-rust-projects/)

A basic workflow looks something like: on push to a tag (e.g. `v*`), run `cargo build --release`, then upload the resulting binary from `target/release/`.

Workflows can be _annoying_ to set up, but they're worth it in the long run. They automate your build process and make it easy to release new versions. But if you're having trouble figuring out how to set one up, the [#resolution-rust Slack channel](https://hackclub.enterprise.slack.com/archives/C0AFY7A312P) (or my DMs) is a great place to ask for help!

Another cool resource is [hello-egui](https://github.com/lucasmerlin/hello_egui), a list of crates for things like animations, icons, forms, routing, webviews and more!

## Bonus challenges

If you finish early and want to stretch:

- Add a menu bar with `egui::TopBottomPanel`
- Persist your app's state between sessions using `eframe::Storage` or by saving to a file with `serde_json`
- Add keyboard shortcuts
- Style your app with custom fonts or a different `egui::Visuals` theme (dark mode, custom colours, etc.)

## Submitting your project

Push your code to GitHub. Make sure you have at least one release published with a working binary. Double check [Hackatime](https://hackatime.hackclub.com) to make sure your time's been tracked.

See you next week! 🦀
