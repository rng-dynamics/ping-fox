import glob
import os
import subprocess
import sys
from pathlib import Path

destination_dir = "target/coverage/html"
output_type = "html"


def create_instrumentation_env():
    env = os.environ.copy()
    env["CARGO_INCREMENTAL"] = "0"
    env["RUSTFLAGS"] = (
        " -Zprofile"
        " -Ccodegen-units=1"
        " -Copt-level=0"
        " -Clink-dead-code"
        " -Coverflow-checks=off"
        " -Zpanic_abort_tests"
        " -Cpanic=abort"
        " -Cinstrument-coverage"
    )
    env["LLVM_PROFILE_FILE"] = "cargo-test-%p-%m.profraw"
    return env


def cargo_test(env):
    print("=== run coverage ===")
    subprocess.run(
        ["cargo", "+nightly", "test", "--lib"],
        env=env,
        check=True,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    print("ok.")


def create_report():
    print("=== generate report ===")
    Path(destination_dir).mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            "grcov",
            ".",
            "--binary-path",
            "./target/debug/deps",
            "-s",
            ".",
            "-t",
            output_type,
            "--branch",
            "--ignore-not-existing",
            "--ignore",
            "/*",
            "-o",
            destination_dir,
        ],
        check=True,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    print("ok.")


def cleanup():
    print("=== cleanup ===")
    for file in glob.iglob("./**/*.profraw", recursive=True):
        os.remove(file)
    print("ok.")


if __name__ == "__main__":
    env = create_instrumentation_env()
    cargo_test(env)
    create_report()
    cleanup()
    print(f"location of coverage report: {destination_dir}")
    exit(0)
