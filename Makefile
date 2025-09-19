.PHONY: audit bootstrap build clean release

OS := $(shell uname -s 2>/dev/null | tr '[:upper:]' '[:lower:]')
BOOTSTRAP :=

ifeq ($(OS),linux)
BOOTSTRAP := ./scripts/linux/bootstrap.sh
else ifeq ($(OS),darwin)
BOOTSTRAP := ./scripts/macos/bootstrap.sh
else
BOOTSTRAP := pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/windows/bootstrap.ps1
endif

audit:
	$(BOOTSTRAP) --audit-only

bootstrap:
	$(BOOTSTRAP) --bootstrap

build:
	$(BOOTSTRAP) --build

release:
	$(BOOTSTRAP)

clean:
	rm -rf build gui/target
