# rusty-sticker is a minimal sticker written in Rust.

Copyright 2018- Tatsuhiro Aoshima (hiro4bbh@gmail.com).

# Introduction
rusty-sticker is a minimal [sticker](https://github.com/hiro4bbh/sticker) written in Rust.
Currently, this supports only `LabelNearest`.
We are not experts of Rust (you can follow up the commits for our evolution), so, if you have any idea for smart coding, please tell us your advice!

The reason why we develop rusty-sticker is:

- Programs written in golang seems to be slower, due to GC or lack of some sophisticated optimizations (maybe. especially, we hate the unoptimized unnecessary panic index checks or the un-inlined function calls).
- Programs written in Rust seems to be faster, thanks to the smart memory management and the sophisticated optimizations provided by LLVM.

However, in Rust ecosystems, there are some cons against golang (the followings can be done easily in a single environment):

- Hard to compile the binaries for all platforms on a single environment (rustc needs the system linkers and libraries)
- Lack of built-in profilers
- Slow compilation (negligible, the optimized codes are amazing!!)

Contrary to our first expectation, writing sticker in Rust is easier and succinct than doing in golang thanks to powerful syntax and type checks.
We will prepare rustdoc, and consider unit tests in rustdoc (currently, we verify only the results on the real datasets as reported in the following sections).

# Results
We evaluate the performances against [sticker](https://github.com/hiro4bbh/sticker) in the same settings of there.

|Dataset Name|sticker (ms/entry)|rusty-sticker (ms/entry)|Delta|
|:---|---:|---:|---:|
|AmazonCat-13K|15.1|12.6|-16.6%|
|Wiki10-31K|1.14|1.00|-12.3%|
|Delicious-200K|4.88|4.43|-9.22%|
|WikiLSHTC-325K|14.1|12.4|-12.1%|
|Amazon-670K|4.19|3.66|-12.6%|
|Amazon-3M|15.5|12.9|-16.8%|

Profiling the code on MacOS with Xcode Instruments (Visual Studio is useless because there is no support of profiling inline functions) shows that:
- The highly-optimized code computing the dot-products dominates the computation.
- In larger datasets, using the same context improves the performance, because the memory clears of the accumulator is no longer needed.

Thus, we can improve the implementation as follows:
- For avoiding the out-of-bound checks, use unsafe code `get_unchecked_mut`
- Use `HashMap` manipulation with `FNVHasher` (improving little)

Currently, the performance difference by the optimization efforts in Rust is not so small.

## Other Miscellaneous Results
The following comparisons are for `sticker`'s `LabelNear`.

### Sorting the K-Largest Entries with a Heap
We compare the sorting of the K-largest entries with a heap against `sticker`'s `SortLargestCountsWithHeap`.
The result is the following running on MacBook Early 2016:

```
~/go/src/github.com/hiro4bbh/sticker$ go test -v . -bench .
BenchmarkKeyCounts32SortLargestCountsWithHeap-4           	     200	   5851999 ns/op
~/src/rusty-sticker$ ./target/release/rusty-sticker-benchmarks
(sort 150 largests with heap in 65536 buckets with 50% filled) * 10000 times: finished in 31.747s (3.174ms/try)
```

The code written in Rust is about 1.8x faster.

### Hash Insertion
We compare the hash insertion performance against `sticker`'s `KeyCountMap32`.
The result is the following running on MacBook Early 2016:

```
~/go/src/github.com/hiro4bbh/sticker$ go test -v . -bench .
BenchmarkKeyCountMap32-4                                  	     500	   2616577 ns/op
~/src/rusty-sticker$ ./target/release/rusty-sticker-benchmarks
(fill 50% of 65536 buckets) * 10000 times: finished in 17.778s (1.777ms/try)
```

The code written in Rust is about 1.5x faster.
