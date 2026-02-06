"""
Test module that imports another module (test_helper).
This tests depth-first resolution: test_helper should be resolved before this file.
"""

import test_helper

def compute(x: int) -> int:
    """Compute using helper module."""
    doubled = test_helper.double(x)
    return doubled + 1
