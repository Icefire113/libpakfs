# PakFiles

## File Structure

The structure of a pak file is defined in [`pakfile.rs`](../pakfs/src/pakfile/pakfile.rs).

### Remarks

All strings in the file, particularly the file paths in the manifest, must be UTF-8 encoded

## Overview

Pak files are not intended to be edited after creation, that is, they should not be used to store data that is ment to be modified.
And after creation they should not be modified in any way, any modifications to the contents of the file require a new pak file to be created.
