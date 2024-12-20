use uuid::Uuid;
use uuid_simd::UuidExt;

pub fn bench(c: &mut criterion::Criterion) {
    c.bench_function("format_uuid_v4", |b| {
        b.iter(|| {
            let value = Uuid::new_v4();
            value.to_string()
        })
    });
    c.bench_function("format_uuid_v7", |b| {
        b.iter(|| {
            let value = Uuid::now_v7();
            value.to_string()
        })
    });
    c.bench_function("encode_uuid_v4", |b| {
        b.iter(|| {
            let value = Uuid::new_v4();
            value.as_hyphenated().encode_lower(&mut [0; 36]).to_owned()
        })
    });
    c.bench_function("encode_uuid_v7", |b| {
        b.iter(|| {
            let value = Uuid::now_v7();
            value.as_hyphenated().encode_lower(&mut [0; 36]).to_owned()
        })
    });
    c.bench_function("format_uuid_v4_simd", |b| {
        b.iter(|| {
            let value = Uuid::new_v4();
            value.format_hyphenated().to_string()
        })
    });
    c.bench_function("format_uuid_v7_simd", |b| {
        b.iter(|| {
            let value = Uuid::now_v7();
            value.format_hyphenated().to_string()
        })
    });
    c.bench_function("parse_uuid_v4", |b| {
        b.iter(|| {
            let text = "67e55044-10b1-426f-9247-bb680e5fe0c8";
            text.parse::<Uuid>()
        })
    });
    c.bench_function("parse_uuid_v4_simd", |b| {
        b.iter(|| {
            let text = "67e55044-10b1-426f-9247-bb680e5fe0c8";
            Uuid::parse(text.as_bytes())
        })
    });
    c.bench_function("parse_uuid_v7", |b| {
        b.iter(|| {
            let text = "01936dc6-e48c-7d22-8e69-b29f85682fac";
            text.parse::<Uuid>()
        })
    });
    c.bench_function("parse_uuid_v7_simd", |b| {
        b.iter(|| {
            let text = "01936dc6-e48c-7d22-8e69-b29f85682fac";
            Uuid::parse(text.as_bytes())
        })
    });
}
