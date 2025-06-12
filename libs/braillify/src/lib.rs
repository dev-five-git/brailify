use jauem::choseong::encode_choseong;
use moeum::jungsong::encode_jungsong;
use utils::has_choseong_o;

use crate::{
    char_struct::CharType,
    jauem::jongseong::encode_jongseong,
    korean_char::encode_korean_char,
    rule::{rule_11, rule_12},
    rule_en::{rule_en_10_4, rule_en_10_6},
    split::split_korean_jauem,
};

mod char_shortcut;
mod char_struct;
mod english;
mod jauem;
mod korean_char;
mod korean_part;
mod math_symbol_shortcut;
mod moeum;
mod number;
mod rule;
mod rule_en;
mod split;
mod symbol_shortcut;
mod unicode;
mod utils;
mod word_shortcut;

pub struct Encoder {
    is_english: bool,
    triple_big_english: bool,
    english_indicator: bool,
    has_processed_word: bool,
}

impl Encoder {
    pub fn new(english_indicator: bool) -> Self {
        Self {
            english_indicator,
            is_english: false,
            triple_big_english: false,
            has_processed_word: false,
        }
    }

    pub fn encode(&mut self, text: &str, result: &mut Vec<u8>) -> Result<(), String> {
        let words = text
            .split_ascii_whitespace()
            .filter(|word| !word.is_empty())
            .collect::<Vec<&str>>();

        let mut word: &str = "";
        let mut remaining_words = &words[..];
        while !remaining_words.is_empty() {
            let prev_word = word;
            (word, remaining_words) = remaining_words.split_first().unwrap();

            let mut skip_count = 0;

            self.encode_word(word, prev_word, remaining_words, &mut skip_count, result)?;
        }
        Ok(())
    }

