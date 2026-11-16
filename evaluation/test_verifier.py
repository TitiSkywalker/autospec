"""Run with `python -m unittest evaluation.test_verifier`."""

import unittest
from pathlib import Path

from evaluation.verifier import _declarations

BENCHMARK = Path(__file__).resolve().parent.parent / "lean_mode" / "LeanMode"


class DeclarationExtractionTests(unittest.TestCase):
    def test_top_level_names(self):
        src = "theorem a : True := by trivial\ntheorem b (n : Nat) : n = n := rfl\n"
        self.assertEqual(_declarations(src), ["a", "b"])

    def test_namespaced_names_are_qualified(self):
        # Comparator needs the name Lean actually assigns; a bare `bar` makes
        # lean4export panic with "Constant not found in environment".
        src = (
            "namespace graph_planarity\n"
            "theorem bar : True := by trivial\n"
            "end graph_planarity\n"
            "theorem baz : True := by trivial\n"
        )
        self.assertEqual(_declarations(src), ["graph_planarity.bar", "baz"])

    def test_nested_and_dotted_namespaces(self):
        src = (
            "namespace A.B\n"
            "theorem x : True := by trivial\n"
            "end A.B\n"
            "namespace C\n"
            "namespace D\n"
            "theorem y : True := by trivial\n"
            "end D\n"
            "end C\n"
        )
        self.assertEqual(_declarations(src), ["A.B.x", "C.D.y"])

    def test_section_and_mutual_scope_end_but_do_not_qualify(self):
        src = (
            "mutual\n  def a := 1\nend\n"
            "section Foo\n  theorem s : True := by trivial\nend Foo\n"
            "noncomputable section\n"
            "theorem t : True := by trivial\n"
            "end\n"
        )
        self.assertEqual(_declarations(src), ["s", "t"])

    def test_indented_declarations_are_found(self):
        src = "namespace Q\n  theorem inner : True := by trivial\nend Q\n"
        self.assertEqual(_declarations(src), ["Q.inner"])

    def test_comments_are_ignored(self):
        src = (
            "/-! `theorem for general graphs` in prose -/\n"
            "/-- A doc string mentioning theorem nope. -/\n"
            "theorem real : True := by trivial -- theorem also_nope\n"
        )
        self.assertEqual(_declarations(src), ["real"])

    def test_lemmas_are_not_targets(self):
        # `lemma` declarations in a challenge are the spec's own helper facts;
        # they are not proof obligations and Comparator is not asked about them.
        src = "lemma helper : True := by trivial\ntheorem main : True := by trivial\n"
        self.assertEqual(_declarations(src), ["main"])


class BenchmarkChallengeTests(unittest.TestCase):
    def test_every_case_yields_declarations(self):
        missed = [
            d.name
            for d in sorted(BENCHMARK.iterdir())
            if d.is_dir() and (d / "Spec.lean").is_file()
            and not _declarations((d / "Spec.lean").read_text())
        ]
        self.assertEqual(missed, [])

    def test_known_namespaced_case(self):
        self.assertEqual(
            _declarations((BENCHMARK / "graph_planarity" / "Spec.lean").read_text()),
            [
                "graph_planarity.is_planar_correct",
                "graph_planarity.wagner_characterisation",
                "graph_planarity.is_planar_iff_minor_free",
            ],
        )


if __name__ == "__main__":
    unittest.main()
