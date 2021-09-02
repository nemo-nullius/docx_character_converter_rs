use crate::common;
use crate::libopencc;
use crate::xml_simp_parser;

#[derive(Debug)]
pub struct DocxXMl<'a> {
    xml_parser: &'a mut xml_simp_parser::XMLSimpParser,
    // { {a, b, c}, {m, n} } a/b/c refers to three indices to which text of one passage is stored separately
    text_index_groups: Vec<Vec<usize>>,
    opencc_config_filepath: &'a str,
}

impl<'a> DocxXMl<'a> {
    pub fn new(
        xml_parser: &'a mut xml_simp_parser::XMLSimpParser,
        opencc_config_filepath: &'a str,
    ) -> Self {
        xml_parser.parse();
        //println!("{:?}", xml_parser);
        DocxXMl {
            xml_parser,
            text_index_groups: Vec::new(),
            opencc_config_filepath,
        }
    }

    pub fn convert(
        &mut self,
        converted_text: &mut String,
        converted_text_html: &mut String,
    ) -> usize {
        let mut converted_zi_sum: usize = 0;
        self.get_text_index_groups();
        converted_zi_sum += self.convert_all_paras(converted_text_html);
        *converted_text = self.xml_parser.counter_parse();
        converted_zi_sum
    }

    pub fn get_zi_sum(&self) -> usize {
        let mut zi_sum: usize = 0;
        // duplicate with DocxXML::get_text_index_groups()
        let mut is_wp: bool = false;
        let mut is_wt: bool = false;
        for stru in &self.xml_parser.tokenized_text_list {
            match stru.token {
                common::XMLSimpToken::TagWpBeg => is_wp = true,
                common::XMLSimpToken::TagWpEnd => is_wp = false,
                common::XMLSimpToken::TagWtBeg => is_wt = true,
                common::XMLSimpToken::TagWtEnd => is_wt = false,
                common::XMLSimpToken::Text => {
                    if is_wp && is_wt {
                        zi_sum += stru.text.chars().collect::<Vec<char>>().len();
                    }
                }
                _ => {}
            }
        }
        zi_sum
    }

    // 1
    fn get_text_index_groups(&mut self) {
        let mut one_group: Vec<usize> = Vec::new(); // indices to text of one paragraph
        let mut is_wp: bool = false;
        let mut is_wt: bool = false;
        for i in 0..self.xml_parser.tokenized_text_list.len() {
            match self.xml_parser.tokenized_text_list[i].token {
                common::XMLSimpToken::TagWpBeg => is_wp = true,
                common::XMLSimpToken::TagWpEnd => {
                    // ==0 with wp but without wt
                    if one_group.len() != 0 {
                        self.text_index_groups.push(one_group.clone())
                    }
                    one_group = Vec::new();
                    is_wp = false;
                }
                common::XMLSimpToken::TagWtBeg => is_wt = true,
                common::XMLSimpToken::TagWtEnd => is_wt = false,
                common::XMLSimpToken::Text => {
                    if is_wp && is_wt {
                        one_group.push(i);
                    }
                }
                _ => {}
            }
        }
    }

    fn convert_one_para(
        &mut self,
        vec: &Vec<usize>,
        one_para_converted_html: &mut String,
    ) -> usize {
        let mut one_para: String = String::new();
        for index in vec {
            one_para = format!(
                "{}{}",
                one_para, self.xml_parser.tokenized_text_list[*index].text
            );
        }
        /*
        let one_para: String = self // accumulate tokenzied_text_list: [TokenziedText.text]
            .xml_parser
            .tokenized_text_list
            .iter()
            .map(|a| &a.text)
            .fold(String::from(""), |acc, x| acc + &x);
            */
        /* which is quicker?
        .iter()
        .map(|a| (&a.text).to_owned())
        .collect::<Vec<String>>()
        .join("");
        */
        // convert by opencc
        let one_para_converted_with_marker =
            libopencc::convert(self.opencc_config_filepath, &one_para);
        ////println!("{}\n----\n{}\n", &one_para, &one_para_converted_with_marker);
        // process opencc result
        let mut one_para_converted: String = String::new();
        let converted_one_para_zi_sum: usize = self.split_converted_result(
            &one_para_converted_with_marker,
            &mut one_para_converted,
            one_para_converted_html, // why no mut?
        );
        // fill the converted text in the original XML file
        let mut offset: usize = 0;
        let one_para_converted_chars: Vec<char> = one_para_converted.chars().collect();
        for i in 0..vec.len() {
            let index: usize = vec[i];
            let original_chars_len = self.xml_parser.tokenized_text_list[index]
                .text
                .chars()
                .collect::<Vec<char>>()
                .len();
            if i == vec.len() - 1 {
                // the last wt - for the new chars len might be different from original_chars_len
                self.xml_parser.tokenized_text_list[index].text =
                    one_para_converted_chars[offset..].into_iter().collect();
            } else {
                self.xml_parser.tokenized_text_list[index].text = one_para_converted_chars
                    [offset..offset + original_chars_len] // original BUG
                    .into_iter()
                    .collect();
            }
            offset += original_chars_len;
        }
        converted_one_para_zi_sum
    }

    // 2
    fn convert_all_paras(&mut self, html_all_paras: &mut String) -> usize {
        let mut converted_all_paras_zi_sum: usize = 0;
        // optimize? - to_own()
        for vec in self.text_index_groups.to_owned() {
            let mut one_para_html: String = String::new();
            converted_all_paras_zi_sum += self.convert_one_para(&vec, &mut one_para_html);
            *html_all_paras = format!("{}{}{}{}", html_all_paras, "<p>", one_para_html, "</p>\n");
        }
        converted_all_paras_zi_sum
    }

    // return zi_sum of converted text
    fn split_converted_result(
        &self,
        converted_text_with_marker: &str,
        converted_text: &mut String,
        converted_html: &mut String,
    ) -> usize {
        // converted result: (.\d)*
        let converted_chars_with_marker: Vec<char> = converted_text_with_marker.chars().collect();
        for i in 0..(converted_chars_with_marker.len() / 2) {
            let idx = i * 2;
            let c: char = converted_chars_with_marker[idx];
            let marker: u32 = converted_chars_with_marker[idx + 1].to_digit(10).unwrap(); // ATTENTION: marker must lt 10
            converted_text.push(converted_chars_with_marker[idx]);
            if marker > 1 {
                *converted_html = format!(
                    "{}{}{}{}",
                    *converted_html, "<span style=\"color: #ff0000\">", c, "◣</span>"
                );
            } else {
                *converted_html = format!("{}{}", *converted_html, c);
            }
        }
        converted_chars_with_marker.len() / 2
    }
}
