# Downsample Oxide

## Largest Triangle Three Buckets implementation based off https://github.com/jeromefroe/lttb-rs

[![docs.rs](https://docs.rs/downsample-oxide/badge.svg)](https://docs.rs/downsample-oxide/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/billy-sheppard/downsample-oxide/master/LICENSE)
[![crates.io](https://img.shields.io/crates/v/downsample-oxide.svg)](https://crates.io/crates/downsample-oxide/)

### [Documentation](https://docs.rs/downsample-oxide/)
___

From [Jerome's Readme](https://github.com/jeromefroe/lttb-rs/blob/master/README.md):
> An implementation of the largest triangle three buckets (lttb)
algorithm for time series downsampling as described in
[Downsampling Time Series for Visual Representation](https://skemman.is/bitstream/1946/15343/3/SS_MSthesis.pdf).
This is a Rust port of
[the original Javascript implementation](https://github.com/sveinn-steinarsson/flot-downsample).

This implementation is heavily based and inspired from his original. Some QOL updates and datatype differences, such as using `rust_decimal` and offering output types to work generically or with `chrono` or `time` (both behind features).
___

## Example
``` rust
use downsample_oxide::*;

fn main() {
    let dps = Vec::from([
        DataPoint::new(first_day_of_month(1), Decimal::from(10)),
        DataPoint::new(first_day_of_month(2), Decimal::from(12)),
        DataPoint::new(first_day_of_month(3), Decimal::from(8)),
        DataPoint::new(first_day_of_month(4), Decimal::from(10)),
        DataPoint::new(first_day_of_month(5), Decimal::from(12)),
    ]);

    let output = dps.downsample(3)
}
```