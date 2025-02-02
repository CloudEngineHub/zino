use rand::{seq::IndexedRandom, Rng};

/// Country codes for a mocked phone number.
const COUNTRY_CODES: [&str; 3] = ["+1", "+49", "+86"];

/// Generates a random phone number.
pub(crate) fn gen_phone_number() -> String {
    let mut rng = rand::rng();
    let country_code = COUNTRY_CODES.choose(&mut rng).unwrap_or(&"+86");
    let national_number = match *country_code {
        "+1" => (0..10)
            .map(|i| match i {
                0 => rng.random_range('2'..='9'),
                1 => rng.random_range('4'..='9'),
                _ => rng.random_range('0'..='9'),
            })
            .collect::<String>(),
        "+49" => (0..11)
            .map(|i| match i {
                0 => '1',
                1 => '7',
                2 => rng.random_range('1'..='9'),
                _ => rng.random_range('0'..='9'),
            })
            .collect::<String>(),
        _ => (0..11)
            .map(|i| match i {
                0 => '1',
                1 => rng.random_range('3'..='9'),
                _ => rng.random_range('0'..='9'),
            })
            .collect::<String>(),
    };
    format!("{country_code}{national_number}")
}
