use super::{AliasDeclaration, ArchitectureDeclaration, ObjectDeclaration};

impl From<ObjectDeclaration> for ArchitectureDeclaration<'_> {
    fn from(object: ObjectDeclaration) -> Self {
        ArchitectureDeclaration::Object(object)
    }
}

impl<'a> From<AliasDeclaration<'a>> for ArchitectureDeclaration<'a> {
    fn from(alias: AliasDeclaration<'a>) -> Self {
        ArchitectureDeclaration::Alias(alias)
    }
}
