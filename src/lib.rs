#![no_std]

mod devops_impl;
pub mod vplic;

mod consts;




#[cfg(test)]
mod more_tests {
    use axaddrspace::device::AccessWidth;
    use axaddrspace::GuestPhysAddr;
    use axdevice_base::BaseDeviceOps;
    use crate::consts::{CONTEXT_ENABLE_STRIDE, CONTEXT_STRIDE, MAX_HARTS, PLIC_ENABLE_BEGIN, PLIC_MAX_IRQ, PLIC_PENDING_BEGIN, PLIC_PRIO_BEGIN, PLIC_THRESHOLD_CLAIM_BEGIN};

    use super::vplic::*;

    const BASE_ADDR: usize = 0x1000;

    fn setup_vplic() -> VPlic {
        VPlic::new(BASE_ADDR)
    }
    fn setup_vplic_with_base(base: usize) -> VPlic {
        VPlic::new(base)
    }
    #[test]
    fn test_pending_multiple_bits() {
        let vplic = setup_vplic();
        vplic.set_pending(1);
        vplic.set_pending(31);
        vplic.set_pending(32);
        assert!(vplic.get_pending(1));
        assert!(vplic.get_pending(31));
        assert!(vplic.get_pending(32));
        vplic.clear_pending(31);
        assert!(!vplic.get_pending(31));
        assert!(vplic.get_pending(1));
        assert!(vplic.get_pending(32));
    }

    #[test]
    fn test_enable_and_pending_combination() {
        let vplic = setup_vplic();
        vplic.set_prio(10, 20);
        vplic.set_pending(10);
        vplic.set_enable_word(0, 0, 0);
        vplic.set_threshold(0, 5);
        // Not enabled, should not claim
        assert_eq!(vplic.claim_irq(0), None);
        // Enable and try again
        vplic.set_enable_word(0, 0, 1 << 10);
        assert_eq!(vplic.claim_irq(0), Some(10));
    }

    #[test]
    fn test_claim_irq_with_equal_priority() {
        let vplic = setup_vplic();
        vplic.set_prio(2, 10);
        vplic.set_prio(3, 10);
        vplic.set_pending(2);
        vplic.set_pending(3);
        vplic.set_enable_word(0, 0, (1 << 2) | (1 << 3));
        vplic.set_threshold(0, 5);
        let first = vplic.claim_irq(0);
        let second = vplic.claim_irq(0);
        assert!(first == Some(2) || first == Some(3));
        assert!(second == Some(2) || second == Some(3));
        assert_ne!(first, second);
    }

    #[test]
    fn test_claim_irq_with_zero_priority() {
        let vplic = setup_vplic();
        vplic.set_prio(4, 0);
        vplic.set_pending(4);
        vplic.set_enable_word(0, 0, 1 << 4);
        vplic.set_threshold(0, 0);
        assert_eq!(vplic.claim_irq(0), None);
    }

    #[test]
    fn test_set_and_get_enable_word_out_of_bounds() {
        let vplic = setup_vplic();
        // Should not panic, but also not set anything meaningful
        let max_word = (PLIC_MAX_IRQ + 32) / 32;
        vplic.set_enable_word(0, max_word, 0xDEADBEEF);
        // Out of bounds, so get_enable_word should not return the set value
        assert_ne!(vplic.get_enable_word(0, max_word), 0xDEADBEEF);
    }

    #[test]
    fn test_set_and_get_pending_word_out_of_bounds() {
        let vplic = setup_vplic();
        let max_word = (PLIC_MAX_IRQ + 32) / 32;
        vplic.set_pending_word(max_word, 0xCAFEBABE);
        assert_ne!(vplic.get_pending_word(max_word), 0xCAFEBABE);
    }

    #[test]
    fn test_multiple_contexts_claim_independence() {
        let vplic = setup_vplic();
        vplic.set_prio(7, 9);
        vplic.set_pending(7);
        vplic.set_enable_word(0, 0, 1 << 7);
        vplic.set_enable_word(1, 0, 1 << 7);
        vplic.set_threshold(0, 5);
        vplic.set_threshold(1, 8);
        assert_eq!(vplic.claim_irq(0), Some(7));
        // Pending is cleared for all, so context 1 should not claim
        assert_eq!(vplic.claim_irq(1), None);
    }


