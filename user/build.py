import os

base_address = 0x80400000
step = 0x20000
linker = 'src/linker.ld'

app_id = 0
apps = os.listdir('./src')
apps.remove('linker.ld')
apps.sort()
for app in apps:
    app = app[:app.rfind('.')]
    print('processing app "%s"' % app)
    with open(linker, 'r+') as f:
        start_addr = base_address + step * app_id
        end_addr = base_address + step * (app_id+1)

        ori_content = f.read()
        new_content = ori_content.replace(hex(base_address), hex(start_addr))

        f.seek(0)
        f.truncate()  # 清空文件内容
        f.write(new_content)
        f.flush()   # 立即写入，后面要用

        os.system(f'cargo build --bin {app} --release')
        os.system(f'cp ./target/riscv64gc-unknown-none-elf/release/{app} ./prog')
        os.system(f'rust-objcopy --binary-architecture=riscv64 ./prog/{app} --strip-all -O binary ./bin/{app}.bin')
        print('[build.py] application "%s" with address [%s..%s)' % (app, hex(start_addr), hex(end_addr)))

        f.seek(0)
        f.truncate()  # 清空文件内容
        f.write(ori_content)
    app_id = app_id + 1