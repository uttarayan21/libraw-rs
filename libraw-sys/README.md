## libraw-sys

Raw bindings to the c api of [libRaw][libraw]

Usually points to the latest "early access" version of [LibRaw][libraw]

Currently uses mozjpeg-sys instead of libjpeg-sys for linking

Links to the following libraries:- 

- mozjpeg
- libjpaser
- zlib

Set `LIBRAW_DIR` to a locally cloned version of [LibRaw][libraw] to not clone from the public repo
```sh
export LIBRAW_DIR=/Users/fs0c131y/Projects/aftershoot/LibRaw
```

This repo is license under MIT but [LibRaw][libraw] itself uses either of the two
1. GNU LESSER GENERAL PUBLIC LICENSE version 2.1
2. COMMON DEVELOPMENT AND DISTRIBUTION LICENSE (CDDL) Version 1.0

for the public version

[libraw]: https://libraw.org
