#[cfg(test)]
mod tests {
    extern crate test;
    use space::Pinfield;
    use landmark::FILES;

    #[test]
    fn concerning_maps_and_territories() {
        for (file, pins) in FILES.iter().enumerate() {
            let locales = Pinfield(*pins).to_locales();
            assert_eq!(8, locales.len());
            for locale in locales {
                assert_eq!(file as u8, locale.file);
            }
        }
    }
}
