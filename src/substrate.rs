use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;


pub fn inventory_memory_kib() -> u32 {
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
        let entry = line.ok().expect("couldn't get line?!");
        if entry.starts_with("MemFree:") {  // like "MemFree:  3274928 kB"
            let mut figure = entry.trim_left_matches("MemFree:");
            figure = figure.trim_left();
            figure = &figure[0..figure.len()-3]; // chop off " kB"
            let value: u32 = figure.parse()
                .ok().expect("couldn't parse memory inventory entry?!");
            return value
        }
    }
    moral_panic!("couldn't find amount of free memory");
}

pub fn inventory_memory_gib() -> f32 {
    inventory_memory_kib() as f32 / (1024. * 1024.)
}

#[cfg(test)]
mod tests {
    use super::inventory_memory_kib;

    #[test]
    fn concerning_the_inventory_of_memory() {
        println!("We have this many kB: {}", inventory_memory_kib());
    }
}
