import json
import sys
import pathlib
with open(pathlib.Path(__file__).parent / "info.json", "r") as f:
    print(json.load(f)[sys.argv[1]]) # pyright: ignore[reportAny]

