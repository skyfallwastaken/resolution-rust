FROM oven/bun:1 AS css

WORKDIR /app
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

COPY static/styles/entrypoint.css static/styles/entrypoint.css
COPY templates templates
RUN bunx tailwindcss -i ./static/styles/entrypoint.css -o ./static/styles/tailwind.css

FROM rust:1-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY src src
COPY templates templates
COPY content content
COPY static static
COPY Cargo.toml Cargo.lock ./
COPY --from=css /app/static/styles/tailwind.css static/styles/tailwind.css
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=build /app/target/release/resolution_rust .
COPY content content
COPY templates templates
COPY static static
COPY --from=css /app/static/styles/tailwind.css static/styles/tailwind.css

EXPOSE 3000
CMD ["./resolution_rust"]
