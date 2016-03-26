use std::net::{Ipv4Addr, Ipv6Addr};

quick_error! {
    /// Errors returned by `ApplyNetmask::apply_netmask`
    #[derive(Debug)]
    pub enum ApplyNetmaskError {
        /// The netmask_bits argument was out of range.
        OutOfRange(netmask_bits: u8, addr_len: u8) {
            description("netmask_bits argument out of range")
            display("netmask_bits argument out of range. netmask_bits == {}. netmask_bits must be <= {}", netmask_bits, addr_len)
        }
    }
}

/// Address types that a netmask can be applied to.
pub trait ApplyNetmask: Sized {
    /// Retain the first `netmask_bits` bits of the address, setting all other bits to zero.
    fn apply_netmask(self, netmask_bits: u8) -> Result<Self, ApplyNetmaskError>;
}

impl ApplyNetmask for Ipv4Addr {
    fn apply_netmask(self, netmask_bits: u8) -> Result<Ipv4Addr, ApplyNetmaskError> {
        if netmask_bits > 32 {
            return Err(ApplyNetmaskError::OutOfRange(netmask_bits, 32));
        }

        let octets = self.octets();
        let x0 = octets[0] & ((0xffu32 << 8u8.saturating_sub(netmask_bits.saturating_sub( 0))) as u8);
        let x1 = octets[1] & ((0xffu32 << 8u8.saturating_sub(netmask_bits.saturating_sub( 8))) as u8);
        let x2 = octets[2] & ((0xffu32 << 8u8.saturating_sub(netmask_bits.saturating_sub(16))) as u8);
        let x3 = octets[3] & ((0xffu32 << 8u8.saturating_sub(netmask_bits.saturating_sub(24))) as u8);
        Ok(Ipv4Addr::new(x0, x1, x2, x3))
    }
}

impl ApplyNetmask for Ipv6Addr {
    fn apply_netmask(self, netmask_bits: u8) -> Result<Ipv6Addr, ApplyNetmaskError> {
        if netmask_bits > 128 {
            return Err(ApplyNetmaskError::OutOfRange(netmask_bits, 128));
        }

        let segments = self.segments();
        let x0 = segments[0] & ((0xffffu32 << 16u8.saturating_sub(netmask_bits.saturating_sub(  0))) as u16);
        let x1 = segments[1] & ((0xffffu32 << 16u8.saturating_sub(netmask_bits.saturating_sub( 16))) as u16);
        let x2 = segments[2] & ((0xffffu32 << 16u8.saturating_sub(netmask_bits.saturating_sub( 32))) as u16);
        let x3 = segments[3] & ((0xffffu32 << 16u8.saturating_sub(netmask_bits.saturating_sub( 48))) as u16);
        let x4 = segments[4] & ((0xffffu32 << 16u8.saturating_sub(netmask_bits.saturating_sub( 64))) as u16);
        let x5 = segments[5] & ((0xffffu32 << 16u8.saturating_sub(netmask_bits.saturating_sub( 80))) as u16);
        let x6 = segments[6] & ((0xffffu32 << 16u8.saturating_sub(netmask_bits.saturating_sub( 96))) as u16);
        let x7 = segments[7] & ((0xffffu32 << 16u8.saturating_sub(netmask_bits.saturating_sub(112))) as u16);
        Ok(Ipv6Addr::new(x0, x1, x2, x3, x4, x5, x6, x7))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn apply_netmask_v4() {
        let addr: Ipv4Addr = unwrap_result!("170.170.170.170".parse());

        let masked = unwrap_result!(addr.apply_netmask(0));
        assert_eq!(masked, unwrap_result!("0.0.0.0".parse()));

        let masked = unwrap_result!(addr.apply_netmask(6));
        assert_eq!(masked, unwrap_result!("168.0.0.0".parse()));

        let masked = unwrap_result!(addr.apply_netmask(14));
        assert_eq!(masked, unwrap_result!("170.168.0.0".parse()));

        let masked = unwrap_result!(addr.apply_netmask(22));
        assert_eq!(masked, unwrap_result!("170.170.168.0".parse()));

        let masked = unwrap_result!(addr.apply_netmask(30));
        assert_eq!(masked, unwrap_result!("170.170.170.168".parse()));

        let masked = unwrap_result!(addr.apply_netmask(32));
        assert_eq!(masked, unwrap_result!("170.170.170.170".parse()));
    }

    #[test]
    #[should_panic]
    fn apply_bad_netmask_v4() {
        let addr: Ipv4Addr = unwrap_result!("170.170.170.170".parse());
        let _ = unwrap_result!(addr.apply_netmask(33));
    }

    #[test]
    fn apply_netmask_v6() {
        let addr: Ipv6Addr = unwrap_result!("aaaa:aaaa:aaaa:aaaa:aaaa:aaaa:aaaa:aaaa".parse());

        let masked = unwrap_result!(addr.apply_netmask(0));
        assert_eq!(masked, unwrap_result!("0000:0000:0000:0000:0000:0000:0000:0000".parse()));

        let masked = unwrap_result!(addr.apply_netmask(14));
        assert_eq!(masked, unwrap_result!("aaa8:0000:0000:0000:0000:0000:0000:0000".parse()));

        let masked = unwrap_result!(addr.apply_netmask(30));
        assert_eq!(masked, unwrap_result!("aaaa:aaa8:0000:0000:0000:0000:0000:0000".parse()));

        let masked = unwrap_result!(addr.apply_netmask(46));
        assert_eq!(masked, unwrap_result!("aaaa:aaaa:aaa8:0000:0000:0000:0000:0000".parse()));

        let masked = unwrap_result!(addr.apply_netmask(62));
        assert_eq!(masked, unwrap_result!("aaaa:aaaa:aaaa:aaa8:0000:0000:0000:0000".parse()));

        let masked = unwrap_result!(addr.apply_netmask(78));
        assert_eq!(masked, unwrap_result!("aaaa:aaaa:aaaa:aaaa:aaa8:0000:0000:0000".parse()));

        let masked = unwrap_result!(addr.apply_netmask(94));
        assert_eq!(masked, unwrap_result!("aaaa:aaaa:aaaa:aaaa:aaaa:aaa8:0000:0000".parse()));

        let masked = unwrap_result!(addr.apply_netmask(110));
        assert_eq!(masked, unwrap_result!("aaaa:aaaa:aaaa:aaaa:aaaa:aaaa:aaa8:0000".parse()));

        let masked = unwrap_result!(addr.apply_netmask(126));
        assert_eq!(masked, unwrap_result!("aaaa:aaaa:aaaa:aaaa:aaaa:aaaa:aaaa:aaa8".parse()));

        let masked = unwrap_result!(addr.apply_netmask(128));
        assert_eq!(masked, unwrap_result!("aaaa:aaaa:aaaa:aaaa:aaaa:aaaa:aaaa:aaaa".parse()));
    }

    #[test]
    #[should_panic]
    fn apply_bad_netmask_v6() {
        let addr: Ipv6Addr = unwrap_result!("::".parse());
        let _ = unwrap_result!(addr.apply_netmask(129));
    }
}

