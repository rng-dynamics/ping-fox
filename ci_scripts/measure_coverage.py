import glob
import os
import subprocess
import sys
from pathlib import Path

html_output_dir = "target/debug/coverage/"


def create_instrumentation_env():
    env = os.environ.copy()
    env["CARGO_INCREMENTAL"] = "0"
    env["RUSTFLAGS"] = (
        # " -Zprofile"
        " -Cinstrument-coverage"
        " -Ccodegen-units=1"
        " -Copt-level=0"
        " -Clink-dead-code"
        " -Coverflow-checks=off"
        " -Zpanic_abort_tests"
        " -Cpanic=abort"
    )
    env["LLVM_PROFILE_FILE"] = "cargo-test-%p-%m.profraw"
    return env


def run_coverage(env):
    print("=== run coverage ===")
    subprocess.run(
        ["cargo", "+nightly", "test", "--lib"],
        env=env,
        check=True,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    print("ok.")


def generate_html_report():
    print("=== generate html report ===")
    Path(html_output_dir).mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            "grcov",
            ".",
            "-s",
            ".",
            "--binary-path",
            "./target/debug/",
            "-t",
            "html",
            "--branch",
            "--ignore-not-existing",
            "--ignore",
            "/*",
            "-o",
            html_output_dir,
        ],
        check=True,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    print("ok.")


def generate_report():
    print("=== generate report ===")
    Path(html_output_dir).mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            "grcov",
            ".",
            "-s",
            ".",
            "--binary-path",
            "./target/debug/",
            "-t",
            "lcov",
            "--branch",
            "--ignore-not-existing",
            "--ignore",
            "/*",
            "-o",
            "./target/debug/lcov.info",
        ],
        check=True,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    subprocess.run(
        [
            "genhtml",
            "-o",
            html_output_dir,
            "--show-details",
            "--highlight",
            "--ignore-errors",
            "source",
            "--legend",
            "./target/debug/lcov.info"
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
    run_coverage(env)
    generate_report()
    cleanup()
    print(f"location of html code coverage report: {html_output_dir}")
    exit(0)
