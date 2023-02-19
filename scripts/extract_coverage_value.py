import os
import re
from sys import exit

report_dir = "target/debug/coverage/"

with open(os.path.join(report_dir, "index.html"), "r") as file:
    lines = file.readlines()
    matching_lines = [
        line for line in lines if re.search(r"^.*>[0-9]+\.?[0-9]? %</td>.*$", line)
    ]
    if not matching_lines:
        exit(1)
    print(re.findall(r"\b[0-9]+\.?[0-9]? %", matching_lines[0])[0])
    exit(0)

exit(1)
