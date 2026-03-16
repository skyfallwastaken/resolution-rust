---
title: "Building a GUI app"
description: "Make a desktop app with egui, then ship it on GitHub Releases using a workflow."
week: 2
---

Welcome back! If you haven't done the first week yet, [go do that first](/guides/getting-started).

This week, you'll be making a **desktop GUI application** using [egui](https://www.egui.rs) and publishing it via GitHub Releases. Unlike last week, this guide is a checklist rather than a step-by-step tutorial. You'll need to read docs, explore examples and figure things out yourself. That's the point!

![Image of Rerun](https://cdn.mahadk.com/s/v3/a1ddf3058341e134_image.png)

<sub>Above is a screenshot of Rerun, a visualizer app made by the developers of egui!</sub>

I want to let you know in advance that this will likely be an uphill battle*. 

The idea is yours, but please do try to make something that you'll enjoy making and I'll enjoy reviewing! Some suggestions to get you going:

- A Pomodoro timer
- A Markdown previewer
- A RSS reader
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

Pick something that interests you and that you can reasonably finish in 3-4 hours.

## Using Hackatime (+ using AI)

For this week onwards, you can use either [Lapse](https://lapse.hackclub.com) or the regular Hackatime plugin. Please don't use AI whilst working on these workshops - this includes for research. The [#resolution-rust Slack channel](https://hackclub.enterprise.slack.com/archives/C0AFY7A312P) is a great place to ask for help!

## Getting help

Same as last week:

- In the [#resolution-rust Slack channel](https://hackclub.enterprise.slack.com/archives/C0AFY7A312P), or
- Via email: [hey@mahadk.com](mailto:hey@mahadk.com)

## The checklist

### 1. Set up a new project with eframe

Create a new project with `cargo new` and add [eframe](https://lib.rs/eframe) as a dependency. eframe is the framework that wraps egui and handles windowing for you.

- 📖 [eframe getting started](https://docs.rs/eframe/latest/eframe/#usage-native)
- 📖 [egui docs on docs.rs](https://docs.rs/egui/latest/egui/)

### 2. Create your app struct and implement `eframe::App`

You need a struct that holds your app's state (think: what data does your app need to keep track of?) and an `impl eframe::App` block with the `update` method.

- 📖 [eframe::App trait](https://docs.rs/eframe/latest/eframe/trait.App.html)
- 📖 [The egui demo app source](https://github.com/emilk/egui/blob/master/crates/egui_demo_app/src/apps/demo/demo_app_windows.rs) is worth a browse for patterns

### 3. Build a UI with egui widgets

egui has loads of built-in widgets. Your app should use **at least three different widget types**. Some options:

- `ui.label()` and `ui.heading()` for text
- `ui.text_edit_singleline()` or `ui.text_edit_multiline()` for input
- `ui.button()` for actions
- `ui.slider()` for numeric ranges
- `ui.checkbox()` and `ui.radio_value()` for selections
- `ui.combo_box()` for dropdowns
- Layouts with `ui.horizontal()`, `ui.vertical()`, `ui.columns()`, etc.

Explore the interactive widget gallery to see what's available:

- 📖 [egui widget gallery (live demo)](https://www.egui.rs/#demo)

### 4. Handle state and interaction

Your app should actually *do* something when the user interacts with it. Buttons should trigger logic, sliders should update values, text input should be used somewhere.

Remember that `update()` is called every frame. egui uses an *immediate mode* paradigm, which is quite different from retained mode frameworks like React or SwiftUI. Read up on this if the concept is new to you:

- 📖 [Immediate mode GUIs (egui explanation)](https://github.com/emilk/egui?tab=readme-ov-file#why-immediate-mode)

### 5. Add a custom window title and optionally an icon

Set your app's window title via `eframe::NativeOptions` and the app name parameter in `eframe::run_native`. If you want to go further, look into setting a window icon with `NativeOptions::viewport`.

### 6. Publish on GitHub Releases

This is the shipping step. You need to:

- Push your code to a GitHub repository
- Create a GitHub Actions workflow that builds your app and uploads the binary to a GitHub Release
- Your workflow should ideally build for at least your own platform (macOS, Linux or Windows), but bonus points for cross-compilation

Useful links:

- 📖 [GitHub Actions quickstart](https://docs.github.com/en/actions/writing-workflows/quickstart)
- 📖 [`actions/upload-artifact`](https://github.com/actions/upload-artifact) for uploading build outputs
- 📖 [`softprops/action-gh-release`](https://github.com/softprops/action-gh-release) for creating releases with attached binaries
- 📖 [Rust CI with GitHub Actions](https://blog.logrocket.com/configuring-ci-cd-pipeline-rust-projects/) for general Rust CI tips

A basic workflow looks something like: on push to a tag (e.g. `v*`), run `cargo build --release`, then upload the resulting binary from `target/release/`.

Workflows can be _annoying_ to set up, but they're worth it in the long run. They automate your build process and make it easy to release new versions. But if you're having trouble figuring out how to set one up, the [#resolution-rust Slack channel](https://hackclub.enterprise.slack.com/archives/C0AFY7A312P) (or my DMs) is a great place to ask for help!

## Bonus challenges

If you finish early and want to stretch:

- Add a menu bar with `egui::TopBottomPanel`
- Persist your app's state between sessions using `eframe::Storage` or by saving to a file with `serde_json`
- Add keyboard shortcuts
- Style your app with custom fonts or a different `egui::Visuals` theme (dark mode, custom colours, etc.)

## Submitting your project

Push your code to GitHub. Make sure you have at least one release published with a working binary. Double check [Hackatime](https://hackatime.hackclub.com) to make sure your time's been tracked.

See you next week! 🦀

---

\* _"Why do it in the first place then?", I hear you ask._ Good question! Week 2 is mostly meant to help you learn how to read documentation and understand how to use the libraries you're working with - after all, you won't have any idea what you're doing when building future projects if you don't know how to read the docs and ask for help when you run into issues!
