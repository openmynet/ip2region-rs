# ip2region-rs
=======

[![ip2region crate](https://img.shields.io/crates/v/ip2region.svg)](https://crates.io/crates/ip2region)
[![ip2region documentation](https://docs.rs/ip2region/badge.svg)](https://docs.rs/ip2region)
![minimum rustc 1.0](https://img.shields.io/badge/rustc-1.0+-red.svg)

> ip2region 的rust非官实现版本

[ip2region官方地址](https://github.com/lionsoul2014/ip2region): <https://github.com/lionsoul2014/ip2region>

---

### ip2region数据格式

数据格式：`xdb`
格式版本：`2.0`

---

### 使用方式

```rust
fn main(){
    let searcher = ip2region::Searcher::new("./data/ip2region.xdb").unwrap();
    let ip_v4 = "120.24.78.129";
    let info  = searcher.search(ip_v4).unwrap();
    println!("{}", info)
    // => `中国|0|广东省|深圳市|阿里云`
    let info  = searcher.std_search(ip_v4).unwrap();
    println("{:?}", info)
    // => `Location { contry: Some("中国"), region: None, province: Some("广东省"), city: Some("深圳市"), isp: Some("阿里云") }`
}
```

---

### 并发查询

现已经将整个xdb加载到内存进行安全并发使用。
