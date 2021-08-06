use crate::generator::vhdl::*;
use crate::Result;

use super::*;

impl Declare for Entity {
    fn declare(&self) -> Result<String> {
        let mut result = String::new();
        if let Some(doc) = self.doc() {
            result.push_str("--");
            result.push_str(doc.replace("\n", "\n--").as_str());
            result.push('\n');
        }
        result.push_str(format!("entity {} is\n", self.identifier()).as_str());
        result.push_str(self.ports().declare()?.as_str());
        result.push_str(format!("end {};", self.identifier()).as_str());
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::common::test::test_comp;

    use super::*;

    #[test]
    fn entity_declare() {
        let c = Entity::from(test_comp()).with_doc(" My awesome\n Entity".to_string());
        assert_eq!(
            c.declare().unwrap(),
            concat!(
                "-- My awesome
-- Entity
entity test_comp is
  port(
    a_dn : in a_dn_type;
    a_up : out a_up_type;
    b_dn : out b_dn_type;
    b_up : in b_up_type
  );
end test_comp;"
            )
        );
    }
}
