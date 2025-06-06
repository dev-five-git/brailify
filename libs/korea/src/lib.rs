use jauem::choseong::encode_choseong;
use moeum::jungsong::encode_jungsong;
use utils::has_choseong_o;

use crate::{
    char_struct::CharType,
    jauem::jongseong::encode_jongseong,
    korean_char::encode_korean_char,
    rule::{rule_11, rule_12},
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
mod split;
mod symbol_shortcut;
mod unicode;
mod utils;
mod word_shortcut;

pub fn encode(text: &str) -> Result<Vec<u8>, String> {
    let mut result: Vec<u8> = Vec::new();
    let words = text.split_whitespace().collect::<Vec<&str>>();
    let word_count = words.len();
    let mut is_english = false;
    // 한국어가 존재할 경우 english_indicator 가 true 가 됩니다.
    let english_indicator = words.iter().any(|word| {
        word.chars().any(|c| {
            return (c as u32 >= 0x3131 && c as u32 <= 0x3163)
                || (0xAC00 <= c as u32 && c as u32 <= 0xD7A3);
        })
    });
    let mut triple_big_english = false;

    for (idx, word) in words.iter().enumerate() {
        if let Some((_, code, rest)) = word_shortcut::split_word_shortcut(word) {
            result.extend(code);
            if !rest.is_empty() {
                result.extend(encode(rest.as_str())?);
            }
        } else {
            let word_chars = word.chars().collect::<Vec<char>>();
            let word_len = word_chars.len();
            let is_all_uppercase = word_chars.iter().all(|c| c.is_uppercase());
            let has_korean_char = word_chars
                .iter()
                .any(|c| (0xAC00 <= *c as u32 && *c as u32 <= 0xD7A3));

            if english_indicator && !is_english && word_chars[0].is_ascii_alphabetic() {
                // 제31항 국어 문장 안에 그리스 문자가 나올 때에는 그 앞에 로마자표 ⠴을 적고 그 뒤에 로마자 종료표 ⠲을 적는다
                result.push(52);
            }

            if is_all_uppercase && !triple_big_english {
                if (idx == 0 || !words[idx - 1].chars().all(|c| c.is_ascii_alphabetic()))
                    && word_count - idx > 2
                    && words[idx + 1].chars().all(|c| c.is_ascii_alphabetic())
                    && words[idx + 2].chars().all(|c| c.is_ascii_alphabetic())
                {
                    triple_big_english = true;
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
                let char_type = CharType::new(*c)?;

                if english_indicator && i > 0 && !c.is_ascii_alphabetic() {
                    // 제31항 국어 문장 안에 그리스 문자가 나올 때에는 그 앞에 로마자표 ⠴을 적고 그 뒤에 로마자 종료표 ⠲을 적는다
                    if is_english {
                        result.push(50);
                    }
                    is_english = false;
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
                            rule_11(&korean, word_chars[i + 1], &mut result)?;
                            rule_12(&korean, word_chars[i + 1], &mut result)?;
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
                                if i == 0 && word_len > 1 && word_chars[1] == '자' {
                                    // 8항 - 단독으로 쓰인 자모
                                    result.push(63);
                                    result.extend(jauem::jongseong::encode_jongseong(c)?);
                                } else {
                                    if has_korean_char {
                                        // 10항 - 단독으로 쓰인 자음자가 단어에 붙어 나올 때
                                        result.push(56);
                                        result.extend(korean_part::encode_korean_part(c)?);
                                    } else {
                                        // 8항 - 단독으로 쓰인 자모
                                        result.push(63);
                                        result.extend(jauem::jongseong::encode_jongseong(c)?);
                                    }
                                }
                            }
                        }
                    }
                    CharType::English(c) => {
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
                        is_english = true;
                        result.push(english::encode_english(c)?);
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
                    CharType::Space => {
                        result.push(0);
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
                        if i < word_len - 1
                            && word_chars[i..]
                                .iter()
                                .any(|c| (0xAC00 <= *c as u32 && *c as u32 <= 0xD7A3))
                        {
                            result.push(0);
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

        if triple_big_english {
            if !(word_count - idx > 1 && words[idx + 1].chars().all(|c| c.is_ascii_alphabetic())) {
                // 28항 [붙임] 로마자가 한 글자만 대문자일 때에는 대문자 기호표 ⠠을 그 앞에 적고, 단어 전체가 대문자이거나 두 글자 이상 연속해서 대문자일 때에는 대문자 단어표
                // ⠠⠠을 그 앞에 적는다. 세 개 이상의 연속된 단어가 모두 대문자일 때에는 첫 단어
                // 앞에 대문자 구절표 ⠠⠠⠠을 적고, 마지막 단어 뒤에 대문자 종료표 ⠠⠄을 적는다.
                result.push(32);
                result.push(4);
            }
        }
        if idx != word_count - 1 {
            if english_indicator && !words[idx + 1].chars().next().unwrap().is_ascii_alphabetic() {
                // 제31항 국어 문장 안에 그리스 문자가 나올 때에는 그 앞에 로마자표 ⠴을 적고 그 뒤에 로마자 종료표 ⠲을 적는다
                if is_english {
                    result.push(50);
                }
                is_english = false;
            }

            result.push(0);
        }
    }
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
pub fn decode(text: &str) -> String {
    text.to_string()
}

#[cfg(test)]
mod test {
    use std::fs::File;

    use crate::unicode::encode_unicode;

    use super::*;
    #[test]
    pub fn test_encode() {
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

        for entry in dir {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().unwrap_or_default() == "csv" {
                let file = File::open(&path).unwrap();
                let reader = csv::ReaderBuilder::new()
                    .has_headers(false)
                    .from_reader(file);

                let filename = path.file_name().unwrap().to_string_lossy();
                let mut file_total = 0;
                let mut file_failed = 0;

                for (line_num, result) in reader.into_records().enumerate() {
                    total += 1;
                    file_total += 1;
                    let record = result.unwrap();
                    let input = &record[0];
                    let expected = record[2].replace(" ", "⠀");
                    match encode(input) {
                        Ok(actual) => {
                            if actual.iter().map(|c| c.to_string()).collect::<String>() != expected
                            {
                                failed += 1;
                                file_failed += 1;
                                failed_cases.push((
                                    filename.to_string(),
                                    line_num + 1,
                                    input.to_string(),
                                    expected.to_string(),
                                    actual.iter().map(|c| c.to_string()).collect::<String>(),
                                    encode_to_unicode(input).unwrap(),
                                    record[3].to_string(),
                                ));
                            }
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
                        }
                    }
                }
                file_stats.insert(filename.to_string(), (file_total, file_failed));
            }
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
        println!("\n파일별 테스트 결과:");
        println!("=================");
        for (filename, (file_total, file_failed)) in file_stats {
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
}
