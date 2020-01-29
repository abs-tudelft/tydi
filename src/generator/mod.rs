//! Source generator module.

use vhdl::{Component, Mode, Port, Type};

use crate::{
    phys::{BitField, Dir, Stream},
    Streamlet,
};
use crate::generator::vhdl::{Record};

pub mod vhdl;

fn log2ceil(v: usize) -> usize { (v as f64).log2().ceil() as usize }

impl From<BitField> for Type {
    fn from(b: BitField) -> Self {
        if b.width() == 1 {
            Type::Bit
        } else {
            Type::BitVec {
                width: b.width()
            }
        }
    }
}

impl From<Dir> for Mode {
    fn from(d: Dir) -> Self {
        if d == Dir::Downstream {
            Mode::Out
        } else {
            Mode::In
        }
    }
}

impl From<Vec<BitField>> for Type {
    fn from(_: Vec<BitField>) -> Self {
        unimplemented!()
    }
}

impl From<Stream> for Vec<Port> {
    fn from(p: Stream) -> Self {
        // TODO(johanpel): insert complexity level selections
        let name: String = p.name_parts.join("_");

        // Up and downstream up
        let mut up = Record::new(format!("{}_up_type", name), vec![]);
        let mut dn = Record::new(format!("{}_dn_type", name), vec![]);

        // Valid/Ready handshake ports
        up.add_field("ready", Type::Bit);
        dn.add_field("valid", Type::Bit);

        // Data elements
        dn.add_field("data", Type::bitvec(p.elements_per_transfer * p.fields.width_recursive()));

        // Transfer metadata
        dn.add_field("stai", Type::BitVec { width: log2ceil(p.elements_per_transfer) });
        dn.add_field("endi", Type::BitVec { width: log2ceil(p.elements_per_transfer) });
        dn.add_field("strb", Type::BitVec { width: p.elements_per_transfer });

        // Dimensional data
        dn.add_field("last", Type::BitVec { width: p.dimensionality });
        dn.add_field("empty", Type::Bit);

        // User data
        dn.add_field("user", Type::Bit);

        vec![Port::new(format!("{}_{}", name, "dn"), p.dir.into(), Type::Record(dn)),
             Port::new(format!("{}_{}", name, "up"), p.dir.reversed().into(), Type::Record(up))]
    }
}

impl From<Streamlet> for Component {
    fn from(s: Streamlet) -> Self {
        let mut result = Component {
            identifier: s.identifier,
            generics: vec![],
            ports: vec![],
        };
        // Obtain the physicals streams of each river.
        for i in s.inputs {
            let phys_streams: Vec<Stream> = i.as_phys(vec![i.identifier().unwrap()]);
            for ps in phys_streams.into_iter() {
                let ps_ports: Vec<Port> = ps.into();
                result.ports.extend(ps_ports.into_iter());
            }
        }
        for o in s.outputs {
            let phys_streams: Vec<Stream> = o.as_phys(vec![o.identifier().unwrap()]);
            for ps in phys_streams.into_iter() {
                let ps_ports: Vec<Port> = ps.into();
                result.ports.extend(ps_ports.into_iter());
            }
        }
        result
    }
}

#[cfg(test)]
mod test {
    use crate::generator::vhdl::{Component, Declare, Package, Port};
    use crate::parser::streamlet::streamlet_interface_definition;
    use crate::phys::{BitField, Complexity, Dir, Stream};
    use crate::Streamlet;

    #[test]
    fn test_from_phys_to_ports() {
        let p = Stream {
            name_parts: vec!["test".to_string(), "phys".to_string()],
            fields: BitField {
                identifier: None,
                width: 0,
                children: vec![
                    BitField::new(Some("a".to_string()), 1),
                    BitField::new(Some("b".to_string()), 2)],
            },
            elements_per_transfer: 1,
            dimensionality: 0,
            dir: Dir::Downstream,
            complexity: Complexity::default(),
        };
        let vp: Vec<Port> = p.into();
        dbg!(vp);
    }

    #[test]
    fn test_simple_streamlet() {
        let streamlet = streamlet_interface_definition(r#"MuhStreamlet

a: Bits<1>
b: Bits<2>

c: Group<Bits<3>, Bits<4>>
d: Bits<4>"#);
        dbg!(&streamlet);
        let streamlet: Streamlet = streamlet.unwrap().1;
        let mut comp: Component = streamlet.into();
        comp.flatten_types();
        let pkg = Package {
            identifier: "Tydi".to_string(),
            components: vec![comp],
        };
        let code: String = pkg.declare();
        println!("{}", code);
    }
}
