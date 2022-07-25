use bencher::{benchmark_group, benchmark_main, Bencher};
use ip2region::Searcher;
pub fn add_three(a: i32) -> i32 {
    a + 3
}
fn benchmark(bencher: &mut Bencher) {
    let file = "./data/ip2region.xdb";
    bencher.iter(|| {
        let searcher = Searcher::new(file).unwrap();
        searcher.std_search("120.24.78.129").ok();
    });
}
fn benchmark_only_search(bencher: &mut Bencher) {
    let file = "./data/ip2region.xdb";
    let searcher = Searcher::new(file).unwrap();
    bencher.iter(|| {
        searcher.std_search("120.24.78.129").ok();
    });
}
// how to run ? `cargo bench`
benchmark_group!(benches, benchmark);
benchmark_group!(benches2, benchmark_only_search);
benchmark_main!(benches, benches2);
