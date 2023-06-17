pub(crate) trait AsciiStr {
    /// Checks if the value comprised of ASCII decimal digits: U+0030 '0' ..= U+0039 '9'.
    fn is_ascii_digits(&self) -> bool;

    fn ends_with_one_of(&self, ch: impl AsRef<[char]>) -> bool;

    fn remove_matches(&self, ch: impl AsRef<[char]>) -> &Self;

    fn replace_all(&self, items: impl AsRef<[&'static str]>, s: &str) -> String;
}

impl AsciiStr for str {
    /// Checks if the value comprised of ASCII decimal digits: U+0030 '0' ..= U+0039 '9'.
    /// This is O(n)
    fn is_ascii_digits(&self) -> bool {
        for ch in self.chars() {
            if !ch.is_ascii_digit() {
                return false;
            }
        }

        true
    }

    fn ends_with_one_of(&self, ch: impl AsRef<[char]>) -> bool {
        let ch = ch.as_ref();
        for ch in ch {
            if self.ends_with(*ch) {
                return true;
            }
        }

        false
    }

    fn remove_matches(&self, ch: impl AsRef<[char]>) -> &Self {
        let ch = ch.as_ref();
        let mut s = self;
        for ch in ch.iter().copied() {
            s = s.trim_matches(ch);
        }

        s
    }

    fn replace_all(&self, items: impl AsRef<[&'static str]>, s: &str) -> String {
        let mut x = self.to_string();
        for item in items.as_ref() {
            x = x.replace(item, s);
        }
        x
    }
}
