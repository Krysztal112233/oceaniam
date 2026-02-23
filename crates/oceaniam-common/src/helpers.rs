use rand::Rng;

pub fn gen_random(len: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";

    gen_random_with_charset(len, CHARSET)
}

pub fn gen_random_with_charset(len: usize, charset: &[u8]) -> String {
    let mut rng = rand::thread_rng();

    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

pub fn gen_random_name() -> String {
    names::Generator::with_naming(names::Name::Plain)
        .next()
        .unwrap()
}
