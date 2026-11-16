"""Library reports; each prints its tables and writes its figures.

``python3 analysis/run_accuracy.py`` runs all of them. Every module is also
importable, so partial runs are one function call.
"""

from analysis.accuracy import tier_radar, tiers, token_stats  # noqa: F401
