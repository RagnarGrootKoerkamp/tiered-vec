# Tiered Vec

A quick & dirty implementation of 2-level "tiered vector"
by Goodrich and Kloss II ([doi](http://dx.doi.org/10.1007/3-540-48447-7_21)) and
3-level "fast dynamic array" variant by Bille et al. ([doi](https://doi.org/10.4230/LIPIcs.ESA.2017.16)).

Both use the implicit heap-like layout.

Currently only supports `insert` and `get`. Resizing is not supported.

Tests and the 3-level index were written by copilot and refined by me.
