/// Who sends or receives a letter: a name and the lines of the postal address, which may be empty.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct LetterParty {
    pub name: String,
    pub address_lines: Vec<String>,
}
