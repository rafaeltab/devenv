.PHONY: build lint format

build:
	pnpm exec turbo run build

lint:
	pnpm exec turbo run lint

format:
	pnpm exec turbo run format

install:
	pnpm i

dev:
	pnpm exec turbo run dev
