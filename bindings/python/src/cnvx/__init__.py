# pyrefly: ignore [missing-import]
from .cnvx import (
    __version__,
)

__all__ = [
    "__version__",
]

# Prenvent the cnvx submodule namespace leaking
del cnvx  # noqa: F821
