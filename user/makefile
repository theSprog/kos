APP_DIR := src
BIN_DIR := bin
PROG_DIR := prog
TARGET_DIR := target/riscv64gc-unknown-none-elf/release
APPS := $(wildcard $(APP_DIR)/*.rs)
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))
BINS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%.bin, $(APPS))

OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

build: clean
	python3 build.py

clean:
# cargo clean
	rm -f bin/*
	rm -f prog/*

