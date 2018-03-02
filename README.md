# rusty-sticker is a minimal sticker written in Rust.

Copyright 2018- Tatsuhiro Aoshima (hiro4bbh@gmail.com).

# Introduction
rusty-sticker is a minimal [sticker](https://github.com/hiro4bbh/sticker) written in Rust.
Currently, this supports only `LabelNearest`, and there is no unit test (we verify only the result of the real datasets).
We are not experts of Rust (you can follow up the commits for our evolution), so, if you have any idea for smart coding, please tell us your advice!

The reason why we develop rusty-sticker is:

- Programs written in golang seems to be slower, due to GC or lack of some sophisticated optimizations (maybe. especially, we hate the unoptimized unnecessary panic index checks or the un-inlined function calls).
- Programs written in Rust seems to be faster, thanks to the smart memory management and the sophisticated optimizations provided by LLVM.

However, in Rust ecosystems, there are some cons against golang (the followings can be done easily in a single environment):

- Hard to compile the binaries for all platforms on a single environment
- Lack of built-in profilers
- Slow compilation (however, the optimized codes are amazing!!)

Contrary to first expectation, writing sticker in Rust is easier and succinct than doing in golang thanks to powerful syntax and type checks.

# Results
We evaluate the performances against [sticker](https://github.com/hiro4bbh/sticker) in the same settings of there.

|Dataset Name|sticker (ms/entry)|rusty-sticker (ms/entry)|Delta|
|:---|---:|---:|---:|
|AmazonCat-13K|15.8|15.7|+0.6%|
|Wiki10-31K|1.22|1.02|+16.3%|
|Delicious-200K|5.50|4.79|+12.9%|
|WikiLSHTC-325K|16.3|16.5|-1.2%|
|Amazon-670K|5.16|4.71|+8.7%|
|Amazon-3M|16.5|16.8|-1.8%|

Profiling the code on MacOS with Xcode Instruments shows that the highly-optimized code computing the dot-products dominates the computation.
Thus, we can speed-up it with unsafe code for avoiding the out-of-bound checks.
Furthermore, we can speed-up `HashMap` manipulation with `FNVHasher` (improving little).
However, the naive golang implementation is not so bad, and the performance difference by the optimization efforts in Rust is not large.

Rust is simple and beautiful in design, but we think that there is no definite advantage against golang.
