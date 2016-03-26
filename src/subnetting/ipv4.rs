use std::net::{Ipv4Addr, AddrParseError};
use std::fmt;
use std::str::FromStr;
use std::num::ParseIntError;

use subnetting::apply_netmask::{ApplyNetmask, ApplyNetmaskError};

/// Represents a range of ipv4 addresses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv4Subnet {
    base_addr: Ipv4Addr,
    netmask_bits: u8,
}

quick_error! {
    /// Errors raised by `Ipv4Subnet::parse`
    #[derive(Debug)]
    pub enum Ipv4SubnetParseError {
        /// Error parsing Ipv4Subnet in CIDR notation a.b.c.d/n
        InvalidFormat {
            description("Error parsing Ipv4Subnet in CIDR notation a.b.c.d/n")
        }
        /// Error parsing base ipv4 address
        ParseAddress {
            err: AddrParseError,
        } {
            description("Error parsing base ipv4 address.")
            display("Error parsing base ipv4 address: {}", err)
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
            err: Ipv4SubnetNewError,
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
    pub enum Ipv4SubnetNewError {
        /// Netmask prefix length out of range
        NetmaskOutOfRange {
            netmask_bits: u8,
        } {
            description("Netmask prefix length out of range")
            display("Netmask prefix length out of range. Netmask prefix length must be <= 32 but \
                     got {}", netmask_bits)
        }
        /// Subnet base address contains ones not covered by netmask
        TrailingOnes {
            base_addr: Ipv4Addr,
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

impl Ipv4Subnet {
    /// Create a new `Ipv4Subnet` with the given base address and netmask prefix length.
    pub fn new(base_addr: Ipv4Addr, netmask_bits: u8) -> Result<Ipv4Subnet, Ipv4SubnetNewError> {
        let masked = match base_addr.apply_netmask(netmask_bits) {
            Ok(masked) => masked,
            Err(ApplyNetmaskError::OutOfRange(n, _))
                => return Err(Ipv4SubnetNewError::NetmaskOutOfRange { netmask_bits: n }),
        };
        if masked != base_addr {
            return Err(Ipv4SubnetNewError::TrailingOnes {
                base_addr: base_addr,
                netmask_bits: netmask_bits,
            });
        }
        Ok(Ipv4Subnet {
            base_addr: base_addr,
            netmask_bits: netmask_bits,
        })
    }

    /// Test whether the subnet contains the given IP address.
    pub fn contains(&self, addr: &Ipv4Addr) -> bool {
        unwrap_result!(addr.apply_netmask(self.netmask_bits)) == self.base_addr
    }

    /// Returns the loopback subnet 127.0.0.0/8
    pub fn loopback() -> Ipv4Subnet {
        Ipv4Subnet {
            base_addr: Ipv4Addr::new(127, 0, 0, 0),
            netmask_bits: 8,
        }
    }

    /// Returns the link-local subnet 169.254.0.0/16
    pub fn link_local() -> Ipv4Subnet {
        Ipv4Subnet {
            base_addr: Ipv4Addr::new(169, 254, 0, 0),
            netmask_bits: 16,
        }
    }

    /// Returns the multicast subnet 224.0.0.0/4
    pub fn multicast() -> Ipv4Subnet {
        Ipv4Subnet {
            base_addr: Ipv4Addr::new(224, 0, 0, 0),
            netmask_bits: 4,
        }
    }
}

impl FromStr for Ipv4Subnet {
    type Err = Ipv4SubnetParseError;

    fn from_str(s: &str) -> Result<Ipv4Subnet, Ipv4SubnetParseError> {
        let mut iter = s.split('/');
        match (iter.next(), iter.next(), iter.next()) {
            (Some(addr), Some(netmask_bits), None) => {
                let addr: Ipv4Addr = match addr.parse() {
                    Ok(addr) => addr,
                    Err(e) => return Err(Ipv4SubnetParseError::ParseAddress { err: e }),
                };
                let netmask_bits = match netmask_bits.parse() {
                    Ok(netmask_bits) => netmask_bits,
                    Err(e) => return Err(Ipv4SubnetParseError::ParseNetmask { err: e }),
                };
                let subnet = match Ipv4Subnet::new(addr, netmask_bits) {
                    Ok(subnet) => subnet,
                    Err(e) => return Err(Ipv4SubnetParseError::InvalidSubnet { err: e }),
                };
                Ok(subnet)
            },
            _ => Err(Ipv4SubnetParseError::InvalidFormat),
        }
    }
}

impl fmt::Display for Ipv4Subnet {
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
        let as_str = "192.168.0.0/16";
        let subnet: Ipv4Subnet = unwrap_result!(as_str.parse());
        let displayed = format!("{}", subnet);
        assert_eq!(displayed, as_str);
    }

    #[test]
    fn parse_bad_subnets() {
        match Ipv4Subnet::from_str("foo") {
            Err(Ipv4SubnetParseError::InvalidFormat) => (),
            x => panic!("Unexpected result {:?}", x),
        };

        match Ipv4Subnet::from_str("foo/bar/baz") {
            Err(Ipv4SubnetParseError::InvalidFormat) => (),
            x => panic!("Unexpected result {:?}", x),
        };

        match Ipv4Subnet::from_str("all_zeros/23") {
            Err(Ipv4SubnetParseError::InvalidFormat) => (),
            Err(Ipv4SubnetParseError::ParseAddress { .. }) => (),
            x => panic!("Unexpected result {:?}", x),
        };

        match Ipv4Subnet::from_str("0.0.0.0/twenty_three") {
            Err(Ipv4SubnetParseError::InvalidFormat) => (),
            Err(Ipv4SubnetParseError::ParseNetmask { .. }) => (),
            x => panic!("Unexpected result {:?}", x),
        };

        match Ipv4Subnet::from_str("255.255.255.255/24") {
            Err(Ipv4SubnetParseError::InvalidSubnet {
                err: Ipv4SubnetNewError::TrailingOnes { .. }
            }) => (),
            x => panic!("Unexpected result {:?}", x),
        };

        match Ipv4Subnet::from_str("255.255.255.255/33") {
            Err(Ipv4SubnetParseError::InvalidSubnet {
                err: Ipv4SubnetNewError::NetmaskOutOfRange { .. }
            }) => (),
            x => panic!("Unexpected result {:?}", x),
        };
    }
}

