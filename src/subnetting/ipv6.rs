use std::net::{Ipv6Addr, AddrParseError};
use std::fmt;
use std::str::FromStr;
use std::num::ParseIntError;

use subnetting::apply_netmask::{ApplyNetmask, ApplyNetmaskError};

/// Represents a range of ipv6 addresses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv6Subnet {
    base_addr: Ipv6Addr,
    netmask_bits: u8,
}

quick_error! {
    /// Errors raised by `Ipv6Subnet::parse`
    #[derive(Debug)]
    pub enum Ipv6SubnetParseError {
        /// Error parsing Ipv6Subnet in CIDR notation a.b.c.d/n
        InvalidFormat {
            description("Error parsing Ipv6Subnet in CIDR notation a:b:c:d:e:f:g:h/n")
        }
        /// Error parsing base ipv6 address
        ParseAddress {
            err: AddrParseError,
        } {
            description("Error parsing base ipv6 address.")
            display("Error parsing base ipv6 address: {}", err)
            cause(err)
        }
        /// Error parsing netmask bits
        ParseNetmask {
            err: ParseIntError,
        } {
            description("Error parsing netmask bits")
            display("Error parsing netmask bits: {}", err)
            cause(err)
        }
        InvalidSubnet {
            err: Ipv6SubnetNewError,
        } {
            description("Invalid subnet")
            display("Invalid subnet: {}", err)
            cause(err)
        }
    }
}

quick_error! {
    /// Errors returned by `Ipv4Subnet::new`
    #[derive(Debug)]
    pub enum Ipv6SubnetNewError {
        /// Netmask prefix length out of range
        NetmaskOutOfRange {
            netmask_bits: u8,
        } {
            description("Netmask prefix length out of range")
            display("Netmask prefix length out of range. Netmask prefix length must be <= 128 but \
                     got {}", netmask_bits)
        }
        /// Subnet base address contains ones not covered by netmask
        TrailingOnes {
            base_addr: Ipv6Addr,
            netmask_bits: u8,
        } {
            description("Subnet base address contains ones not covered by netmask")
            display("Subnet base address contains ones not covered by netmask. {}/{} is not a \
                     valid subnet. Perhaps you meant {}/{}",
                     base_addr, netmask_bits,
                     unwrap_result!(base_addr.apply_netmask(*netmask_bits)), netmask_bits
            )
        }
    }
}

impl Ipv6Subnet {
    /// Create a new `Ipv6Subnet` with the given base address and netmask prefix length.
    pub fn new(base_addr: Ipv6Addr, netmask_bits: u8) -> Result<Ipv6Subnet, Ipv6SubnetNewError> {
        let masked = match base_addr.apply_netmask(netmask_bits) {
            Ok(masked) => masked,
            Err(ApplyNetmaskError::OutOfRange(n, _))
                => return Err(Ipv6SubnetNewError::NetmaskOutOfRange { netmask_bits: n }),
        };
        if masked != base_addr {
            return Err(Ipv6SubnetNewError::TrailingOnes {
                base_addr: base_addr,
                netmask_bits: netmask_bits,
            });
        }
        Ok(Ipv6Subnet {
            base_addr: base_addr,
            netmask_bits: netmask_bits,
        })
    }

    /// Test whether the subnet contains the given IP address.
    pub fn contains(&self, addr: &Ipv6Addr) -> bool {
        unwrap_result!(addr.apply_netmask(self.netmask_bits)) == self.base_addr
    }

    /// Returns the loopback subnet ::1/128
    pub fn loopback() -> Ipv6Subnet {
        Ipv6Subnet {
            base_addr: Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),
            netmask_bits: 128,
        }
    }

    /// Returns the link-local subnet fe80::/10
    pub fn link_local() -> Ipv6Subnet {
        Ipv6Subnet {
            base_addr: Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0),
            netmask_bits: 10,
        }
    }

    /// Returns the multicast subnet ff00::/8
    pub fn multicast() -> Ipv6Subnet {
        Ipv6Subnet {
            base_addr: Ipv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 0),
            netmask_bits: 8,
        }
    }
}

impl FromStr for Ipv6Subnet {
    type Err = Ipv6SubnetParseError;

    fn from_str(s: &str) -> Result<Ipv6Subnet, Ipv6SubnetParseError> {
        let mut iter = s.split('/');
        match (iter.next(), iter.next(), iter.next()) {
            (Some(addr), Some(netmask_bits), None) => {
                let addr: Ipv6Addr = match addr.parse() {
                    Ok(addr) => addr,
                    Err(e) => return Err(Ipv6SubnetParseError::ParseAddress { err: e }),
                };
                let netmask_bits = match netmask_bits.parse() {
                    Ok(netmask_bits) => netmask_bits,
                    Err(e) => return Err(Ipv6SubnetParseError::ParseNetmask { err: e }),
                };
                let subnet = match Ipv6Subnet::new(addr, netmask_bits) {
                    Ok(subnet) => subnet,
                    Err(e) => return Err(Ipv6SubnetParseError::InvalidSubnet { err: e }),
                };
                Ok(subnet)
            },
            _ => Err(Ipv6SubnetParseError::InvalidFormat),
        }
    }
}

impl fmt::Display for Ipv6Subnet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.base_addr, self.netmask_bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    #[test]
    fn parse_display_subnet() {
        let as_str = "fe80::/16";
        let subnet: Ipv6Subnet = unwrap_result!(as_str.parse());
        let displayed = format!("{}", subnet);
        assert_eq!(displayed, as_str);
    }

    #[test]
    fn parse_bad_subnets() {
        match Ipv6Subnet::from_str("foo") {
            Err(Ipv6SubnetParseError::InvalidFormat) => (),
            x => panic!("Unexpected result {:?}", x),
        };

        match Ipv6Subnet::from_str("foo/bar/baz") {
            Err(Ipv6SubnetParseError::InvalidFormat) => (),
            x => panic!("Unexpected result {:?}", x),
        };

        match Ipv6Subnet::from_str("loopback/23") {
            Err(Ipv6SubnetParseError::InvalidFormat) => (),
            Err(Ipv6SubnetParseError::ParseAddress { .. }) => (),
            x => panic!("Unexpected result {:?}", x),
        };

        match Ipv6Subnet::from_str("::1/twenty_three") {
            Err(Ipv6SubnetParseError::InvalidFormat) => (),
            Err(Ipv6SubnetParseError::ParseNetmask { .. }) => (),
            x => panic!("Unexpected result {:?}", x),
        };

        match Ipv6Subnet::from_str("::1/127") {
            Err(Ipv6SubnetParseError::InvalidSubnet {
                err: Ipv6SubnetNewError::TrailingOnes { .. },
            }) => (),
            x => panic!("Unexpected result {:?}", x),
        };

        match Ipv6Subnet::from_str("::1/129") {
            Err(Ipv6SubnetParseError::InvalidSubnet {
                err: Ipv6SubnetNewError::NetmaskOutOfRange { .. }
            }) => (),
            x => panic!("Unexpected result {:?}", x),
        };
    }
}


