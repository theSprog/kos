APP_DIR := src
BIN_DIR := bin
PROG_DIR := prog
TARGET_DIR := target/riscv64gc-unknown-none-elf/release
APPS := $(wildcard $(APP_DIR)/*.rs)
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))
BINS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%.bin, $(APPS))

OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

elf:
	cargo build --release


binary: elf
# 将所有的文件先拷贝到 prog 文件夹下
	$(foreach elf, $(ELFS), cp $(elf) $(patsubst $(TARGET_DIR)/%, $(PROG_DIR)/%, $(elf));)
# 将所有的 elf 文件剥离为 bin 文件
	$(foreach elf, $(ELFS), $(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(BIN_DIR)/%.bin, $(elf));)


build: binary

clean:
	cargo clean
	rm -rf bin/*
	rm -rf prog/*

