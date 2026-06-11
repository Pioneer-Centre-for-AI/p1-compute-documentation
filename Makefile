.PHONY: dev build preview clean

dev:
	bun run dev

build:
	bun run build

preview:
	bun run preview

clean:
	rm -rf build .svelte-kit
