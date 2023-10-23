#[cfg(target_arch = "x86")]
use std::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

#[derive(Copy, Clone, Debug)]
pub enum MetricFuncs {
    Euclidean,
}

pub type MetricFuncT<T, R> = fn(&[T], &[T], usize) -> R;

#[inline]
pub fn euclidean(v1: &[f32], v2: &[f32], n: usize) -> f32 {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        // TODO remove the check on array length with more flexible avx func
        if is_x86_feature_detected!("avx2") && v1.len() % 32 == 0 {
            return simd_avx2_euc_v2(v1, v2, n);
        }
    }
    simd_euc(v1, v2, n)
}

#[inline]
fn hsum_ps_sse3(v: __m128) -> f32 {
    unsafe {
        let mut shuf: __m128 = _mm_movehdup_ps(v); // broadcast elements 3,1 to 2,0
        let mut sums: __m128 = _mm_add_ps(v, shuf);
        shuf = _mm_movehl_ps(shuf, sums); // high half -> low half
        sums = _mm_add_ss(sums, shuf);
        _mm_cvtss_f32(sums)
    }
}

#[inline]
fn hsum256_ps_avx(v: __m256) -> f32 {
    unsafe {
        let mut vlow: __m128 = _mm256_castps256_ps128(v);
        let vhigh: __m128 = _mm256_extractf128_ps(v, 1); // high 128
        vlow = _mm_add_ps(vlow, vhigh); // add the low 128
        hsum_ps_sse3(vlow) // and inline the sse3 version, which is optimal for AVX
    }
}

#[inline]
#[cfg(any(target_arch = "aarch64"))]
pub(crate) fn simd_neon_euc(from: &[f32], to: &[f32], _n: usize) -> f32 {
    unsafe {
        let len = from.len() / 4 * 4;
        let buf = [0.0_f32; 4];
        let mut sum = vld1q_f32(buf.as_ptr());
        for i in (0..len).step_by(4) {
            let left = vld1q_f32(from.as_ptr().add(i));
            let right = vld1q_f32(to.as_ptr().add(i));
            let sub = vsubq_f32(left, right);
            sum = vfmaq_f32(sum, sub, sub);
        }
        let mut sum = -vaddvq_f32(sum);
        sum += simd_euc(&from[len..], &to[len..], _n - len);
        sum
    }
}

// Multiple accumulators and FMA
// since FMA has a latency of 5 cycles but 0.5 CPI
// https://stackoverflow.com/questions/45735679/euclidean-distance-using-intrinsic-instruction
// TODO: extend functionality for vectors of non-multiples of 32 floats
#[inline]
pub fn simd_avx2_euc(a: &[f32], b: &[f32], n: usize) -> f32 {
    unsafe {
        let mut euc1: __m256 = _mm256_setzero_ps();
        let mut euc2: __m256 = _mm256_setzero_ps();
        let mut euc3: __m256 = _mm256_setzero_ps();
        let mut euc4: __m256 = _mm256_setzero_ps();

        for i in (0..n).step_by(32) {
            let v1: __m256 = _mm256_sub_ps(_mm256_loadu_ps(&a[i]), _mm256_loadu_ps(&b[i]));
            euc1 = _mm256_fmadd_ps(v1, v1, euc1);

            let v2: __m256 = _mm256_sub_ps(_mm256_loadu_ps(&a[i + 8]), _mm256_loadu_ps(&b[i + 8]));
            euc2 = _mm256_fmadd_ps(v2, v2, euc2);

            let v3: __m256 =
                _mm256_sub_ps(_mm256_loadu_ps(&a[i + 16]), _mm256_loadu_ps(&b[i + 16]));
            euc3 = _mm256_fmadd_ps(v3, v3, euc3);

            let v4: __m256 =
                _mm256_sub_ps(_mm256_loadu_ps(&a[i + 24]), _mm256_loadu_ps(&b[i + 24]));
            euc4 = _mm256_fmadd_ps(v4, v4, euc4);
        }

        let res: f32 = hsum256_ps_avx(_mm256_add_ps(
            _mm256_add_ps(euc1, euc2),
            _mm256_add_ps(euc3, euc4),
        ));
        -res
    }
}

#[inline]
pub fn simd_avx2_euc_v2(from: &[f32], to: &[f32], n: usize) -> f32 {
    unsafe {
        use std::arch::x86_64::*;
        debug_assert_eq!(from.len(), to.len());

        // Get the potion of the vector that is aligned to 32 bytes.
        let len = from.len() / 8 * 8;
        let mut sums = _mm256_setzero_ps();
        for i in (0..len).step_by(8) {
            let left = _mm256_loadu_ps(from.as_ptr().add(i));
            let right = _mm256_loadu_ps(to.as_ptr().add(i));
            let sub = _mm256_sub_ps(left, right);
            // sum = sub * sub + sum
            sums = _mm256_fmadd_ps(sub, sub, sums);
        }
        // Shift and add vector, until only 1 value left.
        // sums = [x0-x7], shift = [x4-x7]
        let mut shift = _mm256_permute2f128_ps(sums, sums, 1);
        // [x0+x4, x1+x5, ..]
        sums = _mm256_add_ps(sums, shift);
        shift = _mm256_permute_ps(sums, 14);
        sums = _mm256_add_ps(sums, shift);
        sums = _mm256_hadd_ps(sums, sums);
        let mut results: [f32; 8] = [0f32; 8];
        _mm256_storeu_ps(results.as_mut_ptr(), sums);

        // Remaining
        results[0] += l2_scalar(&from[len..], &to[len..], n - len);
        -results[0]
    }
}

/// Calculate the L2 distance between two vectors
///
/// Rely on compiler auto-vectorization.
#[inline]
pub fn l2_scalar(a: &[f32], b: &[f32], _n: usize) -> f32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y) * (x - y))
        .fold(0.0, |acc, x| acc + x)
}

pub fn simd_euc(a: &[f32], b: &[f32], _n: usize) -> f32 {
    -l2_scalar(a, b, _n)
}
