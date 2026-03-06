dev:
    cargo watch -x run -w templates -w src -w content -w static

tailwind-watch:
	bun tailwindcss -i ./static/styles/entrypoint.css -o ./static/styles/tailwind.css --watch

build:
    bunx tailwindcss -i ./static/styles/entrypoint.css -o ./static/styles/tailwind.css
    cargo build --release
