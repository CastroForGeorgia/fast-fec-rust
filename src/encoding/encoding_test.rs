#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_line() {
        // Only ASCII => valid UTF-8, no fallback
        let input = b"Hello, world!";
        let (decoded, has28) = decode_line(input);
        assert_eq!(decoded, "Hello, world!");
        assert!(!has28);
    }

    #[test]
    fn test_ascii28() {
        // Contains ASCII28 (0x1C)
        let input = b"Hello\x1Cthere";
        let (decoded, has28) = decode_line(input);
        assert_eq!(decoded, "Hello\x1Cthere");
        // we don't transform ASCII28 out
        assert!(has28);
    }

    #[test]
    fn test_iso_8859_1() {
        // Example: 0xE9 is 'é' in ISO-8859-1
        // It's not valid ASCII. If we interpret as UTF-8, this is invalid
        let input = vec![0x48, 0x69, 0x20, 0xE9];
        // "Hi é" in ISO-8859-1
        // The decode_line should fallback and interpret that as "Hi " + "é" in UTF-8
        let (decoded, has28) = decode_line(&input);
        assert_eq!(decoded, "Hi é");
        assert!(!has28);
    }

    #[test]
    fn test_valid_utf8() {
        // This is valid UTF-8: "El Niño" with 'ñ' => 0xC3 0xB1
        let input = "El Niño".as_bytes();
        let (decoded, has28) = decode_line(input);
        assert_eq!(decoded, "El Niño");
        assert!(!has28);
    }

    #[test]
    fn test_invalid_utf8_trigger_fallback() {
        // A quick invalid sequence: 0xF0 0x28 => Typically invalid
        let input = vec![0xF0, 0x28, 0x9F];
        let (decoded, has28) = decode_line(&input);
        // We'll fallback to ISO-8859-1 => 0xF0 => 'ò' or something in that range
        // Actually 0xF0 => 0xC3 0xB0 if > 0xBF
        // In decimal, 0xF0=240 => first = 0xc2 + 1 => 0xc3, second => (240 & 0x3f)+0x80 => 0xb0 => '°'
        // So we get "°("... let's see
        // Actually let's see the entire sequence:
        // 0xF0 => 0xc3 0xb0 => "ð"
        // 0x28 => '(' => ASCII => "("
        // 0x9F => 0xc2 + 1 => 0xc3, (0x9F & 0x3f)+0x80 => 0x1F+0x80 => 0x9F => "" (control?)
        // So final => "ð(\u{9f}"
        println!("Fallback => {}", decoded);
        assert!(!has28);
    }
}
