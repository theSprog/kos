import os

linker = 'kernel/linker.ld'

app_id = 0
apps = os.listdir('./src')
apps.sort()

for app in apps:
    app = app[:app.rfind('.')]
    print('processing app "%s"' % app)
    os.system(f'cargo build --bin {app} --release')
    os.system(f'cp ./target/riscv64gc-unknown-none-elf/release/{app} ./prog')
    os.system(f'rust-objcopy --binary-architecture=riscv64 ./prog/{app} --strip-all -O binary ./bin/{app}.bin')
    