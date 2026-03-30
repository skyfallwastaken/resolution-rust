---
title: "Building a web service"
description: "Build an authenticated HTTP API in Axum"
week: 3

prize:
  title: "Cat Printer"
  description: "A tiny, programmable (!) cat/receipt printer! (you can alternatively get $15 in Hetzner credits if you're boring :P)"
---

Welcome to week 3! If you haven't done [week 1](/guides/getting-started) and [week 2](/guides/egui-app), go do those first.

This week, you'll be building a **web service** in Rust using [Axum](https://github.com/tokio-rs/axum). If you've used libraries like Express or Hono, Axum is basically the same thing but for Rust!

The exemplar below is some (simplified) code from Sequin, a project I'm making. The guide below makes HTTP API that lets you create, list, stop and remove Docker containers remotely. Think of it as a tiny, stripped-down version of something like [Coolify](https://coolify.io)'s data plane. It talks to Docker on your behalf over an authenticated API, and logs every action to a SQLite database.

**You do not have to make the below project** (actually, please don't!), but if you get stuck on something, then it may be useful to look at if you need help.

Make something useful! A to-do list API is.. well, not useful.

## Using Hackatime (+ using AI)

Same as before: use [Lapse](https://lapse.hackclub.com) or the regular Hackatime plugin. Please don't use AI whilst working on this. The [#resolution-rust Slack channel](https://hackclub.enterprise.slack.com/archives/C0AFY7A312P) is a great place to ask for help!

## Getting help

- In the [#resolution-rust Slack channel](https://hackclub.enterprise.slack.com/archives/C0AFY7A312P), or
- Via email: [mahad@hackclub.com](mailto:mahad@hackclub.com)

## What is Axum?

Axum is a web framework built on top of [Tokio](https://tokio.rs) and [Hyper](https://hyper.rs). Tokio is the most popular async runtime for Rust, and Axum made by the same team that builds Tokio itself, so the async integration is rock solid.

If you've used Express.js or Flask before, Axum will feel pretty familiar: you define routes, attach handler functions to them and the framework deals with the HTTP stuff behind the scenes. The big difference is that everything is async and strongly typed. Your handler functions declare what they need (e.g. a JSON body, path parameters, shared state) and Axum extracts those automatically.

Here's the mental model:

```
Router::new()
    .route("/hello", get(say_hello))        // GET /hello
    .route("/users", post(create_user))     // POST /users
    .route("/users/{id}", get(get_user))    // GET /users/123
```

Each handler is just an async function. Axum works out how to call it based on its parameter types. This is called the *extractor* pattern, and it's one of Axum's best features.

## What is async?

You might have noticed the word "async" popping up. In weeks 1 and 2, we mostly sidestepped it to keep things simple, but this week we need it.

When a web server handles a request, it often has to wait for things: a database query, a Docker API call, a network response. With synchronous code, the entire thread sits idle whilst waiting, which is incredibly wasteful, especially when you could be handling other requests! And having 

Async code lets you say "start this operation, and whilst it's waiting, go do other work". In Rust, this looks like:

```rust
async fn get_data() -> String {
    let result = some_slow_operation().await;  // yield whilst waiting
    result
}
```

The `.await` keyword is where the magic happens. It tells the runtime "I'm waiting for something, feel free to run other tasks in the meantime". The `async` keyword on the function marks it as returning a *future* rather than a value directly.

[Tokio](https://tokio.rs) is the async runtime we'll use. It handles scheduling all these futures across threads. The `#[tokio::main]` attribute on our `main` function sets up this runtime for us.

Don't worry if this feels abstract by the way! Once you actually start writing some async code, it should be pretty simple to get your head around. The key thing to remember is: `.await` = "wait for this without blocking everything else".

## Prerequisites

_To be clear, this is for the Docker container API we're making as an exemplar. You don't need it for your real project!_

You'll need Docker installed and running on your machine. If you don't have it, grab [Docker Desktop](https://www.docker.com/products/docker-desktop/) (macOS/Windows) or install it via your package manager on Linux.

You can check it's working by running:

```bash
docker ps
```

If that prints a (possibly empty) list of containers, you're good to go.

## Setting up the project

```bash
cargo new docker_pilot
cd docker_pilot
```

Now let's add our dependencies. `cargo add` makes this quick:

```bash
cargo add axum --features json
cargo add tokio --features full
cargo add serde --features derive
cargo add serde_json
cargo add sqlx --features "runtime-tokio,sqlite"
cargo add bollard
```

Let's talk about what each crate does:

| Crate | What it does |
|-------|-------------|
| **axum** | The web framework. The `json` feature enables JSON request/response support. |
| **tokio** | The async runtime! `full` enables all features (timers, I/O, etc.). |
| **serde** | Serialisation/deserialisation (parsing). `derive` lets us use `#[derive(Serialize, Deserialize)]`. |
| **serde_json** | JSON support for serde. Axum uses this under the hood. For our intents and purposes, it's the same thing as `JSON.parse` (but stricter) |
| **sqlx** | An async SQL toolkit. We're using SQLite as our database. |
| **bollard** | A Rust client for the Docker API. It talks to Docker's socket directly. |

## Building the API

### The full code

This is a bigger project than previous weeks, so let's start with the full code and then break it down. Replace everything in `src/main.rs` with:

```rust
use axum::{
    Json, Router,
    extract::{State, Path},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post},
};
use bollard::Docker;
use bollard::models::ContainerCreateBody;
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, ListContainersOptions, RemoveContainerOptionsBuilder,
    StopContainerOptionsBuilder,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;

struct AppState {
    docker: Docker,
    db: SqlitePool,
    api_key: String,
}

#[derive(Serialize)]
struct ContainerInfo {
    id: String,
    name: String,
    image: String,
    state: String,
}

#[derive(Deserialize)]
struct CreateContainerRequest {
    name: String,
    image: String,
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct AuditLog {
    id: i64,
    action: String,
    container_name: String,
    timestamp: String,
}

fn check_auth(headers: &HeaderMap, api_key: &str) -> Result<(), (StatusCode, Json<ApiResponse>)> {
    let provided = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if provided != api_key {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                message: "invalid or missing API key".to_string(),
            }),
        ));
    }

    Ok(())
}

async fn log_action(db: &SqlitePool, action: &str, container_name: &str) {
    let _ = sqlx::query("INSERT INTO audit_log (action, container_name) VALUES (?, ?)")
        .bind(action)
        .bind(container_name)
        .execute(db)
        .await;
}

async fn health() -> Json<ApiResponse> {
    Json(ApiResponse {
        message: "ok".to_string(),
    })
}

async fn list_containers(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<ContainerInfo>>, (StatusCode, Json<ApiResponse>)> {
    check_auth(&headers, &state.api_key)?;

    let options = ListContainersOptions {
        all: true,
        ..Default::default()
    };

    let containers = state
        .docker
        .list_containers(Some(options))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    message: format!("Docker error: {e}"),
                }),
            )
        })?;

    let result: Vec<ContainerInfo> = containers
        .into_iter()
        .map(|c| ContainerInfo {
            id: c.id.unwrap_or_default(),
            name: c
                .names
                .and_then(|n| n.first().cloned())
                .unwrap_or_default()
                .trim_start_matches('/')
                .to_string(),
            image: c.image.unwrap_or_default(),
            state: c
                .state
                .map(|s| s.to_string())
                .unwrap_or_default(),
        })
        .collect();

    Ok(Json(result))
}

async fn create_container(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateContainerRequest>,
) -> Result<(StatusCode, Json<ApiResponse>), (StatusCode, Json<ApiResponse>)> {
    check_auth(&headers, &state.api_key)?;

    let options = CreateContainerOptionsBuilder::new()
        .name(&body.name)
        .build();

    let config = ContainerCreateBody {
        image: Some(body.image),
        ..Default::default()
    };

    state
        .docker
        .create_container(Some(options), config)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    message: format!("failed to create container: {e}"),
                }),
            )
        })?;

    state
        .docker
        .start_container(&body.name, None)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    message: format!("created but failed to start: {e}"),
                }),
            )
        })?;

    log_action(&state.db, "create", &body.name).await;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            message: format!("container '{}' created and started", body.name),
        }),
    ))
}

async fn stop_container(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    check_auth(&headers, &state.api_key)?;

    let options = StopContainerOptionsBuilder::new()
        .t(10)
        .build();

    state
        .docker
        .stop_container(&name, Some(options))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    message: format!("failed to stop container: {e}"),
                }),
            )
        })?;

    log_action(&state.db, "stop", &name).await;

    Ok(Json(ApiResponse {
        message: format!("container '{name}' stopped"),
    }))
}

async fn remove_container(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    check_auth(&headers, &state.api_key)?;

    let options = RemoveContainerOptionsBuilder::new()
        .force(true)
        .build();

    state
        .docker
        .remove_container(&name, Some(options))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    message: format!("failed to remove container: {e}"),
                }),
            )
        })?;

    log_action(&state.db, "remove", &name).await;

    Ok(Json(ApiResponse {
        message: format!("container '{name}' removed"),
    }))
}

async fn get_logs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<AuditLog>>, (StatusCode, Json<ApiResponse>)> {
    check_auth(&headers, &state.api_key)?;

    let rows: Vec<AuditLog> = sqlx::query_as(
        "SELECT id, action, container_name, timestamp FROM audit_log ORDER BY id DESC LIMIT 50",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                message: format!("database error: {e}"),
            }),
        )
    })?;

    Ok(Json(rows))
}

#[tokio::main]
async fn main() {
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");

    let db = SqlitePool::connect("sqlite:docker_pilot.db?mode=rwc")
        .await
        .expect("failed to connect to database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            action TEXT NOT NULL,
            container_name TEXT NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&db)
    .await
    .expect("failed to create table");

    let docker = Docker::connect_with_local_defaults().expect("failed to connect to Docker");

    let state = Arc::new(AppState {
        docker,
        db,
        api_key,
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/containers", get(list_containers))
        .route("/containers", post(create_container))
        .route("/containers/{name}/stop", post(stop_container))
        .route("/containers/{name}", delete(remove_container))
        .route("/logs", get(get_logs))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind");

    println!("listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.expect("server error");
}
```

That's a lot. Let's go through it piece by piece.

### The data types

```rust
struct AppState {
    docker: Docker,
    db: SqlitePool,
    api_key: String,
}
```

`AppState` holds everything our handlers need: a Docker client, a database connection pool and the API key. This struct gets shared across all requests using `Arc` (Atomic Reference Counting), which lets multiple async tasks read from it safely.

```rust
#[derive(Serialize)]
struct ContainerInfo {
    id: String,
    name: String,
    image: String,
    state: String,
}
```

This is the shape of data we send back when listing containers. `#[derive(Serialize)]` tells serde to generate code that converts this struct into JSON automatically.

```rust
#[derive(Deserialize)]
struct CreateContainerRequest {
    name: String,
    image: String,
}
```

And this is what we *receive* when someone wants to create a container. `#[derive(Deserialize)]` does the reverse: it parses JSON into this struct.

```rust
#[derive(Serialize, sqlx::FromRow)]
struct AuditLog {
    id: i64,
    action: String,
    container_name: String,
    timestamp: String,
}
```

The `sqlx::FromRow` derive is new. It tells sqlx how to convert a database row into this struct. Each field name must match a column name in the query result.

### Authentication

```rust
fn check_auth(headers: &HeaderMap, api_key: &str) -> Result<(), (StatusCode, Json<ApiResponse>)> {
    let provided = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if provided != api_key {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                message: "invalid or missing API key".to_string(),
            }),
        ));
    }

    Ok(())
}
```

This is a simple helper that checks the `x-api-key` header against our stored key. It returns `Result<(), Error>`, which means it either succeeds with nothing (`()`) or fails with a status code and JSON error.

The `?` operator in our handlers (e.g. `check_auth(&headers, &state.api_key)?;`) will automatically return the error early if auth fails. We used `.expect()` for this in week 1, but `Result` with `?` is the proper way to handle errors in Rust.

### The handlers

Let's look at the simplest handler first:

```rust
async fn health() -> Json<ApiResponse> {
    Json(ApiResponse {
        message: "ok".to_string(),
    })
}
```

It takes no parameters and returns JSON. Axum sees the `Json<...>` return type and automatically sets the `Content-Type: application/json` header and serialises the struct.

Now the list endpoint:

```rust
async fn list_containers(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<ContainerInfo>>, (StatusCode, Json<ApiResponse>)> {
```

This is the *extractor* pattern in action. The function signature tells Axum what to inject:

- `State(state): State<Arc<AppState>>` extracts the shared app state
- `headers: HeaderMap` extracts the request headers

Axum matches these types and fills them in automatically. You don't need to parse anything by hand.

The return type `Result<Json<Vec<ContainerInfo>>, (StatusCode, Json<ApiResponse>)>` means "either return a JSON array of containers, or return an error with a status code and message". Axum knows how to convert both into HTTP responses.

The bollard calls are fairly straightforward:

```rust
let options = ListContainersOptions {
    all: true,
    ..Default::default()
};
let containers = state.docker.list_containers(Some(options)).await.map_err(|e| { ... })?;
```

The builder pattern (`...Builder::new().option(value).build()`) is common in Rust for constructing config objects. `all: true` means "show all containers, not just running ones". The `.await` is because this is an async operation (it talks to Docker over a socket), and `.map_err()` converts bollard's error type into our API error format.

### The create endpoint

```rust
async fn create_container(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateContainerRequest>,
) -> Result<(StatusCode, Json<ApiResponse>), (StatusCode, Json<ApiResponse>)> {
```

Notice the third parameter: `Json(body): Json<CreateContainerRequest>`. This tells Axum "parse the request body as JSON and deserialise it into a `CreateContainerRequest`". If the JSON is malformed or missing fields, Axum returns a 422 error automatically.

The handler creates a container and then immediately starts it:

```rust
let config = ContainerCreateBody {
    image: Some(body.image),
    ..Default::default()
};
```

`..Default::default()` fills in all the other fields with their defaults. This is a common Rust pattern when you only care about a few fields of a large struct.

The return type `(StatusCode, Json<ApiResponse>)` is a tuple. Axum returns this as an HTTP response with the given status code (201 Created) and JSON body.

### The database

```rust
async fn log_action(db: &SqlitePool, action: &str, container_name: &str) {
    let _ = sqlx::query("INSERT INTO audit_log (action, container_name) VALUES (?, ?)")
        .bind(action)
        .bind(container_name)
        .execute(db)
        .await;
}
```

sqlx uses `.bind()` to safely parameterise queries (no SQL injection!). The `?` placeholders get replaced with the bound values. The `let _ =` means we're deliberately ignoring the result. For audit logging, if it fails, we don't want to break the actual request.

For reading logs:

```rust
let rows: Vec<AuditLog> = sqlx::query_as(
    "SELECT id, action, container_name, timestamp FROM audit_log ORDER BY id DESC LIMIT 50",
)
.fetch_all(&state.db)
.await
```

`sqlx::query_as` runs a query and maps each row into the `AuditLog` struct (using the `FromRow` derive we added earlier). `.fetch_all()` collects all rows into a `Vec`.

### The main function

```rust
#[tokio::main]
async fn main() {
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");
```

`#[tokio::main]` is a macro that sets up the Tokio async runtime. Without it, you can't use `.await` in `main`. We read the API key from an environment variable so it's never hardcoded.

```rust
let db = SqlitePool::connect("sqlite:docker_pilot.db?mode=rwc")
    .await
    .expect("failed to connect to database");
```

This creates (or opens) a SQLite database file. The `?mode=rwc` means "read-write-create": it'll create the file if it doesn't exist.

```rust
let app = Router::new()
    .route("/health", get(health))
    .route("/containers", get(list_containers))
    .route("/containers", post(create_container))
    .route("/containers/{name}/stop", post(stop_container))
    .route("/containers/{name}", delete(remove_container))
    .route("/logs", get(get_logs))
    .with_state(state);
```

This is where all the routes come together. Notice that `/containers` has both a `get` and a `post` handler. `{name}` in the path is a dynamic parameter that gets extracted by `axum::extract::Path(name)` in the handler.

## Running it

First, set your API key and run the server:

```bash
API_KEY=my-secret-key cargo run
```

You should see:

```
listening on http://0.0.0.0:3000
```

Now in another terminal, try it out with `curl`:

```bash
# Health check (no auth needed)
curl http://localhost:3000/health

# List containers
curl -H "x-api-key: my-secret-key" http://localhost:3000/containers

# Create and start an nginx container
curl -X POST -H "x-api-key: my-secret-key" -H "Content-Type: application/json" \
  -d '{"name": "my-nginx", "image": "nginx:latest"}' \
  http://localhost:3000/containers

# Stop it
curl -X POST -H "x-api-key: my-secret-key" http://localhost:3000/containers/my-nginx/stop

# Remove it
curl -X DELETE -H "x-api-key: my-secret-key" http://localhost:3000/containers/my-nginx

# Check the audit log
curl -H "x-api-key: my-secret-key" http://localhost:3000/logs
```

## Making it publicly accessible

Your API needs a stable, publicly accessible URL. The easiest way to do this during development is with a _tunnelling service_, which'll basically give you a link to a proxy that sends the requests to your locally running dev server. Here are a few options:

- [Cloudflare Tunnel](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/) (free, stable URLs with a Cloudflare account)
- [ngrok](https://ngrok.com) (free tier available, gives you a public URL)
- [bore.pub](https://github.com/ekzhang/bore) (open source, no account needed, probably the simplest of the three?): `cargo install bore-cli && bore local 3000 --to bore.pub`

Pick whichever you prefer, point it at port 3000, and you'll have a public URL. In production, though, you'll want to deploy it to something like Railway or a server/VPS!

## What you learnt

Let's recap:

- Async Rust: using `async`/`.await` with Tokio to handle concurrent requests without blocking
- How to make routes in Axum, and what the extractor pattern is (declaring what your handler needs in its function signature, and letting Axum provide the deets to your function)
- sqlx: connecting to a database, creating tables, inserting rows and querying data
- How to check headers and return proper HTTP error codes
- The builder pattern: using `...Builder::new().option(value).build()` to construct configuration objects
- Safely sharing data across async tasks using `Arc`!

## Now build your own!

The Docker Pilot was a guided exercise. For your submission, you need to build a **different** web service. It must meet these requirements:

- **Publicly accessible** with a stable URL
  - You can deploy your project to something like Railway for this!
- **At least 3 GET endpoints** and **1 POST endpoint**
- A database (SQLite with sqlx is the easiest option, but PostgreSQL or some other DB works too)
- Something useful that you'd actually want to run :P

Here are some ideas:

- A URL shortener with click tracking and analytics? (think dub.co but actually good)
- A webhook relay that receives webhooks and forwards them, storing a log
- A Pastebin clone where you can create, retrieve and list code snippets (maybe use the `syntect` crate for syntax highlighting!)
- A server monitor that pings URLs on a schedule and exposes uptime data via an API (think Uptime Kuma)
- A CI status dashboard that aggregates build statuses from GitHub Actions across your repos
- Maybe build upon the Docker example and make it into something more fully-featured!
  - (Coolify is garbage 🙏)

And some things to avoid:

- A basic to-do list API (too simple)
- Anything with no database
- Something you'll submit and never think about again (please..)

## Extensions

- Add webpages to your server with something like [Askama.](https://lib.rs/askama)
  - Fun fact: [this very site](https://github.com/skyfallwastaken/resolution-rust) uses Askama + Axum!
  - You could even set up Tailwind for your site! (The repo link above may be useful)
- Add integration tests to your API with something like [axum-test!](https://lib.rs/crates/axum-test)
- Add logins to your site! You can handroll it with Axum + [JWTs](https://lib.rs/crates/jsonwebtoken) or session tokens.
- Add file uploads to your API with something like [rust-s3!](https://lib.rs/crates/rust-s3)
- Make custom error pages (for things like 404s)!

## Submitting your project

Make sure your API is accessible at a public URL and include that URL in your submission. Double check [Hackatime](https://hackatime.hackclub.com) to make sure your time's been tracked, then hit that button below!

See you next week :D 🦀

(Oh, and sorry for the delays. I promise the schedule will be less messed up after this week...)
