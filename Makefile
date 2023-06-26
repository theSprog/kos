OS_DIR = target/riscv64gc-unknown-none-elf/release
DEBUG_OS_DIR = target/riscv64gc-unknown-none-elf/debug
QEMU = qemu-system-riscv64
TOOLS = ~/riscv64-elf-tools/bin
KERNEL = kos
FS_IMG = ./ext2.img

QEMU_FLAGS = -machine virt -nographic -m 256M -smp 1

# 以下配置, 顺序不可随意调换, 空格不可随意添加, 例如不要随便在 "," 后加上空格 " "
BOOTLOADER = -bios ./boot/rustsbi-qemu.bin
QEMU_DEVICE1 = -device loader,file=./$(KERNEL),addr=0x80200000
QEMU_DRIVE = -drive file=$(FS_IMG),if=none,format=raw,id=x0
QEMU_DEVICE2 = -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
QEMU_BIOS = $(BOOTLOADER) $(QEMU_DEVICE1) $(QEMU_DRIVE) $(QEMU_DEVICE2)


# 允许的指令
.PHONY: run debugs debugc user_build clean

# 使用原始的进入退出 make
# 因为 makefile 会调用 python, 在处理路径时不协调
# 使用 && 连接命令, 只有当前一个命令执行成功才会执行后一个命令
user_build:
	@cd ./user && make build && cd ..

build: clean user_build 
	cargo b --release
	ln -sf $(OS_DIR)/$(KERNEL) ./$(KERNEL)

run: build
	$(QEMU) $(QEMU_FLAGS) $(QEMU_BIOS)

debug_build: clean user_build 
	cargo b
	ln -sf $(DEBUG_OS_DIR)/$(KERNEL) ./$(KERNEL)

# server
debugs: debug_build
	$(QEMU) $(QEMU_FLAGS) $(QEMU_BIOS) -s -S

# client
debugc:
	$(TOOLS)/riscv64-unknown-elf-gdb -ex 'file ./kos' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234' -ex 'b *0x80200000' -ex 'c'

clean:
	cd ./user && make clean && cd ..
	rm -f kos
	cargo clean
