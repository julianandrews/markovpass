pub struct Corpus {
    text: String,
    ngram_length: usize,
    original_byte_length: usize,
}

impl Corpus {
    pub fn new(
        mut reader: Box<dyn std::io::Read>,
        ngram_length: usize,
        min_word_length: usize,
    ) -> Result<Corpus, Box<dyn std::error::Error>> {
        // TODO: Process the input to generate text efficiently.
        let mut text = String::new();
        reader.read_to_string(&mut text)?;
        let mut text = Corpus::clean_text(&text, min_word_length);
        let original_byte_length = text.len();
        // Push the first few characters onto the end so we can return `&str`s for the wrap around.
        text.push_str(&text.chars().take(ngram_length).collect::<String>());

        Ok(Corpus {
            text,
            ngram_length,
            original_byte_length,
        })
    }

    pub fn ngrams(&self) -> impl Iterator<Item = &str> {
        Ngrams {
            corpus: self,
            byte_index: 0,
        }
    }

    fn clean_text(text: &str, min_word_length: usize) -> String {
        let text = text.to_lowercase();
        let words = text
            .split_whitespace()
            .filter_map(|word| Corpus::clean_word(word, min_word_length));

        // Insert a space at the start of the corpus so that every word begins with a space.
        Some("")
            .into_iter()
            .chain(words)
            .collect::<Vec<&str>>()
            .join(" ")
    }

    fn clean_word(word: &str, min_length: usize) -> Option<&str> {
        let is_word_char = |c: char| c.is_alphabetic() || c == '\'';
        let word = word.trim_matches(|c| !is_word_char(c));

        if word.chars().all(is_word_char) && word.len() >= min_length {
            Some(word)
        } else {
            None
        }
    }
}

struct Ngrams<'a> {
    corpus: &'a Corpus,
    byte_index: usize,
}

impl<'a> Iterator for Ngrams<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.byte_index >= self.corpus.original_byte_length {
            return None;
        }

        let (_, ngram_start) = self.corpus.text.split_at(self.byte_index);
        let mut ngram_char_indices = ngram_start
            .char_indices()
            .take(self.corpus.ngram_length + 1)
            .skip(1);

        let first_char_byte_length = ngram_char_indices.next().unwrap().0;
        let ngram_byte_length = ngram_char_indices
            .last()
            .map(|(i, _)| i)
            .unwrap_or(first_char_byte_length);
        let ngram_start_index = self.byte_index;
        self.byte_index += first_char_byte_length;

        Some(&self.corpus.text[ngram_start_index..ngram_start_index + ngram_byte_length])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_word() {
        assert_eq!(Corpus::clean_word("Test", 3), Some("Test"));
        assert_eq!(Corpus::clean_word("123test@314", 3), Some("test"));
        assert_eq!(Corpus::clean_word("2#@test'in23", 3), Some("test'in"));
        assert_eq!(Corpus::clean_word("31ld;Test", 3), None);
        assert_eq!(Corpus::clean_word("a", 2), None);
        assert_eq!(Corpus::clean_word("Test", 5), None);
    }

    #[test]
    fn test_clean_corpus() {
        assert_eq!(Corpus::clean_text("this is a test", 3), " this test");
        assert_eq!(Corpus::clean_text("Some awes0me test", 3), " some test");
        assert_eq!(Corpus::clean_text("test'in", 3), " test'in");
        assert_eq!(Corpus::clean_text("this is a test", 5), "");
    }

    #[test]
    fn test_ngrams() {
        let corpus = Corpus::new(Box::new("this is a test".as_bytes()), 3, 3).unwrap();
        let ngrams = corpus.ngrams();
        assert_eq!(
            ngrams.collect::<Vec<_>>(),
            vec![" th", "thi", "his", "is ", "s t", " te", "tes", "est", "st ", "t t"]
        );
        let corpus = Corpus::new(Box::new("this is a test".as_bytes()), 5, 3).unwrap();
        let ngrams = corpus.ngrams();
        assert_eq!(
            ngrams.collect::<Vec<_>>(),
            vec![
                " this", "this ", "his t", "is te", "s tes", " test", "test ", "est t", "st th",
                "t thi",
            ]
        );
        let corpus = Corpus::new(Box::new("this is a test".as_bytes()), 3, 2).unwrap();
        let ngrams = corpus.ngrams();
        assert_eq!(
            ngrams.collect::<Vec<_>>(),
            vec![
                " th", "thi", "his", "is ", "s i", " is", "is ", "s t", " te", "tes", "est", "st ",
                "t t",
            ]
        );
    }
}
