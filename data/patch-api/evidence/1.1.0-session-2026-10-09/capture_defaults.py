"""Capture own-base default extractor bytes/errors once; never overwrite."""

import importlib.util
import json
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent


def load(name):
    spec = importlib.util.spec_from_file_location(
        name, EVIDENCE / "historical-tools" / f"{name}.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    generator = load("gen_patch_wikitext_register")
    extractor = load("extract_patch_non_inventory")
    output = EVIDENCE / "default-extract.txt"
    assert not output.exists(), "refuse default extract overwrite"
    output.write_bytes(
        extractor.extract_text((EVIDENCE / "source.wikitext").read_text()).encode()
    )
    errors = []
    for tool, function, argument in [
        (generator, "parse_symbol", "not a reference"),
        (extractor, "extract_text", None),
    ]:
        try:
            getattr(tool, function)(argument)
        except Exception as error:
            errors.append(
                {
                    "function": function,
                    "argument": argument,
                    "type": type(error).__name__,
                    "message": str(error),
                }
            )
        else:
            raise AssertionError("historical error unexpectedly accepted")
    destination = EVIDENCE / "default-errors.json"
    assert not destination.exists(), "refuse error capture overwrite"
    destination.write_text(json.dumps(errors, indent=2) + "\n")
    print(
        json.dumps(
            {"extract_bytes": len(output.read_bytes()), "errors": errors}, indent=2
        )
    )


if __name__ == "__main__":
    main()
