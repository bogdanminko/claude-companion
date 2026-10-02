.PHONY: build run install uninstall restart logs preview clean

build:
	./scripts/build.sh

preview:
	mkdir -p build
	swiftc -O -o build/octo-preview Sources/Octo/OctoSprite.swift tools/preview/main.swift
	./build/octo-preview build/preview.png
	open build/preview.png

run: build
	open build/Octo.app

install:
	./scripts/install.sh

uninstall:
	./scripts/uninstall.sh

restart:
	launchctl kickstart -k gui/$$(id -u)/com.bogdanminko.octo

logs:
	tail -f /tmp/octo.err.log /tmp/octo.out.log

clean:
	rm -rf .build build
