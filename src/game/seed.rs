//! Seed codes ([067]): a run's seed as Crockford base32, grouped by four, so a player can
//! read it out, type it and share it. A new run's seed has 40 bits: eight characters.

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
/// A code has at least this many characters; a 40-bit seed fills them.
pub const CODE_LEN: usize = 8;
/// A `u64` needs at most this many.
pub const MAX_CODE_LEN: usize = 13;

pub fn code(seed: u64) -> String {
    let mut digits = vec![];
    let mut rest = seed;
    while rest > 0 || digits.len() < CODE_LEN {
        digits.push(ALPHABET[(rest % 32) as usize] as char);
        rest /= 32;
    }
    digits.reverse();
    group(&digits.into_iter().collect::<String>())
}

/// `chars` in groups of four.
pub fn group(chars: &str) -> String {
    let chars: Vec<char> = chars.chars().collect();
    chars
        .chunks(4)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("-")
}

/// The seed of a code; dashes and spaces do not count, case does not matter, and I, L and O
/// read as 1, 1 and 0.
pub fn parse(text: &str) -> Option<u64> {
    let mut seed = 0u64;
    let mut len = 0;
    for c in text.chars().filter(|c| *c != '-' && !c.is_whitespace()) {
        seed = seed.checked_mul(32)?.checked_add(digit(c)?)?;
        len += 1;
    }
    (len > 0).then_some(seed)
}

/// The value of one code character, or `None` if it is not one.
pub fn digit(c: char) -> Option<u64> {
    let c = match c.to_ascii_uppercase() {
        'I' | 'L' => '1',
        'O' => '0',
        c => c,
    };
    ALPHABET
        .iter()
        .position(|&a| a as char == c)
        .map(|value| value as u64)
}

/// The code in pasted text: the last word that is a code, as its plain characters (upper
/// case, no dashes). A shared message ends with the code.
pub fn find(text: &str) -> Option<String> {
    text.split_whitespace().rev().find_map(|word| {
        let chars: String = word
            .chars()
            .filter(|c| *c != '-')
            .map(|c| digit(c).map(char_of))
            .collect::<Option<_>>()?;
        (!chars.is_empty() && parse(&chars).is_some()).then_some(chars)
    })
}

/// The character of a value from 0 to 31.
pub fn char_of(value: u64) -> char {
    ALPHABET[value as usize] as char
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_seed_round_trips_through_its_code() {
        for seed in [0, 1, 31, 32, 1 << 39, (1 << 40) - 1, 1 << 40, u64::MAX] {
            assert_eq!(parse(&code(seed)), Some(seed), "{seed}");
        }
        assert_eq!(code(1), "0000-0001");
        assert_eq!(code((1 << 40) - 1), "ZZZZ-ZZZZ");
        assert_eq!(code(u64::MAX), "FZZZ-ZZZZ-ZZZZ-Z");
        assert_eq!(code(u64::MAX).replace('-', "").len(), MAX_CODE_LEN);
    }

    #[test]
    fn a_code_reads_loosely_and_rejects_junk() {
        let seed = parse("K3M9-ZQ4T").unwrap();
        assert_eq!(parse("k3m9 zq4t"), Some(seed));
        assert_eq!(parse("1OIL"), parse("1011"));
        assert_eq!(parse("U"), None, "no U in Crockford base32");
        assert_eq!(parse(""), None);
        assert_eq!(parse("--"), None);
        assert_eq!(parse("ZZZZ-ZZZZ-ZZZZ-Z"), None, "past u64");
    }

    #[test]
    fn a_pasted_message_gives_its_code() {
        assert_eq!(find("Rix seed k3m9-zq4t").as_deref(), Some("K3M9ZQ4T"));
        assert_eq!(find("  K3M9-ZQ4T\n").as_deref(), Some("K3M9ZQ4T"));
        assert_eq!(find("I0L0").as_deref(), Some("1010"));
        assert_eq!(find("you rock!"), None, "no word is all code characters");
    }
}
