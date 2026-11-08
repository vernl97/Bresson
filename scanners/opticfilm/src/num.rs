// SPDX-License-Identifier: GPL-3.0-or-later
//! Python / numpy arithmetic that the port must reproduce exactly.

/// Python `round(x)` / numpy `rint`: round half to even.
pub fn py_round(x: f64) -> i64 {
    x.round_ties_even() as i64
}

/// Python `round(x, ndigits)` for display values (half-even on the scaled value).
pub fn py_round_to(x: f64, ndigits: i32) -> f64 {
    let m = 10f64.powi(ndigits);
    (x * m).round_ties_even() / m
}

/// Python `a // b` (floor division) on integers.
pub fn floor_div(a: i64, b: i64) -> i64 {
    a.div_euclid(b) - if b < 0 && a.rem_euclid(b) != 0 { 1 } else { 0 }
}

/// Python `a % b` on integers (sign of the divisor).
pub fn py_mod(a: i64, b: i64) -> i64 {
    a - b * floor_div(a, b)
}

/// Python `int(x)` on a float: truncate toward zero.
pub fn py_int(x: f64) -> i64 {
    x.trunc() as i64
}

/// Python `math.ceil` to an integer.
pub fn ceil_i(x: f64) -> i64 {
    x.ceil() as i64
}

/// Python `min(table, key=lambda k: abs(k - x))`: first key with the smallest distance.
pub fn nearest_key<V>(table: &[(i64, V)], x: i64) -> Option<&(i64, V)> {
    let mut best: Option<&(i64, V)> = None;
    for e in table {
        if best.is_none_or(|b| (e.0 - x).abs() < (b.0 - x).abs()) {
            best = Some(e);
        }
    }
    best
}

/// Look up `key` in a static `(key, value)` table.
pub fn lookup<K: PartialEq + Copy, V: Copy>(table: &[(K, V)], key: K) -> Option<V> {
    table.iter().find(|e| e.0 == key).map(|e| e.1)
}

/// numpy `_lerp` as used by `np.percentile(method="linear")`.
fn np_lerp(a: f64, b: f64, t: f64) -> f64 {
    let d = b - a;
    if t >= 0.5 {
        b - d * (1.0 - t)
    } else {
        a + d * t
    }
}

/// `np.percentile(sorted, q)` (linear method) on already-sorted data.
pub fn percentile_sorted(sorted: &[f64], q: f64) -> f64 {
    let n = sorted.len();
    assert!(n > 0, "percentile of empty data");
    let virt = (q / 100.0) * (n - 1) as f64;
    let prev = virt.floor();
    let gamma = virt - prev;
    let i = (prev as isize).clamp(0, n as isize - 1) as usize;
    let j = (i + 1).min(n - 1);
    np_lerp(sorted[i], sorted[j], gamma)
}

/// Sort a copy (NaN-free data).
pub fn sorted_f64(it: impl IntoIterator<Item = f64>) -> Vec<f64> {
    let mut v: Vec<f64> = it.into_iter().collect();
    v.sort_by(|a, b| a.partial_cmp(b).expect("NaN in percentile data"));
    v
}

/// `np.percentile(data, q)` (linear method).
pub fn percentile(data: impl IntoIterator<Item = f64>, q: f64) -> f64 {
    percentile_sorted(&sorted_f64(data), q)
}

/// `np.percentile(data, [q...])`.
pub fn percentiles(data: impl IntoIterator<Item = f64>, qs: &[f64]) -> Vec<f64> {
    let s = sorted_f64(data);
    qs.iter().map(|&q| percentile_sorted(&s, q)).collect()
}

/// `np.median(data)`: mean of the two middle values for even counts.
pub fn median(data: impl IntoIterator<Item = f64>) -> f64 {
    let s = sorted_f64(data);
    let n = s.len();
    assert!(n > 0, "median of empty data");
    if n % 2 == 1 {
        s[n / 2]
    } else {
        (s[n / 2 - 1] + s[n / 2]) / 2.0
    }
}

/// `np.mean` (f64 accumulator).
pub fn mean(data: impl IntoIterator<Item = f64>) -> f64 {
    let (mut s, mut n) = (0.0, 0usize);
    for v in data {
        s += v;
        n += 1;
    }
    if n == 0 { f64::NAN } else { s / n as f64 }
}

/// `np.clip(np.rint(x), 0, 65535).astype(np.uint16)`.
pub fn to_u16(x: f64) -> u16 {
    x.round_ties_even().clamp(0.0, 65535.0) as u16
}

/// `np.searchsorted(a, v, side="left")`.
pub fn searchsorted_left(a: &[f64], v: f64) -> usize {
    a.partition_point(|&x| x < v)
}

/// `np.searchsorted(a, v, side="right")`.
pub fn searchsorted_right(a: &[f64], v: f64) -> usize {
    a.partition_point(|&x| x <= v)
}

/// 65536-bin histogram of u16 samples: exact order statistics without sorting.
pub struct U16Hist {
    pub counts: Vec<u64>,
    pub n: u64,
}

impl U16Hist {
    pub fn new(it: impl IntoIterator<Item = u16>) -> Self {
        let mut counts = vec![0u64; 65536];
        let mut n = 0;
        for v in it {
            counts[v as usize] += 1;
            n += 1;
        }
        Self { counts, n }
    }

