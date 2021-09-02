#[derive(Debug)]
pub enum XMLSimpToken {
    TagProlog,
    TagWpBeg,
    TagWpEnd,
    TagWtBeg,
    TagWtEnd,
    TagOtherBeg,
    TagOtherEnd,
    TagOtherBegEnd,
    Text,
}

#[derive(Debug)]
pub struct TokenizedText {
    pub token: XMLSimpToken,
    pub text: String,
}
