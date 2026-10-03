.PHONY: build run install uninstall restart logs test preview icon demo clean

UNAME := $(shell uname -s)

build:
	./scripts/build.sh

run: build
ifeq ($(UNAME),Darwin)
	open "build/Claude Companion.app"
else
	./target/release/claude-companion
endif

install:
	./scripts/install.sh

uninstall:
	./scripts/uninstall.sh

restart:
ifeq ($(UNAME),Darwin)
	launchctl kickstart -k gui/$$(id -u)/com.bogdanminko.claude-companion
else
	pkill -f "^$$HOME/.local/bin/claude-companion( |$$)" || true
	setsid "$$HOME/.local/bin/claude-companion" --supervise >/dev/null 2>&1 < /dev/null &
endif

logs:
ifeq ($(UNAME),Darwin)
	tail -f /tmp/claude-companion.err.log /tmp/claude-companion.out.log
else
	tail -f "$${XDG_CACHE_HOME:-$$HOME/.cache}/claude-companion/companion.log"
endif

test:
	cargo test

preview:
	cargo run --example preview -- build/preview.png

demo:
	cargo run --release --example demo -- docs

# regenerates the macOS .icns (needs iconutil), the Windows .ico and the Linux .png
icon:
	cargo run --example icon -- build/icons
	cp build/icons/claude-companion.ico build/icons/claude-companion.png Resources/
ifeq ($(UNAME),Darwin)
	iconutil -c icns -o Resources/AppIcon.icns build/icons/AppIcon.iconset
endif

clean:
	cargo clean
	rm -rf build
