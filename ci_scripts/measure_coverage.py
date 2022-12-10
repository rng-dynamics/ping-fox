import glob
import os
import subprocess
from pathlib import Path

destination_dir = "target/coverage/html"
output_type = "html"


def generate_env():
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
    p = subprocess.run(
        ["cargo", "+nightly", "test", "--lib"],
        env=env,
        # TODO: cleanup
        # check=True,
        capture_output=True,
        text=True
    )
    print(p.stdout)
    print(p.stderr)
    print("ok.")


def generate_report():
    print("=== generate report ===")
    Path(destination_dir).mkdir(parents=True, exist_ok=True)
    p = subprocess.run(
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
        capture_output=True,
        text=True,
    )
    print(p.stdout)
    print("ok.")


def cleanup():
    print("=== cleanup ===")
    for file in glob.iglob("./**/*.profraw", recursive=True):
        os.remove(file)
    print("ok.")


if __name__ == "__main__":
    env = generate_env()
    cargo_test(env)
    generate_report()
    cleanup()
    print(f"location of coverage report: {destination_dir}")
