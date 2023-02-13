# Final State Compression

This is a tiny full Rust implementation of the FSE algorihm.

This library does not content any foreign tools like histogram computation,
data iterators, chunk creation from files or euristic optimization.

This library shouldn't be used in production without a carefull check from
yourself. This is principally a repository for studying the data compression
with some arithmetic.

However, these public functions seems to work correctly:

```rust
/// Symbol index contains the position for each symbol in the histogram.
/// You should care about your alphabet outside the function.
pub fn encode(
    hist: &mut [usize],
    symbol_index: &HashMap<u16, usize>,
    table_log: usize, // R
    src: &[u16],
) -> (usize, Vec<u32>, Vec<u8>)

/// Hist is a 255 sized slice containing the symbol's index itself
pub fn encode_u8(
    hist: &mut [usize],
    table_log: usize, // R
    src: &[u8],
) -> (usize, Vec<u32>, Vec<u8>);

pub fn decode(
    mut state: usize,
    mut bits: Vec<u32>,
    str: Vec<u8>,
    normalized_counter: &[usize],
    symbols: &[u16],
    table_log: usize,
) -> Vec<u16>;

pub fn decode_u8(
    mut state: usize,
    mut bits: Vec<u32>,
    str: Vec<u8>,
    normalized_counter: &[usize],
    table_log: usize,
) -> Vec<u8>;
```

## License

Since the FSE algorithm is public and the FB implementation too, the current
Rust interpretation should be under the MIT license.
