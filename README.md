# rusty-sticker is a minimal sticker written in Rust.

Copyright 2018- Tatsuhiro Aoshima (hiro4bbh@gmail.com).

# Introduction
rusty-sticker is a minimal [sticker](https://github.com/hiro4bbh/sticker) written in Rust.
Currently, this supports only `LabelNearest`, and there is no unit test (we verify only the result of the real datasets).
We are not experts of Rust, so, if you have any idea for smart coding, please tell us your advice!

The reason why we develop rusty-sticker is:

- Programs written in golang seems to be slower, due to GC or lack of some sophisticated optimizations (maybe. especially, we hate the unoptimized unnecessary panic index checks).
- Programs written in Rust seems to be faster, thanks to the smart (?) memory management and the sophisticated optimizations provided by LLVM.

However, in Rust ecosystems, there are some cons against golang (the followings can be done easily in a single environment):

- Hard to compile the binaries for all platforms on a single environment
- Lack of built-in profilers

Contrary to first expectation, writing sticker in Rust is easier and succinct than doing in golang thanks to powerful syntax and type checks.

# Results
We evaluate the performances against [sticker](https://github.com/hiro4bbh/sticker) in the same settings of there.

|Dataset Name|sticker (ms/entry)|rusty-sticker (ms/entry)|
|:---|---:|---:|
|AmazonCat-13K|15.8|17.0|
|Wiki10-31K|1.22|1.17|
|Delicious-200K|5.50|5.15|
|WikiLSHTC-325K|16.3|17.9|
|Amazon-670K|5.16|5.15|
|Amazon-3M|16.5|18.2|

We think that it was only to show our poor understanding for Rust.
We are currently working on improving the code.
