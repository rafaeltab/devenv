.PHONY: build lint format ci

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

activate:
	pnpm exec turbo run activate

ci:
	pnpm exec turbo run build lint test
