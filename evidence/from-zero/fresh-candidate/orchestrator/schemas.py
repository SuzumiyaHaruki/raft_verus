"""JSON Schema is validation, never approval or proof."""
from __future__ import annotations
import json
from pathlib import Path


class ValidationError(ValueError):
    pass


def read_json(path: str | Path):
    try:
        return json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise ValidationError(f"{path}: {exc}") from exc


def validate_schema(value, schema_path: str | Path) -> None:
    try:
        import jsonschema
    except ImportError as exc:
        raise ValidationError("jsonschema is required; install the declared dependency") from exc
    schema = read_json(schema_path)
    try:
        jsonschema.Draft7Validator.check_schema(schema)
        jsonschema.Draft7Validator(schema).validate(value)
    except jsonschema.exceptions.ValidationError as exc:
        location = "/".join(str(p) for p in exc.absolute_path)
        raise ValidationError(f"{schema_path}:{location}: {exc.message}") from exc
    except jsonschema.exceptions.SchemaError as exc:
        raise ValidationError(f"invalid schema {schema_path}: {exc.message}") from exc


def wire_schema(name: str) -> Path:
    return Path(__file__).parent / "wire" / (name + ".schema.json")
