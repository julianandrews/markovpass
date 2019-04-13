// A corpus of cleaned text optimized for generating ngrams as string slices.
pub struct Corpus {
    text: String,
    wrap_around: String,
    ngram_length: usize,
}

impl Corpus {
    pub fn new(text: &str, ngram_length: usize, min_word_length: usize) -> Corpus {
        let text = clean_corpus(text, min_word_length);
        // TODO: handle unwrap failure.
        let (last_index, _) = text.char_indices().rev().nth(ngram_length - 1).unwrap();
        let mut wrap_around: String = text[last_index..].to_string();
        wrap_around.push_str(&text.chars().take(ngram_length).collect::<String>());

        Corpus {
            text: text,
            wrap_around: wrap_around,
            ngram_length: ngram_length,
        }
    }

    pub fn get_ngrams(&self) -> Vec<&str> {
        let ngrams = self
            .text
            .char_indices()
            .zip(self.text.char_indices().skip(self.ngram_length))
            .map(|((i, _), (j, _))| &self.text[i..j]);
        let wrap_ngrams = self
            .wrap_around
            .char_indices()
            .zip(self.wrap_around.char_indices().skip(self.ngram_length))
            .map(|((i, _), (j, _))| &self.wrap_around[i..j]);

        ngrams.chain(wrap_ngrams).collect()
    }
}

fn clean_corpus(text: &str, min_word_length: usize) -> String {
    let text = text.to_lowercase();
    let words = text
        .split_whitespace()
        .filter_map(|word| clean_word(word, min_word_length));

    Some("")
        .into_iter()
        .chain(words)
        .collect::<Vec<&str>>()
        .join(" ")
}

fn clean_word(word: &str, min_length: usize) -> Option<&str> {
    let word = word.trim_matches(|c| !is_word_char(c));

    if word.chars().all(is_word_char) && word.len() >= min_length {
        Some(word)
    } else {
        None
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphabetic() || c == '\''
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_word() {
        assert_eq!(clean_word("Test", 3), Some("Test"));
        assert_eq!(clean_word("123test@314", 3), Some("test"));
        assert_eq!(clean_word("2#@test'in23", 3), Some("test'in"));
        assert_eq!(clean_word("31ld;Test", 3), None);
        assert_eq!(clean_word("a", 2), None);
        assert_eq!(clean_word("Test", 5), None);
    }

    #[test]
    fn test_clean_corpus() {
        assert_eq!(clean_corpus("this is a test", 3), " this test");
        assert_eq!(clean_corpus("Some awes0me test", 3), " some test");
        assert_eq!(clean_corpus("test'in", 3), " test'in");
        assert_eq!(clean_corpus("this is a test", 5), "");
    }

    #[test]
    fn test_get_ngrams() {
        let corpus = Corpus::new("this is a test", 3, 3);
        assert_eq!(
            corpus.get_ngrams(),
            vec![" th", "thi", "his", "is ", "s t", " te", "tes", "est", "st ", "t t"]
        );
        let corpus = Corpus::new("this is a test", 5, 3);
        assert_eq!(
            corpus.get_ngrams(),
            vec![
                " this", "this ", "his t", "is te", "s tes", " test", "test ", "est t", "st th",
                "t thi",
            ]
        );
        let corpus = Corpus::new("this is a test", 3, 2);
        assert_eq!(
            corpus.get_ngrams(),
            vec![
                " th", "thi", "his", "is ", "s i", " is", "is ", "s t", " te", "tes", "est", "st ",
                "t t",
            ]
        );
    }
}
