.PHONY: build run install uninstall restart logs clean

build:
	./scripts/build.sh

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
