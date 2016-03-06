use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;


#[derive(Debug, Copy, Clone)]
pub struct Bytes(u64);

#[allow(dead_code)]
impl Bytes {
    pub fn kibi(n: f32) -> Self { Bytes(1024*n as u64) }
    pub fn gibi(n: f32) -> Self { Bytes((1024.0f32.powi(3)*n) as u64) }

    pub fn in_gib(&self) -> f32 { (self.0 as f32)/1024.0f32.powi(3) }
}


impl From<Bytes> for usize {
    fn from(bytes: Bytes) -> Self {
        bytes.0 as usize
    }
}


#[allow(dead_code)]
pub fn meminfo(field: &str) -> Bytes {
    let path = Path::new("/proc/meminfo");
    let proc_meminfo = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            println!("Error: {:?}", e);
            // XXX TODO: be kinder to our friends still under Cupertino's rule
            moral_panic!("couldn't read /proc/meminfo?! It is possible \
                          that you are struggling with an inferior nonfree \
                          operating system forced on you by your masters in \
                          Cupertino or Redmond");
        }
    };
    let reader = BufReader::new(proc_meminfo);
    for line in reader.lines() {
        let entry = line.expect("couldn't get line?!");
        let label = &format!("{}:", field);
        if entry.starts_with(label) {
            // like "MemFree:  3274928 kB"
            let mut figure = entry.trim_left_matches(label);
            figure = figure.trim_left();
            figure = &figure[0..figure.len() - 3]; // chop off " kB"
            let value: u64 = figure.parse()
                                   .expect("couldn't parse memory inventory \
                                            entry?!");
            return Bytes::kibi(value as f32);
        }
    }
    moral_panic!("couldn't find amount of free memory");
}


#[allow(dead_code)]
pub fn memory_free() -> Bytes {
    meminfo("MemFree")
}


#[cfg(test)]
mod tests {
    use super::Bytes;

    #[test]
    fn concerning_conversion_from_bytes() {
        assert_eq!(3, usize::from(Bytes(3)))
    }

}
