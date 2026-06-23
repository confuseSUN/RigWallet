use english::ENGLISH_WORDS;

pub mod english;

pub fn language_index_of(word: &str) -> Option<usize> {
    ENGLISH_WORDS.iter().position(|x| *x == word)
}
