/// Integration tests using the VHDL back-end.
extern crate tydi;

use tydi::generator::vhdl::Declare;
use tydi::generator::Componentify;
use tydi::Name;
use tydi::UniquelyNamedBuilder;

#[test]
fn streamlet_async() {
    let (_, streamlet) =
        tydi::parser::nom::streamlet("Streamlet test (a : in Bits<1>, b : out Bits<2>)").unwrap();
    assert_eq!(
        streamlet.canonical(None).declare().unwrap(),
        "component test
  port(
    a : in std_logic_vector(0 downto 0);
    b : out std_logic_vector(1 downto 0)
  );
end component;"
    );
    assert_eq!(
        streamlet.user(None).unwrap().declare().unwrap(),
        "component test
  port(
    a : in std_logic_vector(0 downto 0);
    b : out std_logic_vector(1 downto 0)
  );
end component;"
    );
}

#[test]
fn streamlet_async_nested() {
    let (_, streamlet) = tydi::parser::nom::streamlet(
        "Streamlet test (a : in Group<b: Bits<1>, c: Bits<2>>, d : out Bits<1>)",
    )
    .unwrap();
    let lib = tydi::design::library::Library::from_builder(
        Name::try_new("test").unwrap(),
        UniquelyNamedBuilder::new().with_items(vec![streamlet]),
    );

    let lib: tydi::generator::common::Library = lib.unwrap().into();
    assert_eq!(
        lib.declare().unwrap(),
        "package test is

component test_com
  port(
    a_b : in std_logic_vector(0 downto 0);
    a_c : in std_logic_vector(1 downto 0);
    d : out std_logic_vector(0 downto 0)
  );
end component;

record test_a_type
  b : std_logic_vector(0 downto 0);
  c : std_logic_vector(1 downto 0);
end record;

component test
  port(
    a : in test_a_type;
    d : out std_logic_vector(0 downto 0)
  );
end component;

end test;"
    );
}

#[test]
fn streamlet_streams() {
    let (_, streamlet) = tydi::parser::nom::streamlet(
        "Streamlet test (a : in Stream<Bits<1>>, b : out Stream<Bits<2>, d=2>)",
    )
    .unwrap();
    let lib = tydi::design::library::Library::from_builder(
        Name::try_new("test").unwrap(),
        UniquelyNamedBuilder::new().with_items(vec![streamlet]),
    );

    let lib: tydi::generator::common::Library = lib.unwrap().into();
    assert_eq!(
        lib.declare().unwrap(),
        "package test is

component test_com
  port(
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(0 downto 0);
    b_valid : out std_logic;
    b_ready : in std_logic;
    b_data : out std_logic_vector(1 downto 0);
    b_last : out std_logic_vector(1 downto 0);
    b_strb : out std_logic_vector(0 downto 0)
  );
end component;

record test_a_type
  valid : std_logic;
  ready : std_logic;
  data : std_logic_vector(0 downto 0);
end record;

record test_b_type
  valid : std_logic;
  ready : std_logic;
  data : std_logic_vector(1 downto 0);
  last : std_logic_vector(1 downto 0);
  strb : std_logic_vector(0 downto 0);
end record;

component test
  port(
    a : in test_a_type;
    b : out test_b_type
  );
end component;

end test;"
    );
}

#[test]
fn streamlet_group_async_streams() {
    let (_, streamlet) = tydi::parser::nom::streamlet(
        "Streamlet test (a : in Group<b:Bits<2>, c:Stream<Bits<1>>>, d : out Stream<Bits<1>>)",
    )
    .unwrap();
    let lib = tydi::design::library::Library::from_builder(
        Name::try_new("test").unwrap(),
        UniquelyNamedBuilder::new().with_items(vec![streamlet]),
    );

    let lib: tydi::generator::common::Library = lib.unwrap().into();
    assert_eq!(
        lib.declare().unwrap(),
        "package test is

component test_com
  port(
    a_b : in std_logic_vector(1 downto 0);
    a_c_valid : in std_logic;
    a_c_ready : out std_logic;
    a_c_data : in std_logic_vector(0 downto 0);
    d_valid : out std_logic;
    d_ready : in std_logic;
    d_data : out std_logic_vector(0 downto 0)
  );
end component;

record test_a_type
  b : std_logic_vector(1 downto 0);
end record;

record test_a_c_type
  valid : std_logic;
  ready : std_logic;
  data : std_logic_vector(0 downto 0);
end record;

record test_d_type
  valid : std_logic;
  ready : std_logic;
  data : std_logic_vector(0 downto 0);
end record;

component test
  port(
    a : in test_a_type;
    a_c : in test_a_c_type;
    d : out test_d_type
  );
end component;

end test;"
    );
}

#[test]
fn streamlet_async_all() {
    let (_, streamlet) = tydi::parser::nom::streamlet(
        "Streamlet test (
            a : in Null,
            b : in Bits<1>,
            c : in Group<d:Bits<1>, e:Bits<2>>,
            f : in Union<g:Null, h:Bits<3>>,
            i : out Group<q:Null, r:Bits<1>, s:Group<t:Bits<1>, u:Bits<2>>, v:Union<g:Null, w:Bits<3>>>
        )",
    )
    .unwrap();
    let lib = tydi::design::library::Library::from_builder(
        Name::try_new("test").unwrap(),
        UniquelyNamedBuilder::new().with_items(vec![streamlet]),
    );

    let lib: tydi::generator::common::Library = lib.unwrap().into();
    assert_eq!(
        lib.declare().unwrap(),
        "package test is

component test_com
  port(
    b : in std_logic_vector(0 downto 0);
    c_d : in std_logic_vector(0 downto 0);
    c_e : in std_logic_vector(1 downto 0);
    f_tag : in std_logic_vector(0 downto 0);
    f_h : in std_logic_vector(2 downto 0);
    i_r : out std_logic_vector(0 downto 0);
    i_s_t : out std_logic_vector(0 downto 0);
    i_s_u : out std_logic_vector(1 downto 0);
    i_v_tag : out std_logic_vector(0 downto 0);
    i_v_w : out std_logic_vector(2 downto 0)
  );
end component;

record test_c_type
  d : std_logic_vector(0 downto 0);
  e : std_logic_vector(1 downto 0);
end record;

record test_f_type
  tag : std_logic_vector(0 downto 0);
  h : std_logic_vector(2 downto 0);
end record;

record test_i_type
  r : std_logic_vector(0 downto 0);
  s : test_i_s_type;
  v : test_i_v_type;
end record;

record test_i_s_type
  t : std_logic_vector(0 downto 0);
  u : std_logic_vector(1 downto 0);
end record;

record test_i_v_type
  tag : std_logic_vector(0 downto 0);
  w : std_logic_vector(2 downto 0);
end record;

component test
  port(
    b : in std_logic_vector(0 downto 0);
    c : in test_c_type;
    f : in test_f_type;
    i : out test_i_type
  );
end component;

end test;"
    );
}
