import os
import subprocess

addr2line_path = "~/riscv64-elf-tools/bin/riscv64-unknown-elf-addr2line"
kernel = "kos"


def run_command(command):
    try:
        output = subprocess.check_output(command, shell=True, stderr=subprocess.STDOUT, encoding="utf-8")
        return output.strip()
    except subprocess.CalledProcessError as e:
        print(f"Command execution failed with return code {e.returncode}")
        return None

def print_color_path(paths):
    paths.reverse()
    for line in paths:
        parts = line.split('/')
        colored_segments = []

        for i, part in enumerate(parts):
            if i >= len(parts) - 2:
                # 最后两个字段用红色
                colored_segments.append("\033[31m{}\033[0m".format(part))
            else:
                # 其他字段用普通颜色
                colored_segments.append(part)

        colored_line = '/'.join(colored_segments)
        print(colored_line)

if __name__ == "__main__":
    print("type backtrace address, 'ok' to stop typing")
    addrs = []
    i = input()
    while i != 'ok':
        addrs.append(i)
        i = input()

    addrs_list = " ".join(addrs)

    output = run_command(f'{addr2line_path} --exe={kernel} {addrs_list}')
    print_color_path(output.split("\n"))
    