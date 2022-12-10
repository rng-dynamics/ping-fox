import os
import re
from sys import exit

report_dir = "target/coverage/html"

for line in open(os.path.join(report_dir, "badges/flat.svg"), "r").readlines():
    if re.search(r"^[ \t]*<title>coverage: [0-9]+%</title>$", line):
        print(re.findall(r"\b[0-9]+%", line)[0])
        exit(0)
exit(1)
