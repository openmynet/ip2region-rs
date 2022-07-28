use speedy::{Readable, Writable};
use std::path::Path;

const HEADER_INFO_LENGTH: u32 = 256;
const VECTOR_INDEX_ROWS: u32 = 256;
const VECTOR_INDEX_COLS: u32 = 256;
const VECTOR_INDEX_SIZE: u32 = 8;
const SEGMENT_INDEX_SIZE: u32 = 14;

type Error = Box<dyn std::error::Error>;

#[derive(Clone, PartialEq, Debug, Readable, Writable)]
pub struct Header {
    // data []byte
    /// 版本号
    pub version: u16,
    /// 索引策略
    pub index_policy: u16,
    /// xdb创建时间,时间戳(秒)
    pub created_at: u32,
    /// 索引起始位置
    pub start_index_ptr: u32,
    /// 索引结束位置
    pub end_index_ptr: u32,
}

impl Header {
    pub fn new(data: &[u8]) -> Result<Self, Error> {
        let cmd = Header::read_from_buffer(data)?;
        Ok(cmd)
    }
}

/// ip index
#[derive(Clone, PartialEq, Debug, Default, Readable, Writable)]
pub struct IpIndex {
    pub start_ip: u32,
    pub end_ip: u32,
    pub data_len: u16,
    pub data_index: u32,
}
impl IpIndex {
    pub fn new(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let cmd = IpIndex::read_from_buffer(data)?;
        Ok(cmd)
    }
    /// Data index start position
    ///
    /// 数据索引起始位置
    pub fn start_at(&self) -> usize {
        self.data_index as usize
    }
    /// Data index end position
    ///
    /// 数据索引结束位置
    pub fn end_at(&self) -> usize {
        (self.data_index + self.data_len as u32) as usize
    }
}
/// ip2region Official Standard Data Format
///
/// ip2region 官方标准数据格式
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Location<'a> {
    pub contry: Option<&'a str>,
    pub region: Option<&'a str>,
    pub province: Option<&'a str>,
    pub city: Option<&'a str>,
    pub isp: Option<&'a str>,
}
/// ip2region searcher
///
/// ip2region 搜索器
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Searcher {
    content: Vec<u8>,
}

impl Searcher {
    /// new Searcher
    pub fn new<P: AsRef<Path>>(file: P) -> Result<Self, Error> {
        let content = std::fs::read(file)?;
        Ok(Self { content })
    }
    /// vector index
    pub fn vector_index(&self) -> Vec<u8> {
        let len = VECTOR_INDEX_ROWS * VECTOR_INDEX_COLS * SEGMENT_INDEX_SIZE;
        let start_at = HEADER_INFO_LENGTH as usize;
        let end_at = start_at + len as usize;
        if self.content.len() < end_at {
            return vec![];
        }
        self.content[start_at..end_at].to_vec()
    }
    /// xdb content
    pub fn content(&self) -> Vec<u8> {
        self.content.clone()
    }
    /// Search IP area information
    ///
    /// 搜索IP区域信息
    pub fn search(&self, ip_v4: &str) -> Result<&str, Error> {
        let ip = ip_v4.parse::<std::net::Ipv4Addr>()?.octets();
        let ip = u32::from_be_bytes(ip);
        // Integer of the first byte (&0xFF, make sure the value is between 0 ~ 255)
        // 第一个字节的整数 (&0xFF，确保取值在 0 ~ 255)
        let il0 = (ip >> 24) & 0xFF;

        // Integer of the second byte (&0xFF, make sure the value is between 0 ~ 255)
        // 第二个字节的整数 (&0xFF，确保取值在 0 ~ 255)
        let il1 = (ip >> 16) & 0xFF;

        // Get binary index segment from vector index
        // 从 vector 索引中获取二分索引段
        let idx = il0 * VECTOR_INDEX_COLS * VECTOR_INDEX_SIZE + il1 * VECTOR_INDEX_SIZE;

        let offset = HEADER_INFO_LENGTH + idx;
        // locate the vector index position
        // 定位到向量索引位置
        let vector_index = &self.content[offset as usize..];
        let s_ptr: [u8; 4] = vector_index[..4].try_into()?;
        let s_ptr = u32::from_le_bytes(s_ptr);
        let e_ptr: [u8; 4] = vector_index[4..8].try_into()?;
        let e_ptr = u32::from_le_bytes(e_ptr);
        // Do a split-half search in a binary index
        // 在二分索引中进行拆半查找
        let mut l = 0;
        let mut h = (e_ptr - s_ptr) / SEGMENT_INDEX_SIZE;

        let mut index = None;
        while l < h {
            let m = (l + h) >> 1;
            let p = s_ptr + m * SEGMENT_INDEX_SIZE;
            let buff = &self.content[p as usize..];
            let ip_index = IpIndex::new(buff)?;
            if ip < ip_index.start_ip {
                h = m - 1;
            } else if ip > ip_index.end_ip {
                l = m + 1;
            } else {
                index = Some(ip_index);
                break;
            };
        }
        let index = index.ok_or_else(|| {
            super::error::Ip2RegionError::NoneError("No results found!".to_string())
        })?;
        let data = &self.content[index.start_at()..index.end_at()];
        let ip = std::str::from_utf8(data)?;
        Ok(ip)
    }

    /// Search and return results in official standard format
    ///
    /// 搜索并返回官方标准格式的结果
    pub fn std_search(&self, ip_v4: &str) -> Result<Location, Error> {
        let ip = self.search(ip_v4)?;
        let ip = ip.split('|').collect::<Vec<&str>>();
        let r = Location {
            contry: get(&ip, 0),
            region: get(&ip, 1),
            province: get(&ip, 2),
            city: get(&ip, 3),
            isp: get(&ip, 4),
        };
        Ok(r)
    }
    /// xdb info header
    pub fn header(&self) -> Result<Header, Error> {
        let header = Header::new(&self.content)?;
        Ok(header)
    }
}

pub fn get<'a, 'b>(v: &'a [&'b str], i: usize) -> Option<&'b str> {
    v.get(i).and_then(|i| {
        if i.is_empty() || *i == "0" {
            None
        } else {
            Some(*i)
        }
    })
}

#[test]
fn test_parse_ip() {
    let ip = "120.24.78.129";
    let ip = ip.parse::<std::net::Ipv4Addr>().unwrap();
    let ip = i32::from_be_bytes(ip.octets());
    assert_eq!(ip, 2014858881)
}

#[test]
fn test_ip() {
    let file = "./data/ip2region.xdb";
    let searcher = Searcher::new(file).unwrap();
    let h = searcher.header();
    assert!(h.is_ok());
    println!("{:?}", h.unwrap());
    let location = searcher.std_search("120.24.78.129");
    println!("{:?}", location);
    assert!(location.is_ok());
    let location = location.unwrap();
    assert_eq!(location.city, Some("深圳市"));
    assert_eq!(location.isp, Some("阿里云"));
    let location = searcher.std_search("208.67.222.222");
    println!("{:?}", location);
    assert!(location.is_ok());
}
