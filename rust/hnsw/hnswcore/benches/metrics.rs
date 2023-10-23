#[allow(unused)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hnswcore::metrics;
use rand::distributions::Uniform;
use rand::prelude::*;

#[allow(unused)]
fn get_testdata(nb_elem: usize, dim: usize) -> Vec<Vec<f32>> {
    let mut rng = thread_rng();
    let unif = Uniform::<f32>::new(0., 1.);
    let mut data = Vec::with_capacity(nb_elem);
    for _ in 0..nb_elem {
        let column = (0..dim)
            .into_iter()
            .map(|_| rng.sample(unif))
            .collect::<Vec<f32>>();
        data.push(column);
    }
    data
}

fn get_testquery(dim: usize) -> Vec<f32> {
    let mut r_vec = Vec::<f32>::with_capacity(dim);
    let mut rng = thread_rng();
    let unif = Uniform::<f32>::new(0., 1.);
    for _ in 0..dim {
        r_vec.push(rng.sample(unif));
    }
    r_vec
}

#[allow(non_snake_case)]
fn bench_distance(c: &mut Criterion) {
    let DIMENSION = 1024;

    c.bench_function("L2(compiler auto-vectorization)", |b| {
        b.iter(|| {
            metrics::simd_euc(
                &get_testquery(DIMENSION),
                &get_testquery(DIMENSION),
                DIMENSION,
            );
        })
    });

    c.bench_function("L2(simd - intel avx2)", |b| {
        b.iter(|| {
            metrics::simd_avx2_euc(
                &get_testquery(DIMENSION),
                &get_testquery(DIMENSION),
                DIMENSION,
            );
        })
    });

    c.bench_function("L2(simd - intel avx2 v2)", |b| {
        b.iter(|| {
            metrics::simd_avx2_euc_v2(
                &get_testquery(DIMENSION),
                &get_testquery(DIMENSION),
                DIMENSION,
            );
        })
    });
}

criterion_group!(
    name=benches;
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = bench_distance);
criterion_main!(benches);
