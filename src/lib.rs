// MIT License

// Copyright (c) 2017 Jerome Froelich

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// ------------------------------------------------------ \\
// ADAPTED FROM https://github.com/jeromefroe/lttb-rs
// ------------------------------------------------------ //

use rust_decimal::Decimal;
//
#[cfg(all(not(feature = "time"), feature = "chrono"))]
use chrono_crate as chrono;
#[cfg(all(feature = "time", not(feature = "chrono")))]
use time_crate as time;

/// DataPoint
///
/// Struct used to represent a single datapoint in a time series.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DataPoint {
    x: Decimal,
    y: Decimal,
}

impl DataPoint {
    pub fn new(x: impl Into<std::time::SystemTime>, y: Decimal) -> Self {
        DataPoint {
            // convert from anything that impls SystemTime to UNIX epoch as seconds, then into Decimal for arithmetic reasons
            x: x.into()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .into(),
            y,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DataOutput {
    #[cfg(all(not(feature = "time"), feature = "chrono"))]
    pub x: chrono::DateTime<chrono::Utc>,
    #[cfg(all(feature = "time", not(feature = "chrono")))]
    pub x: time::OffsetDateTime,
    #[cfg(all(not(feature = "time"), not(feature = "chrono")))]
    pub x: std::time::SystemTime,
    pub y: Decimal,
}
impl From<DataPoint> for DataOutput {
    fn from(value: DataPoint) -> Self {
        #[allow(unused_variables)]
        let systemtime = std::time::UNIX_EPOCH + std::time::Duration::from_secs(value.x.try_into().unwrap());
        Self {
            #[cfg(all(not(feature = "time"), feature = "chrono"))]
            x: systemtime.try_into().unwrap(),
            #[cfg(all(feature = "time", not(feature = "chrono")))]
            x: systemtime.try_into().unwrap(),
            #[cfg(all(not(feature = "time"), not(feature = "chrono")))]
            x: systemtime,
            y: value.y,
        }
    }
}

pub trait Lttb {
    fn downsample(self, threshold: usize) -> Vec<DataOutput>;
}
impl Lttb for Vec<DataPoint> {
    fn downsample(self, threshold: usize) -> Vec<DataOutput> {
        if threshold >= self.len() || threshold == 0 {
            // Nothing to do.
            return self.into_iter().map(Into::into).collect();
        }

        let mut sampled = Vec::with_capacity(threshold);

        // Bucket size. Leave room for start and end data points.
        let every = Decimal::from(self.len() - 2) / (Decimal::from(threshold - 2));

        // Initially a is the first point in the triangle.
        let mut a = 0;

        // Always add the first point.
        sampled.push(self[a]);

        for i in 0..threshold - 2 {
            // Calculate point average for next bucket (containing c).
            let mut avg_x = Decimal::from(0);
            let mut avg_y = Decimal::from(0);

            let avg_range_start = (i + 1) * (usize::try_from(every).unwrap()) + 1;

            let mut end = ((i + 2) * usize::try_from(every).unwrap()) + 1;
            if end >= self.len() {
                end = self.len();
            }
            let avg_range_end = end;

            let avg_range_length = avg_range_end - avg_range_start;

            for i in 0..(avg_range_end - avg_range_start) {
                let idx = avg_range_start + i;
                avg_x += self[idx].x;
                avg_y += self[idx].y;
            }
            avg_x /= Decimal::from(avg_range_length);
            avg_y /= Decimal::from(avg_range_length);

            // Get the range for this bucket.
            let range_offs: usize = ((i) * usize::try_from(every).unwrap()) + 1;
            let range_to: usize = ((i + 1) * usize::try_from(every).unwrap()) + 1;

            // Point a.
            let point_a_x = self[a].x;
            let point_a_y = self[a].y;

            let mut max_area = Decimal::from(-1);
            let mut next_a = range_offs;
            for i in 0..(range_to - range_offs) {
                let idx = range_offs + i;

                // Calculate triangle area over three buckets.
                let area = ((point_a_x - avg_x) * (self[idx].y - point_a_y)
                    - (point_a_x - self[idx].x) * (avg_y - point_a_y))
                    .abs()
                    * Decimal::try_from(0.5).unwrap();
                if area > max_area {
                    max_area = area;
                    next_a = idx; // Next a is this b.
                }
            }

            sampled.push(self[next_a]); // Pick this point from the bucket.
            a = next_a; // This a is the next a (chosen b).
        }

        // Always add the last point.
        sampled.push(self[self.len() - 1]);

        sampled.into_iter().map(Into::into).collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono_crate::*;

    use super::*;

    fn first_day_of_month(month_num: u32) -> DateTime<Utc> {
        NaiveDate::from_ymd_opt(2022, month_num, 1)
            .unwrap()
            .and_time(NaiveTime::default())
            .and_local_timezone(Utc)
            .unwrap()
    }

    #[test]
    fn lttb_test_5() {
        let dps = Vec::from([
            DataPoint::new(first_day_of_month(1), Decimal::from(10)),
            DataPoint::new(first_day_of_month(2), Decimal::from(12)),
            DataPoint::new(first_day_of_month(3), Decimal::from(8)),
            DataPoint::new(first_day_of_month(4), Decimal::from(10)),
            DataPoint::new(first_day_of_month(5), Decimal::from(12)),
        ]);

        let expected: Vec<DataOutput> = Vec::from([
            DataPoint::new(first_day_of_month(1), Decimal::from(10)).into(),
            DataPoint::new(first_day_of_month(3), Decimal::from(8)).into(),
            DataPoint::new(first_day_of_month(5), Decimal::from(12)).into(),
        ]);

        assert_eq!(expected, dps.downsample(3));
    }

    #[test]
    fn lttb_test_12() {
        let dps = Vec::from([
            DataPoint::new(first_day_of_month(1), Decimal::from(10)),
            DataPoint::new(first_day_of_month(2), Decimal::from(12)),
            DataPoint::new(first_day_of_month(3), Decimal::from(8)),
            DataPoint::new(first_day_of_month(4), Decimal::from(10)),
            DataPoint::new(first_day_of_month(5), Decimal::from(12)),
            DataPoint::new(first_day_of_month(6), Decimal::from(10)),
            DataPoint::new(first_day_of_month(7), Decimal::from(12)),
            DataPoint::new(first_day_of_month(8), Decimal::from(8)),
            DataPoint::new(first_day_of_month(9), Decimal::from(10)),
            DataPoint::new(first_day_of_month(10), Decimal::from(12)),
            DataPoint::new(first_day_of_month(11), Decimal::from(12)),
            DataPoint::new(first_day_of_month(12), Decimal::from(12)),
        ]);

        let expected: Vec<DataOutput> = Vec::from([
            DataPoint::new(first_day_of_month(1), Decimal::from(10)).into(),
            DataPoint::new(first_day_of_month(3), Decimal::from(8)).into(),
            DataPoint::new(first_day_of_month(7), Decimal::from(12)).into(),
            DataPoint::new(first_day_of_month(12), Decimal::from(12)).into(),
        ]);

        assert_eq!(expected, dps.downsample(4));
    }
}
