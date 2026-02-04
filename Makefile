.PHONY: build release debug install clean

PREFIX ?= /usr/local

build: release

release:
	swift build -c release

clean:
	swift package clean
	rm -rf .build
