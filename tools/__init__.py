"""Tool package — importing triggers all tool registrations."""

from tools.filesystem import list_directory, read_file, search_code
from tools.cargo_tools import read_cargo_toml, list_tests, run_cargo_test
from tools.spec_tools import write_spec
from tools.doc_json import generate_doc_json, read_rustdoc

__all__ = [
    "list_directory",
    "read_file",
    "search_code",
    "read_cargo_toml",
    "list_tests",
    "run_cargo_test",
    "write_spec",
    "generate_doc_json",
    "read_rustdoc",
]
