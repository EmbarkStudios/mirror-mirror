use core::net::IpAddr;

use crate::DescribeType;
use crate::type_info::{Variant,Type,ScalarType};

#[test]
fn ipaddress_type_info() {
    let type_info = <IpAddr as DescribeType>::type_descriptor();
    assert_eq!(type_info.get_type().type_name(), "core::net::ip_addr::IpAddr");
    let enum_ = type_info.as_enum().unwrap();
    assert_eq!(enum_.variants_len(), 2);

    
    let ipv4 = match enum_.variant("V4").unwrap() {
        Variant::Tuple(x) => x,
        y => panic!("incorrect variant type: '{:?}'", y),
    };
    assert_eq!(ipv4.fields_len(), 1);
    let inner = match ipv4.field_type_at(0).unwrap().get_type() {
        Type::Scalar(ScalarType::Ipv4Addr) => { },
        y => panic!("incorrect interior type: '{:?}'", y),
    };

    let ipv6 = match enum_.variant("V6").unwrap() {
        Variant::Tuple(x) => x,
        y => panic!("incorrect variant type: '{:?}'", y),
    };
    assert_eq!(ipv6.fields_len(), 1);
    let inner = match ipv6.field_type_at(0).unwrap().get_type() {
        Type::Scalar(ScalarType::Ipv6Addr) => { },
        y => panic!("incorrect interior type: '{:?}'", y),
    };
}
