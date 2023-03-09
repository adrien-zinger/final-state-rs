# Final State Compression

This is a tiny full Rust implementation of the FSE algorihm.

## Disclamer

This library contains only a set of tools to brew a data compression software. There is no specific data iterators, chunks creation from files, heuristics or any optimizations inside the algorithms. Because you may choose them by yourself.

This library shouldn't be used in production without a carefull check. This is principally a repository for studying the data compression with no further ambitions.

## How to use it

The repository contains multiple sub-methods of data compression, mainly about rANS and tANS technics.

This library provide all pieces and you can compose as you want your data compression. For example, a tANS requires:

1. An histogram, the count of each symbol in a source
2. A normalization on the histogram in order to have Sum(hist) == 2^table_log
3. A spread method required to build the tANS state machine
4. The encoding method that will build the state machine table and encode the source. It will return the stream produced by the compression and the state that should be used to decompress it.

```rust
const TABLE_LOG: usize = 10;
let mut hist = [0; 256];
// 1.
let _ = simple_count_u8_inplace(src, &mut hist);

// 2.
let normalized_hist = match normalization_with_fast_compensation(&hist, TABLE_LOG) {
    Ok(ret) => ret,
    Err(err) => match *err {
        NormError::RunLengthEncoding(_) => return rle(&src),
        _ => panic!(""),
        NormError::NormalizationError => panic!("norm error"),
    },
};

// 3.
let spread = fse_spread_unsorted(&normalized_hist, TABLE_LOG);

// 4.
let (stream, _) = encode_tans(&src, &normalized_hist, &spread, TABLE_LOG, state);
```

A rANS compression requires less functions. Bisacally it can looks like this:

```Rust
simple_count_u8_inplace(&src, &mut hist);
let normalized_hist = normalization_with_fast_compensation(&hist, TABLE_LOG).unwrap();
let (state, nb_bits_table, stream) = encode_rans(&normalized_hist, TABLE_LOG, &src);
```

## Why the library is builded like that

You can notice that ANS algorithm can have a big gap of performance by changing one of its components. The compression may be in the worst case bigger than the input if you change the `table_log` variable, the size of the chunks, the normalization, etc...

So a good approach to use correctly that library could be to add some heuristics of what can be used to produce the best output from a source.

## Contribute

Despite the small size of the library, any contribution would be nice. You can purpose first some variations of spreads algorithms or normalizations. If you want to do it properly, some tests that prove the validity / the resistance of that algorithm are welcome. If you like to do better, you could add a benchmark of it so we can compare with other methods.

You can fix bugs if you spot one, you can also write documentation (in english or french, both are great). You can provide samples, show your results. And you can reorder the repository as you want (rename things, suggest a better organization)

This repository isn't dedicated to ANS only, you can implement a totally different compression algorithm. Keep in mind that it should be usable in a composition like for tANS and rANS.

## License

Since the FSE algorithm is public and the FB implementation too, the current Rust interpretation should be under the MIT license.
