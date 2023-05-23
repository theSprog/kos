OS_DIR = target/riscv64gc-unknown-none-elf/release
DEBUG_OS_DIR = target/riscv64gc-unknown-none-elf/debug
QEMU_FLAGS = -machine virt -nographic -m 256M -smp 4
QEMU_BIOS = -bios ./boot/rustsbi-qemu.bin -device loader,file=./kos,addr=0x80200000
QEMU = qemu-system-riscv64
TOOLS = ~/riscv64-elf-tools/bin
KERNEL = kos

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
	cd ./user_lib && cargo clean && cd ..
	cd ./component && cargo clean && cd ..
	cd ./logger && cargo clean && cd ..
	cd ./qemu_config && cargo clean && cd ..
	cd ./sys_interface && cargo clean && cd ..
	rm -f kos
	cargo clean