    fn encode_word(
        &mut self,
        word: &str,
        prev_word: &str,
        remaining_words: &[&str],
        skip_count: &mut usize,
        result: &mut Vec<u8>,
    ) -> Result<(), String> {
        if let Some((_, code, rest)) = word_shortcut::split_word_shortcut(word) {
            result.extend(code);
            if !rest.is_empty() {
                // Recursively encode the rest using the current encoder state
                self.encode(rest.as_str(), result)?;
            }
        } else {
            let word_chars = word.chars().collect::<Vec<char>>();
            let word_len = word_chars.len();
            let is_all_uppercase = word_chars.iter().all(|c| c.is_uppercase());
            let has_korean_char = word_chars
                .iter()
                .any(|c| (0xAC00 <= *c as u32 && *c as u32 <= 0xD7A3));

            if self.english_indicator && !self.is_english && word_chars[0].is_ascii_alphabetic() {
                // 제31항 국어 문장 안에 그리스 문자가 나올 때에는 그 앞에 로마자표 ⠴을 적고 그 뒤에 로마자 종료표 ⠲을 적는다

                self.is_english = true;
                result.push(52);
            }

            if is_all_uppercase && !self.triple_big_english {
                if (!self.has_processed_word || !prev_word.chars().all(|c| c.is_ascii_alphabetic()))
                    && remaining_words.len() >= 2
                    && remaining_words[0].chars().all(|c| c.is_ascii_alphabetic())
                    && remaining_words[1].chars().all(|c| c.is_ascii_alphabetic())
                {
                    self.triple_big_english = true;
                    result.push(32);
                    result.push(32);
                    result.push(32);
                } else if word_len >= 2 {
                    // 28항 [붙임] 로마자가 한 글자만 대문자일 때에는 대문자 기호표 ⠠을 그 앞에 적고, 단
                    // 어 전체가 대문자이거나 두 글자 이상 연속해서 대문자일 때에는 대문자 단어표
                    // ⠠을 그 앞에 적는다. 세 개 이상의 연속된 단어가 모두 대문자일 때에는 첫 단어
                    // 앞에 대문자 구절표 ⠠⠠⠠을 적고, 마지막 단어 뒤에 대문자 종료표 ⠠⠄을 적는다.
                    result.push(32);
                    result.push(32);
                }
            }

            let mut is_number = false;
            let mut is_big_english = false;
            // let mut over_three_big_english = false;

            for (i, c) in word_chars.iter().enumerate() {
                if *skip_count > 0 {
                    *skip_count -= 1;
                    continue;
                }

                let char_type = CharType::new(*c)?;

                if self.english_indicator && i > 0 && !c.is_ascii_alphabetic() {
                    // 제31항 국어 문장 안에 그리스 문자가 나올 때에는 그 앞에 로마자표 ⠴을 적고 그 뒤에 로마자 종료표 ⠲을 적는다
                    if self.is_english && !['"', ')', '('].contains(c) {
                        // 제34항 로마자가 따옴표나 괄호 등으로 묶일 때에는 로마자 종료표를 적지 않는다.
                        result.push(50);
                    }
                    self.is_english = false;
                }

                match char_type {
                    CharType::Korean(korean) => {
                        if is_number
                            && (['ㄴ', 'ㄷ', 'ㅁ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'].contains(&korean.cho)
                                || *c == '운')
                        {
                            // 44항 [다만] 숫자와 혼동되는 ‘ㄴ, ㄷ, ㅁ, ㅋ, ㅌ, ㅍ, ㅎ’의 첫소리 글자와 ‘운’의 약자는 숫자 뒤에 붙어 나오더라도 숫자와 한글을 띄어 쓴다.
                            result.push(0);
                        }

                        // "겄"의 경우 4항으로 해석해야 하지만 "것 + ㅅ" 으로 해석될 여지가 있으므로 예외처리
                        if ['팠', '껐', '셩', '쎵', '졍', '쪙', '쳥', '겄'].contains(c) {
                            // 14항 [붙임] "팠"을 적을 때에는 "ㅏ"를 생략하지 않고 적는다.
                            // 16항 [붙임] ‘껐’을 적을 때에는 ‘꺼’와 받침 ‘ㅆ’ 약자를 어울러 적는다.
                            // 제17항 ‘성, 썽, 정, 쩡, 청’을 적을 때에는 ‘ㅅ, ㅆ, ㅈ, ㅉ, ㅊ’ 다음에 ‘영’ 의 약자 ⠻을 적어 나타낸다. -> 그러므로 셩, 쪙 등 [ㅅ, ㅆ, ㅈ, ㅉ, ㅊ] + 영의 경우 초, 중, 종성 모두 결합
                            let (cho0, cho1) = split_korean_jauem(korean.cho)?;
                            if cho1.is_some() {
                                // 쌍자음 경우의 수
                                result.push(32);
                            }
                            result.push(encode_choseong(cho0)?);
                            result.extend(encode_jungsong(korean.jung)?);
                            result.extend(encode_jongseong(korean.jong.unwrap())?);
                        } else if ['나', '다', '마', '바', '자', '카', '타', '파', '하'].contains(c)
                            && i < word_len - 1
                            && has_choseong_o(word_chars[i + 1])
                        {
                            // 14항 ‘나, 다, 마, 바, 자, 카, 타, 파, 하’에 모음이 붙어 나올 때에는 약자를 사용하지 않는다
                            result.push(encode_choseong(korean.cho)?);
                            result.extend(encode_jungsong(korean.jung)?);
                        } else {
                            result.extend(encode_korean_char(&korean)?);
                        }

                        if i < word_len - 1 {
                            // 11 - 모음자에 ‘예’가 붙어 나올 때에는 그 사이에 구분표 -을 적어 나타낸다
                            rule_11(&korean, word_chars[i + 1], result)?;
                            rule_12(&korean, word_chars[i + 1], result)?;
                        }
                    }
                    CharType::KoreanPart(c) => {
                        match word_len {
                            1 => {
                                // 8항 - 단독으로 쓰인 자모
                                result.push(63);
                                result.extend(korean_part::encode_korean_part(c)?);
                            }
                            2 => {
                                // 9항 - 한글의 자음자가 번호로 쓰이는 경우
                                if i == 0 && word_chars[1] == '.' {
                                    result.push(63);
                                    result.extend(jauem::jongseong::encode_jongseong(c)?);
                                } else {
                                    // 8항 - 단독으로 쓰인 자모
                                    result.push(63);
                                    result.extend(korean_part::encode_korean_part(c)?);
                                }
                            }
                            _ => {
                                if (i == 0 && word_len > 1 && word_chars[1] == '자')
                                    || ((i == 0
                                        || (i > 0
                                            && matches!(
                                                CharType::new(word_chars[i - 1])?,
                                                CharType::Symbol(_)
                                            )))
                                        && (word_len - 1 == i
                                            || (i < word_len - 1
                                                && matches!(
                                                    CharType::new(word_chars[i + 1])?,
                                                    CharType::Symbol(_)
                                                ))))
                                {
                                    // 8항 - 단독으로 쓰인 자모
                                    result.push(63);
                                    result.extend(korean_part::encode_korean_part(c)?);
                                } else {
                                    if has_korean_char {
                                        // 10항 - 단독으로 쓰인 자음자가 단어에 붙어 나올 때
                                        result.push(56);
                                        result.extend(korean_part::encode_korean_part(c)?);
                                    } else {
                                        // 10항 - 단독으로 쓰인 자음자가 단어에 붙어 나올 때
                                        // 8항 - 단독으로 쓰인 자모
                                        result.push(63);
                                        result.extend(korean_part::encode_korean_part(c)?);
                                    }
                                }
                            }
                        }
                    }
                    CharType::English(c) => {
                        if self.english_indicator && !self.is_english {
                            // 제31항 국어 문장 안에 그리스 문자가 나올 때에는 그 앞에 로마자표 ⠴을 적고 그 뒤에 로마자 종료표 ⠲을 적는다

                            result.push(52);
                        }

                        if (!is_all_uppercase || word_len < 2)
                            && !is_big_english
                            && c.is_uppercase()
                        {
                            // 28항 [붙임] 로마자가 한 글자만 대문자일 때에는 대문자 기호표 ⠠을 그 앞에 적고, 단어 전체가 대문자이거나 두 글자 이상 연속해서 대문자일 때에는 대문자 단어표
                            // ⠠⠠을 그 앞에 적는다. 세 개 이상의 연속된 단어가 모두 대문자일 때에는 첫 단어
                            // 앞에 대문자 구절표 ⠠⠠⠠을 적고, 마지막 단어 뒤에 대문자 종료표 ⠠⠄을 적는다.
                            is_big_english = true;

                            for idx in 0..std::cmp::min(word_len - i, 2) {
                                if word_chars[i + idx].is_uppercase() {
                                    result.push(32);
                                } else {
                                    break;
                                }
                            }
                        }
                        if !self.is_english || i == 0 {
                            if let Some((code, len)) = rule_en_10_6(
                                &word_chars[i..].iter().collect::<String>().to_lowercase(),
                            ) {
                                result.push(code);
                                *skip_count = len;
                            } else if let Some((code, len)) = rule_en_10_4(
                                &word_chars[i..].iter().collect::<String>().to_lowercase(),
                            ) {
                                result.push(code);
                                *skip_count = len;
                            } else {
                                result.push(english::encode_english(c)?);
                            }
                        } else if let Some((code, len)) =
                            rule_en_10_4(&word_chars[i..].iter().collect::<String>().to_lowercase())
                        {
                            result.push(code);
                            *skip_count = len;
                        } else {
                            result.push(english::encode_english(c)?);
                        }
                        self.is_english = true;
                    }
                    CharType::Number(c) => {
                        if !is_number {
                            // 제43항 숫자 사이에 마침표, 쉼표, 연결표가 붙어 나올 때에는 뒤의 숫자에 수표를 적지 않는다.
                            if !(i > 0 && ['.', ','].contains(&word_chars[i - 1])) {
                                // 제40항 숫자는 수표 ⠼을 앞세워 다음과 같이 적는다.
                                result.push(60);
                            }
                            is_number = true;
                        }
                        result.extend(number::encode_number(c));
                    }
                    CharType::Symbol(c) => {
                        if c == ','
                            && is_number
                            && i < word_len - 1
                            && word_chars[i + 1].is_numeric()
                        {
                            // 제41항 숫자 사이에 붙어 나오는 쉼표와 자릿점은 ⠂으로 적는다.
                            result.push(2);
                        } else {
                            result.extend(symbol_shortcut::encode_char_symbol_shortcut(c)?);
                        }
                    }
                    CharType::Space(c) => {
                        result.push(if c == '\n' { 255 } else { 0 });
                    }
                    CharType::MathSymbol(c) => {
                        if i > 0
                            && word_chars[..i]
                                .iter()
                                .any(|c| (0xAC00 <= *c as u32 && *c as u32 <= 0xD7A3))
                        {
                            result.push(0);
                        }
                        result.extend(math_symbol_shortcut::encode_char_math_symbol_shortcut(c)?);
                        if i < word_len - 1 {
                            let mut korean = vec![];
                            for wc in word_chars[i..].iter() {
                                if 0xAC00 <= *wc as u32 && *wc as u32 <= 0xD7A3 {
                                    korean.push(*wc);
                                } else if !korean.is_empty() {
                                    break;
                                }
                            }
                            if !korean.is_empty() {
                                // 조사일 경우, 수 뒤에 올 경우 구분하는 것으로 판단
                                if !["과", "와", "이다", "하고", "이랑", "와", "랑", "아니다"]
                                    .contains(&korean.iter().collect::<String>().as_str())
                                {
                                    result.push(0);
                                }
                            }
                        }
                    }
                }
                if !c.is_numeric() {
                    is_number = false;
                }
                if c.is_ascii_alphabetic() && !c.is_uppercase() {
                    is_big_english = false;
                }
            }
        }

        if self.triple_big_english {
            if !(remaining_words
                .first()
                .is_some_and(|w| w.chars().all(|c| c.is_ascii_alphabetic())))
            {
                // 28항 [붙임] 로마자가 한 글자만 대문자일 때에는 대문자 기호표 ⠠을 그 앞에 적고, 단어 전체가 대문자이거나 두 글자 이상 연속해서 대문자일 때에는 대문자 단어표
                // ⠠⠠을 그 앞에 적는다. 세 개 이상의 연속된 단어가 모두 대문자일 때에는 첫 단어
                // 앞에 대문자 구절표 ⠠⠠⠠을 적고, 마지막 단어 뒤에 대문자 종료표 ⠠⠄을 적는다.
                result.push(32);
                result.push(4);
                self.triple_big_english = false; // Reset after adding terminator
            }
        }
        if !remaining_words.is_empty() {
            if self.english_indicator
                && !remaining_words[0]
                    .chars()
                    .next()
                    .unwrap()
                    .is_ascii_alphabetic()
            {
                // 제31항 국어 문장 안에 그리스 문자가 나올 때에는 그 앞에 로마자표 ⠴을 적고 그 뒤에 로마자 종료표 ⠲을 적는다
                if self.is_english {
                    result.push(50);
                }
                self.is_english = false;
            }

            result.push(0);
        }

        // Update state for next iteration
        if !self.has_processed_word {
            self.has_processed_word = true;
        }
        Ok(())
    }

