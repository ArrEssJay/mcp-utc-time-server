use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mcp_utc_time_server::time::utc::EnhancedTimeResponse;
use mcp_utc_time_server::time::UnixTime;

fn benchmark_unix_time(c: &mut Criterion) {
    c.bench_function("unix_time_now", |b| {
        b.iter(|| {
            let time = UnixTime::now();
            black_box(time);
        });
    });
}

fn benchmark_enhanced_time(c: &mut Criterion) {
    c.bench_function("enhanced_time_response", |b| {
        b.iter(|| {
            let response = EnhancedTimeResponse::now();
            black_box(response);
        });
    });
}

fn benchmark_custom_format(c: &mut Criterion) {
    c.bench_function("custom_format", |b| {
        let response = EnhancedTimeResponse::now();
        b.iter(|| {
            let formatted = response.format_custom("%Y-%m-%d %H:%M:%S").unwrap();
            black_box(formatted);
        });
    });
}

criterion_group!(
    benches,
    benchmark_unix_time,
    benchmark_enhanced_time,
    benchmark_custom_format
);
criterion_main!(benches);
