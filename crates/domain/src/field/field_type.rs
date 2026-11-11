/// What kind of value a custom field holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldType {
    Text,
    Number,
    Money,
    Date,
    Boolean,
}