    pub fn finish(&mut self, result: &mut Vec<u8>) -> Result<(), String> {
        // Handle any end-of-stream processing
        if self.triple_big_english {
            // Close triple big english if still active
            result.push(32); // ⠠
            result.push(4); // ⠄
        }
        Ok(())
    }
}

pub fn encode(text: &str) -> Result<Vec<u8>, String> {
    // 한국어가 존재할 경우 english_indicator 가 true 가 됩니다.
    let english_indicator = text.split(' ').filter(|word| !word.is_empty()).any(|word| {
        word.chars().any(|c| {
            (c as u32 >= 0x3131 && c as u32 <= 0x3163) || (0xAC00 <= c as u32 && c as u32 <= 0xD7A3)
        })
    });

    let mut encoder = Encoder::new(english_indicator);
    let mut result = Vec::new();
    encoder.encode(text, &mut result)?;
    encoder.finish(&mut result)?;
    Ok(result)
}

pub fn encode_to_unicode(text: &str) -> Result<String, String> {
    let result = encode(text)?;
    Ok(result
        .iter()
        .map(|c| unicode::encode_unicode(*c))
        .collect::<String>())
}

pub fn encode_to_braille_font(text: &str) -> Result<String, String> {
    let result = encode(text)?;
    Ok(result
        .iter()
        .map(|c| unicode::encode_unicode(*c))
        .collect::<String>())
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, fs::File};

    use crate::unicode::encode_unicode;
    use proptest::prelude::*;

    use super::*;
    #[test]
    pub fn test_encode() {
        assert_eq!(encode_to_unicode("상상이상의 ").unwrap(), "⠇⠶⠇⠶⠕⠇⠶⠺");
        assert_eq!(encode_to_unicode("안녕\n반가워").unwrap(), "⠣⠒⠉⠻\n⠘⠒⠫⠏");
        assert_eq!(encode_to_unicode("BMI(지수)").unwrap(), "⠴⠠⠠⠃⠍⠊⠦⠄⠨⠕⠠⠍⠠⠴");
        assert_eq!(encode_to_unicode("지수(BMI)").unwrap(), "⠨⠕⠠⠍⠦⠄⠴⠠⠠⠃⠍⠊⠠⠴");
        assert_eq!(
            encode_to_unicode("체질량 지수(BMI)").unwrap(),
            "⠰⠝⠨⠕⠂⠐⠜⠶⠀⠨⠕⠠⠍⠦⠄⠴⠠⠠⠃⠍⠊⠠⠴"
        );
        assert_eq!(
            encode_to_unicode("Roma [ㄹㄹ로마]").unwrap(),
            "⠴⠠⠗⠕⠍⠁⠲⠀⠦⠆⠸⠂⠸⠂⠐⠥⠑⠰⠴"
        );
        assert_eq!(
            encode_to_unicode("‘ㅖ’로 적는다.").unwrap(),
            "⠠⠦⠿⠌⠴⠄⠐⠥⠀⠨⠹⠉⠵⠊⠲"
        );
        assert_eq!(encode_to_unicode("Contents").unwrap(), "⠠⠒⠞⠢⠞⠎");

        assert_eq!(
            encode_to_unicode("Table of Contents").unwrap(),
            "⠠⠞⠁⠃⠇⠑⠀⠷⠀⠠⠒⠞⠢⠞⠎"
        );
        assert_eq!(encode_to_unicode("bonjour").unwrap(), "⠃⠕⠝⠚⠳⠗");
        assert_eq!(encode_to_unicode("삼각형 ㄱㄴㄷ").unwrap(), "⠇⠢⠫⠁⠚⠻⠀⠿⠁⠿⠒⠿⠔");
        assert_eq!(encode_to_unicode("걲").unwrap(), "⠈⠹⠁");
        assert_eq!(encode_to_unicode("겄").unwrap(), "⠈⠎⠌");
        assert_eq!(encode_to_unicode("kg").unwrap(), "⠅⠛");
        assert_eq!(encode_to_unicode("(kg)").unwrap(), "⠦⠄⠅⠛⠠⠴");
        assert_eq!(
            encode_to_unicode("나루 + 배 = 나룻배").unwrap(),
            "⠉⠐⠍⠀⠢⠀⠘⠗⠀⠒⠒⠀⠉⠐⠍⠄⠘⠗"
        );
        assert_eq!(
            encode_to_unicode("02-2669-9775~6").unwrap(),
            "⠼⠚⠃⠤⠼⠃⠋⠋⠊⠤⠼⠊⠛⠛⠑⠈⠔⠼⠋"
        );
        assert_eq!(
            encode_to_unicode("WELCOME TO KOREA").unwrap(),
            "⠠⠠⠠⠺⠑⠇⠉⠕⠍⠑⠀⠞⠕⠀⠅⠕⠗⠑⠁⠠⠄"
        );
        assert_eq!(encode_to_unicode("SNS에서").unwrap(), "⠴⠠⠠⠎⠝⠎⠲⠝⠠⠎");
        assert_eq!(encode_to_unicode("ATM").unwrap(), "⠠⠠⠁⠞⠍");
        assert_eq!(encode_to_unicode("ATM 기기").unwrap(), "⠴⠠⠠⠁⠞⠍⠲⠀⠈⠕⠈⠕");
        assert_eq!(encode_to_unicode("1,000").unwrap(), "⠼⠁⠂⠚⠚⠚");
        assert_eq!(encode_to_unicode("0.48").unwrap(), "⠼⠚⠲⠙⠓");
        assert_eq!(
            encode_to_unicode("820718-2036794").unwrap(),
            "⠼⠓⠃⠚⠛⠁⠓⠤⠼⠃⠚⠉⠋⠛⠊⠙"
        );
        assert_eq!(
            encode_to_unicode("5개−3개=2개").unwrap(),
            "⠼⠑⠈⠗⠀⠔⠀⠼⠉⠈⠗⠀⠒⠒⠀⠼⠃⠈⠗"
        );
        assert_eq!(encode_to_unicode("소화액").unwrap(), "⠠⠥⠚⠧⠤⠗⠁");
        assert_eq!(encode_to_unicode("X").unwrap(), "⠠⠭");
        assert_eq!(encode_to_unicode("껐").unwrap(), "⠠⠈⠎⠌");
        assert_eq!(encode_to_unicode("TV를").unwrap(), "⠴⠠⠠⠞⠧⠲⠐⠮");
        assert_eq!(encode_to_unicode("껐어요.").unwrap(), "⠠⠈⠎⠌⠎⠬⠲");
        assert_eq!(encode_to_unicode("5운6기").unwrap(), "⠼⠑⠀⠛⠼⠋⠈⠕");
        assert_eq!(encode_to_unicode("끊").unwrap(), "⠠⠈⠵⠴");
        assert_eq!(encode_to_unicode("끊겼어요").unwrap(), "⠠⠈⠵⠴⠈⠱⠌⠎⠬");
        assert_eq!(encode_to_unicode("시예요").unwrap(), "⠠⠕⠤⠌⠬");
        assert_eq!(encode_to_unicode("정").unwrap(), "⠨⠻");
        assert_eq!(encode_to_unicode("나요").unwrap(), "⠉⠣⠬");
        assert_eq!(encode_to_unicode("사이즈").unwrap(), "⠇⠕⠨⠪");
        assert_eq!(encode_to_unicode("청소를").unwrap(), "⠰⠻⠠⠥⠐⠮");
        assert_eq!(encode_to_unicode("것").unwrap(), "⠸⠎");
        assert_eq!(encode_to_unicode("것이").unwrap(), "⠸⠎⠕");
        assert_eq!(encode_to_unicode("이 옷").unwrap(), "⠕⠀⠥⠄");
        assert_eq!(encode_to_unicode(".").unwrap(), "⠲");
        assert_eq!(encode_to_unicode("안").unwrap(), "⠣⠒");
        assert_eq!(encode_to_unicode("안녕").unwrap(), "⠣⠒⠉⠻");
        assert_eq!(encode_to_unicode("안녕하").unwrap(), "⠣⠒⠉⠻⠚");

        assert_eq!(encode_to_unicode("세요").unwrap(), "⠠⠝⠬");

        assert_eq!(encode_to_unicode("하세요").unwrap(), "⠚⠠⠝⠬");
        assert_eq!(encode_to_unicode("안녕하세요").unwrap(), "⠣⠒⠉⠻⠚⠠⠝⠬");
        //                                           ⠣⠒⠉⠻⠚⠠⠕⠃⠉⠕⠠⠈⠣
        assert_eq!(encode_to_unicode("안녕하십니까").unwrap(), "⠣⠒⠉⠻⠚⠠⠕⠃⠉⠕⠠⠫");

        assert_eq!(encode_to_unicode("그래서 작동").unwrap(), "⠁⠎⠀⠨⠁⠊⠿");
        assert_eq!(encode_to_unicode("그래서 작동하나").unwrap(), "⠁⠎⠀⠨⠁⠊⠿⠚⠉");
        //                                               ⠁⠎⠀⠨⠁⠊⠿⠚⠉⠬
        assert_eq!(
            encode_to_unicode("그래서 작동하나요").unwrap(),
            "⠁⠎⠀⠨⠁⠊⠿⠚⠉⠣⠬"
        );
        assert_eq!(
            encode_to_unicode("그래서 작동하나요?").unwrap(),
            "⠁⠎⠀⠨⠁⠊⠿⠚⠉⠣⠬⠦"
        );
        assert_eq!(encode_to_unicode("이 노래").unwrap(), "⠕⠀⠉⠥⠐⠗");
        assert_eq!(encode_to_unicode("아").unwrap(), "⠣");
        assert_eq!(encode_to_unicode("름").unwrap(), "⠐⠪⠢");
        assert_eq!(encode_to_unicode("아름").unwrap(), "⠣⠐⠪⠢");
        // ⠠⠶
        assert_eq!(encode_to_unicode("사").unwrap(), "⠇");
        assert_eq!(encode_to_unicode("상").unwrap(), "⠇⠶");
        assert_eq!(
            encode_to_unicode("아름다운 세상.").unwrap(),
            "⠣⠐⠪⠢⠊⠣⠛⠀⠠⠝⠇⠶⠲"
        );
        assert_eq!(
            encode_to_unicode("모든 것이 무너진 듯해도").unwrap(),
            "⠑⠥⠊⠵⠀⠸⠎⠕⠀⠑⠍⠉⠎⠨⠟⠀⠊⠪⠄⠚⠗⠊⠥"
        );
    }

    #[test]
    pub fn test_by_testcase() {
        let test_cases_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../test_cases");
        let dir = std::fs::read_dir(test_cases_dir).unwrap();
        let mut total = 0;
        let mut failed = 0;
        let mut failed_cases = Vec::new();
        let mut file_stats = std::collections::HashMap::new();
        let files = dir
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().unwrap_or_default() == "csv")
            .collect::<Vec<_>>();

        // read rule_map.json
        let rule_map: HashMap<String, HashMap<String, String>> = serde_json::from_str(
            &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../rule_map.json"))
                .unwrap(),
        )
        .unwrap();

        let rule_map_keys: std::collections::HashSet<String> = rule_map.keys().cloned().collect();
        let file_keys: std::collections::HashSet<_> = files
            .iter()
            .map(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .split('.')
                    .next()
                    .unwrap()
                    .to_string()
            })
            .collect();
        let missing_keys = rule_map_keys.difference(&file_keys).collect::<Vec<_>>();
        let extra_keys = file_keys.difference(&rule_map_keys).collect::<Vec<_>>();
        if !missing_keys.is_empty() || !extra_keys.is_empty() {
            panic!("rule_map.json 파일이 올바르지 않습니다.");
        }

        for path in files {
            let file = File::open(&path).unwrap();
            let filename = path.file_name().unwrap().to_string_lossy();
            let reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(file);

            let mut file_total = 0;
            let mut file_failed = 0;
            // input, expected, actual, is_success
            let mut test_status: Vec<(String, String, String, bool)> = Vec::new();

            for (line_num, result) in reader.into_records().enumerate() {
                total += 1;
                file_total += 1;
                let record = result.unwrap();
                let input = &record[0];
                let expected = record[2].replace(" ", "⠀");
                match encode(input) {
                    Ok(actual) => {
                        let braille_expected = actual
                            .iter()
                            .map(|c| unicode::encode_unicode(*c))
                            .collect::<String>();
                        let actual_str = actual.iter().map(|c| c.to_string()).collect::<String>();
                        if actual_str != expected {
                            failed += 1;
                            file_failed += 1;
                            failed_cases.push((
                                filename.to_string(),
                                line_num + 1,
                                input.to_string(),
                                expected.to_string(),
                                actual_str.clone(),
                                braille_expected.clone(),
                                record[3].to_string(),
                            ));
                        }

                        test_status.push((
                            input.to_string(),
                            record[3].to_string(),
                            braille_expected.clone(),
                            record[3].to_string() == braille_expected,
                        ));
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        failed += 1;
                        file_failed += 1;
                        failed_cases.push((
                            filename.to_string(),
                            line_num + 1,
                            input.to_string(),
                            expected.to_string(),
                            "".to_string(),
                            e.to_string(),
                            record[3].to_string(),
                        ));

                        test_status.push((
                            input.to_string(),
                            record[3].to_string(),
                            e.to_string(),
                            false,
                        ));
                    }
                }
            }
            file_stats.insert(
                path.file_stem().unwrap().to_string_lossy().to_string(),
                (file_total, file_failed, test_status),
            );
        }

        if !failed_cases.is_empty() {
            println!("\n실패한 케이스:");
            println!("=================");
            for (filename, line_num, input, expected, actual, unicode, braille) in failed_cases {
                let diff = {
                    let unicode_words: Vec<&str> = unicode.split(encode_unicode(0)).collect();
                    let braille_words: Vec<&str> = braille.split(encode_unicode(0)).collect();
                    let mut diff = Vec::new();
                    for (i, (u, b)) in unicode_words.iter().zip(braille_words.iter()).enumerate() {
                        if u != b {
                            diff.push(i);
                        }
                    }
                    diff
                };

                let input_words: Vec<&str> = input.split(' ').collect();
                let unicode_words: Vec<&str> = unicode.split(encode_unicode(0)).collect();
                if input_words.len() != unicode_words.len() {
                    println!("파일: {}, 라인 {}: '{}'", filename, line_num, input);
                    println!("  예상: {}", expected);
                    println!("  실제: {}", actual);
                    println!("  유니코드 Result: {}", unicode);
                    println!("  유니코드 Expected: {}", braille);
                } else {
                    let mut colored_input = String::new();
                    let mut colored_unicode = String::new();

                    for (i, word) in input_words.iter().enumerate() {
                        if diff.contains(&i) {
                            colored_input.push_str(&format!("\x1b[31m{}\x1b[0m", word));
                            colored_unicode
                                .push_str(&format!("\x1b[31m{}\x1b[0m", unicode_words[i]));
                        } else {
                            colored_input.push_str(word);
                            colored_unicode.push_str(unicode_words[i]);
                        }
                        if i < input_words.len() - 1 {
                            colored_input.push(' ');
                            colored_unicode.push(' ');
                        }
                    }
                    println!("파일: {}, 라인 {}: '{}'", filename, line_num, colored_input);
                    println!("  예상: {}", expected);
                    println!("  실제: {}", actual);
                    println!("  유니코드 Result: {}", colored_unicode);
                    println!("  유니코드 Expected: {}", braille);
                }
                println!();
            }
        }

        // write test_status to file
        serde_json::to_writer_pretty(
            File::create(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../test_status.json"
            ))
            .unwrap(),
            &file_stats,
        )
        .unwrap();

        println!("\n파일별 테스트 결과:");
        println!("=================");
        for (filename, (file_total, file_failed, _)) in file_stats {
            let success_rate =
                ((file_total - file_failed) as f64 / file_total as f64 * 100.0) as i32;
            let color = if success_rate == 100 {
                "\x1b[32m" // 초록색
            } else if success_rate == 0 {
                "\x1b[31m" // 빨간색
            } else {
                "\x1b[33m" // 주황색
            };
            println!(
                "{}: {}개 중 {}개 성공 (성공률: {}{}%\x1b[0m)",
                filename,
                file_total,
                file_total - file_failed,
                color,
                success_rate
            );
        }
        println!("\n전체 테스트 결과 요약:");
        println!("=================");
        println!("총 테스트 케이스: {}", total);
        println!("성공: {}", total - failed);
        println!("실패: {}", failed);
        if failed > 0 {
            panic!(
                "{}개 중 {}개의 테스트 케이스가 실패했습니다.",
                total, failed
            );
        }
    }

    proptest! {
        #[test]
        fn test_encode_proptest(s: String) {
            let result = encode(&s);
            let _encoded = match result {
                Ok(encoded) => {
                    // Empty result is valid for strings that contain only spaces
                    let is_only_spaces = s.chars().all(|c| c == ' ');
                    assert!(!encoded.is_empty() || s.is_empty() || is_only_spaces);

                    let unicode_result = encode_to_unicode(&s);
                    assert!(unicode_result.is_ok());

                    let unicode_string = unicode_result.unwrap();
                    assert!(!unicode_string.is_empty() || s.is_empty() || is_only_spaces);

                    encoded
                }
                Err(_) => {
                    return Ok(()); // ok
                }
            };

            // let decoded = decode(&encoded);
            // assert_eq!(s, decoded, "Decoded string does not match original input: {}", s);
        }
    }

    #[test]
    fn test_encoder_streaming() {
        // Test encoder can be reused
        let mut encoder = Encoder::new(false); // English only test
        let mut buffer = Vec::new();

        // Encode multiple times with same encoder
        encoder.encode("test", &mut buffer).unwrap();
        encoder.encode("ing", &mut buffer).unwrap();
        encoder.finish(&mut buffer).unwrap();

        // Should produce same result as one-shot
        let expected = encode("testing").unwrap();
        assert_eq!(buffer, expected);
    }
}
