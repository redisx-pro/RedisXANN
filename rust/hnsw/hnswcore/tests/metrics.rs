use hnswcore::metrics;

// note: macos rustc version need nightly version 1.27+ for intel __m256 simd

#[test]
fn diff_is_zero() {
    let v1 = vec![1.0; 512];
    let v2 = vec![1.0; 512];
    let r1 = metrics::simd_avx2_euc(&v1, &v2, 512);
    assert_eq!(r1, 0.0);
    let r2 = metrics::simd_avx2_euc_v2(&v1, &v2, 512);
    assert_eq!(r2, 0.0);
    let r3 = metrics::simd_euc(&v1, &v2, 512);
    assert_eq!(r3, 0.0);
}

#[test]
fn diff_is_512() {
    let v1 = vec![0.0; 512];
    let v2 = vec![1.0; 512];
    let r1 = metrics::simd_avx2_euc(&v1, &v2, 512);
    assert_eq!(r1, -512.0);
    let r2 = metrics::simd_avx2_euc_v2(&v1, &v2, 512);
    assert_eq!(r2, -512.0);
    let r3 = metrics::simd_euc(&v1, &v2, 512);
    assert_eq!(r3, -512.0);
}

#[test]
fn diff_is_512_2_x512() {
    let v1 = vec![0.0; 512];
    let v2 = vec![512.0; 512];
    let r1 = metrics::simd_avx2_euc(&v1, &v2, 512);
    assert_eq!(r1, -134217728.0);
    let r2 = metrics::simd_avx2_euc_v2(&v1, &v2, 512);
    assert_eq!(r2, -134217728.0);
    let r3 = metrics::simd_euc(&v1, &v2, 512);
    assert_eq!(r3, -134217728.0);
}

#[test]
fn diff_non_x32() {
    let v1 = vec![0.0; 33];
    let v2 = vec![1.0; 33];
    let r1 = metrics::simd_avx2_euc_v2(&v1, &v2, 33);
    assert_eq!(r1, -33.0);
    let r2 = metrics::simd_euc(&v1, &v2, 33);
    assert_eq!(r2, -33.0);
    //assert_eq!(metrics::simd_avx2_euc(&v1, &v2, 33), -33.0);
}