    #[test]
    fn handle_read_write_prio_happy_path() {
        let base = 0x1000_0000;
        let vplic = setup_vplic_with_base(base);
        let irq = 3;
        let addr = GuestPhysAddr::from(base + PLIC_PRIO_BEGIN + irq * 4);

        vplic.handle_write(addr, AccessWidth::Dword, 0x55).unwrap();
        let val = vplic.handle_read(addr, AccessWidth::Dword).unwrap();
        assert_eq!(val, 0x55);
    }

    #[test]
    fn handle_read_write_enable_happy_path() {
        let base = 0x1000_0000;
        let vplic = setup_vplic_with_base(base);
        let ctx = 1;
        let word = 2;
        let addr = GuestPhysAddr::from(
            base + PLIC_ENABLE_BEGIN + ctx * CONTEXT_ENABLE_STRIDE + word * 4,
        );

        vplic.handle_write(addr, AccessWidth::Dword, 0xAA).unwrap();
        let val = vplic.handle_read(addr, AccessWidth::Dword).unwrap();
        assert_eq!(val, 0xAA);
    }

    #[test]
    fn handle_read_write_threshold_claim_happy_path() {
        let base = 0x1000_0000;
        let vplic = setup_vplic_with_base(base);
        let ctx = 0;
        let threshold_addr = GuestPhysAddr::from(
            base + PLIC_THRESHOLD_CLAIM_BEGIN + ctx * CONTEXT_STRIDE,
        );
        let claim_addr = GuestPhysAddr::from(
            base + PLIC_THRESHOLD_CLAIM_BEGIN + ctx * CONTEXT_STRIDE + 4,
        );

        vplic.handle_write(threshold_addr, AccessWidth::Dword, 0x5).unwrap();
        let val = vplic.handle_read(threshold_addr, AccessWidth::Dword).unwrap();
        assert_eq!(val, 0x5);

        vplic.handle_write(claim_addr, AccessWidth::Dword, 0x7).unwrap();
        let val = vplic.handle_read(claim_addr, AccessWidth::Dword).unwrap();
        assert_eq!(val, 0);
    }

    #[test]
    fn handle_read_pending_word_edge_case_out_of_bounds() {
        let base = 0x1000_0000;
        let vplic = setup_vplic_with_base(base);
        let word = (PLIC_MAX_IRQ + 32) / 32;
        let addr = GuestPhysAddr::from(base + PLIC_PENDING_BEGIN + word * 4);

        let val = vplic.handle_read(addr, AccessWidth::Dword).unwrap();
        assert_eq!(val, 0);
    }

    #[test]
    fn handle_write_enable_word_edge_case_out_of_bounds() {
        let base = 0x1000_0000;
        let vplic = setup_vplic_with_base(base);
        let ctx = MAX_HARTS;
        let word = (PLIC_MAX_IRQ + 32) / 32;
        let addr = GuestPhysAddr::from(
            base + PLIC_ENABLE_BEGIN + ctx * CONTEXT_ENABLE_STRIDE + word * 4,
        );

        vplic.handle_write(addr, AccessWidth::Dword, 0xDEAD_BEEF).unwrap();
        let val = vplic.handle_read(addr, AccessWidth::Dword).unwrap();
        assert_eq!(val, 0);
    }

    #[test]
    fn handle_read_write_with_non_dword_width_returns_zero() {
        let base = 0x1000_0000;
        let vplic = setup_vplic_with_base(base);
        let addr = GuestPhysAddr::from(base + PLIC_PRIO_BEGIN);

        let val = vplic.handle_read(addr, AccessWidth::Byte).unwrap();
        assert_eq!(val, 0);

        vplic.handle_write(addr, AccessWidth::Word, 0x1234).unwrap();
        let val = vplic.handle_read(addr, AccessWidth::Dword).unwrap();
        assert_eq!(val, 0);
    }
}
