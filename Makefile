.PHONY: build run install uninstall restart logs preview icon demo clean

build:
	./scripts/build.sh

preview:
	mkdir -p build
	swiftc -O -o build/sprite-preview Sources/ClaudeCompanion/PixelSprite.swift tools/preview/main.swift
	./build/sprite-preview build/preview.png
	open build/preview.png

icon:
	mkdir -p build
	swiftc -O -o build/make-icon Sources/ClaudeCompanion/PixelSprite.swift tools/icon/main.swift
	./build/make-icon build/AppIcon.iconset
	iconutil -c icns -o Resources/AppIcon.icns build/AppIcon.iconset

demo:
	mkdir -p build
	swiftc -O -o build/make-demo $(filter-out Sources/ClaudeCompanion/main.swift,$(wildcard Sources/ClaudeCompanion/*.swift)) tools/demo/main.swift
	./build/make-demo docs

run: build
	open "build/Claude Companion.app"

install:
	./scripts/install.sh

uninstall:
	./scripts/uninstall.sh

restart:
	launchctl kickstart -k gui/$$(id -u)/com.bogdanminko.claude-companion

logs:
	tail -f /tmp/claude-companion.err.log /tmp/claude-companion.out.log

clean:
	rm -rf .build build
