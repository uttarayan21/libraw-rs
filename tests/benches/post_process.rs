use criterion::{black_box, criterion_group, criterion_main, Criterion};

const IMAGE: &[u8] = include_bytes!("../assets/RAW_NIKON_D3X.NEF");
fn post_process(data: &[u8]) -> Vec<u8> {
    use libraw_r::Processor;
    let mut processor = Processor::default();
    processor.open_buffer(data).unwrap();
    processor.unpack().unwrap();
    processor.dcraw_process().unwrap();
    let i = processor.dcraw_process_make_mem_image().unwrap();
    i.as_slice_u8().into()
}

fn libraw_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("Post Process");
    group
        .sample_size(20)
        .measurement_time(std::time::Duration::from_secs(32))
        .bench_function("Post Processing", |b| b.iter(|| post_process(black_box(IMAGE))));
    group.finish();
}

criterion_group!(benches, libraw_benches);
criterion_main!(benches);
