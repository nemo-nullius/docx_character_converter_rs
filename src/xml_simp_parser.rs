//#[path = "common.rs"]
//mod common;

use crate::common;

#[derive(Debug)]
pub struct XMLSimpParser {
    xml: Vec<char>,
    p: usize,
    xml_buf: Vec<char>,
    pub tokenized_text_list: Vec<common::TokenizedText>, // why need pub?
}

impl XMLSimpParser {
    pub fn new(s: &String) -> Self {
        XMLSimpParser {
            xml: s.chars().collect(),
            p: 0,
            xml_buf: Vec::new(),
            tokenized_text_list: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> &Vec<common::TokenizedText> {
        self.simp_parse();
        return &self.tokenized_text_list;
    }

    pub fn counter_parse(&self) -> String {
        return self.simp_combine();
    }

    fn eat(&mut self) -> char {
        let val = self.xml[self.p];
        self.p += 1;
        return val;
    }

    fn simp_parse(&mut self) {
        loop {
            let c = self.eat();
            if self.p == self.xml.len() {
                self.xml_buf.push(c);
                self.tokenize();
                self.xml_buf = Vec::new();
                return;
            }
            match c {
                '<' => {
                    self.tokenize();
                    self.xml_buf = vec![c];
                }
                '>' => {
                    self.xml_buf.push(c);
                    self.tokenize();
                    self.xml_buf = Vec::new();
                }
                _ => self.xml_buf.push(c),
            }
        }
    }

    fn tokenize(&mut self) {
        if self.xml_buf.len() == 0 {
            return;
        }
        let bufs: String = (&self.xml_buf).into_iter().collect();
        let buf = &self.xml_buf;
        let token = if bufs.find('<') != Some(0) {
            common::XMLSimpToken::Text
        } else if buf.len() >= 4 && bufs.find("<?") == Some(0) {
            // <?...?>
            common::XMLSimpToken::TagProlog
        } else if bufs == "</w:p>" {
            common::XMLSimpToken::TagWpEnd
        } else if bufs == "</w:t>" {
            common::XMLSimpToken::TagWtEnd
        } else if buf.len() >= 3 && buf[buf.len() - 1] == '>' && buf[buf.len() - 2] == '/' {
            // </> <.../>
            common::XMLSimpToken::TagOtherBegEnd
        } else if buf.len() >= 3 && bufs.find("</") == Some(0) {
            // </...>
            common::XMLSimpToken::TagOtherEnd
        } else if bufs == "<w:p>" || bufs.find("<w:p ") == Some(0) {
            common::XMLSimpToken::TagWpBeg
        } else if bufs == "<w:t>" || bufs.find("<w:t ") == Some(0) {
            common::XMLSimpToken::TagWtBeg
        } else {
            common::XMLSimpToken::TagOtherBeg
        };
        self.tokenized_text_list
            .push(common::TokenizedText { token, text: bufs });
    }

    fn simp_combine(&self) -> String {
        let mut content = String::new();
        for stru in &self.tokenized_text_list {
            content += &stru.text;
        }
        content
    }
}