    /// The `k`-th smallest sample (0-based).
    pub fn kth(&self, k: u64) -> f64 {
        let mut acc = 0;
        for (v, &c) in self.counts.iter().enumerate() {
            acc += c;
            if acc > k {
                return v as f64;
            }
        }
        65535.0
    }

    /// `np.percentile(samples, q)` (linear method).
    pub fn percentile(&self, q: f64) -> f64 {
        assert!(self.n > 0, "percentile of empty data");
        let virt = (q / 100.0) * (self.n - 1) as f64;
        let prev = virt.floor();
        let gamma = virt - prev;
        let i = (prev.max(0.0) as u64).min(self.n - 1);
        let j = (i + 1).min(self.n - 1);
        let (a, b) = (self.kth(i), self.kth(j));
        let d = b - a;
        if gamma >= 0.5 {
            b - d * (1.0 - gamma)
        } else {
            a + d * gamma
        }
    }
}

impl U16Hist {
    /// `np.percentile(f(samples), q)` for a monotone non-decreasing `f`.
    pub fn percentile_mapped(&self, q: f64, f: impl Fn(f64) -> f64) -> f64 {
        assert!(self.n > 0, "percentile of empty data");
        let virt = (q / 100.0) * (self.n - 1) as f64;
        let prev = virt.floor();
        let gamma = virt - prev;
        let i = (prev.max(0.0) as u64).min(self.n - 1);
        let j = (i + 1).min(self.n - 1);
        let (a, b) = (f(self.kth(i)), f(self.kth(j)));
        let d = b - a;
        if gamma >= 0.5 {
            b - d * (1.0 - gamma)
        } else {
            a + d * gamma
        }
    }
}

/// `np.percentile(f32_data, q)` with scalar `q`: numpy lerps in float32.
pub fn percentile_sorted_f32(sorted: &[f32], q: f64) -> f32 {
    let n = sorted.len();
    assert!(n > 0, "percentile of empty data");
    let virt = (q / 100.0) * (n - 1) as f64;
    let prev = virt.floor();
    let gamma = (virt - prev) as f32;
    let i = (prev as isize).clamp(0, n as isize - 1) as usize;
    let j = (i + 1).min(n - 1);
    let (a, b) = (sorted[i], sorted[j]);
    let d = b - a;
    if gamma >= 0.5 {
        b - d * (1.0 - gamma)
    } else {
        a + d * gamma
    }
}

/// `np.quantile(f32_data, qs_array)`: float32 difference, float64 result.
pub fn quantiles_sorted_f32(sorted: &[f32], qs: &[f64]) -> Vec<f64> {
    let n = sorted.len();
    qs.iter()
        .map(|&q| {
            let virt = q * (n - 1) as f64;
            let prev = virt.floor();
            let gamma = virt - prev;
            let i = (prev as isize).clamp(0, n as isize - 1) as usize;
            let j = (i + 1).min(n - 1);
            let (a, b) = (sorted[i], sorted[j]);
            let d = (b - a) as f64;
            if gamma >= 0.5 {
                b as f64 - d * (1.0 - gamma)
            } else {
                a as f64 + d * gamma
            }
        })
        .collect()
}

pub fn sort_f32(mut v: Vec<f32>) -> Vec<f32> {
    v.sort_by(|a, b| a.partial_cmp(b).expect("NaN in data"));
    v
}

/// `np.median(f32_data)`: float32 mean of the middle pair.
pub fn median_f32(v: Vec<f32>) -> f32 {
    let s = sort_f32(v);
    let n = s.len();
    assert!(n > 0, "median of empty data");
    if n % 2 == 1 {
        s[n / 2]
    } else {
        (s[n / 2 - 1] + s[n / 2]) / 2.0
    }
}

/// `np.linspace(a, b, n)`.
pub fn linspace(a: f64, b: f64, n: usize) -> Vec<f64> {
    if n == 1 {
        return vec![a];
    }
    let step = (b - a) / (n - 1) as f64;
    let mut v: Vec<f64> = (0..n).map(|i| a + i as f64 * step).collect();
    v[n - 1] = b;
    v
}

/// Least squares for `[x, 1] @ [a, b] ≈ y` with row weights `w` (2×2 normal equations).
pub fn lstsq_line(x: &[f64], y: &[f64], w: &[f64]) -> [f64; 2] {
    let (mut sxx, mut sx, mut s1, mut sxy, mut sy) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for i in 0..x.len() {
        let (wx, wi) = (x[i] * w[i], w[i]);
        let wy = y[i] * w[i];
        sxx += wx * wx;
        sx += wx * wi;
        s1 += wi * wi;
        sxy += wx * wy;
        sy += wi * wy;
    }
    let det = sxx * s1 - sx * sx;
    if det.abs() < 1e-300 {
        return [0.0, if s1 > 0.0 { sy / s1 } else { 0.0 }];
    }
    [(sxy * s1 - sx * sy) / det, (sxx * sy - sx * sxy) / det]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_rounding_and_division() {
        assert_eq!(py_round(2.5), 2);
        assert_eq!(py_round(3.5), 4);
        assert_eq!(py_round(-0.5), 0);
        assert_eq!(floor_div(-7, 2), -4);
        assert_eq!(floor_div(7, -2), -4);
        assert_eq!(floor_div(7, 2), 3);
        assert_eq!(py_mod(-7, 3), 2);
        assert_eq!(py_mod(7, -3), -2);
    }
}
