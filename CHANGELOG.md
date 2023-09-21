# Changelog

This file logs the versions of quantr.

## 0.1.2 - CZ and Swap gate confusion

Fixes:
- There was confusion in thinking that CZ and Swap gates were the same
  (probably due to the similar notation in circuit diagrams). This has
  now been corrected in the documentation of the code and quick start
  guide.
- The swap gate was incorrectly defined, there was a negative sign in
  the mapping of the |11> state. Now, the swap gate has the correct
  definition.

Additions:
- An extra unit tests that now verifies the mappings of the swap and CZ
  gates, in addition to acknowledging that they're different.

## 0.1.1 - Reviewed README.md and QUICK_START

Fixes:
- Reviewed both documents mentioned in title, correcting spelling errors
  and sentences that didn't flow well.
- Corrected other spelling errors in other documents.

## 0.1.0 - Initial commit

The initial commit of quantr! 

See the 
[quick start guide](QUICK_START.md) to get started with quantr.

