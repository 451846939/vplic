use axaddrspace::device::AccessWidth;
use axaddrspace::{GuestPhysAddr, GuestPhysAddrRange};
use axdevice_base::{BaseDeviceOps, EmuDeviceType};
use axerrno::AxResult;

use crate::consts::{
    CONTEXT_ENABLE_STRIDE, CONTEXT_STRIDE, MAX_HARTS, PLIC_ENABLE_BEGIN, PLIC_ENABLE_END, PLIC_MAX_IRQ, PLIC_PENDING_BEGIN, PLIC_PENDING_END, PLIC_PRIO_BEGIN, PLIC_PRIO_END, PLIC_THRESHOLD_CLAIM_BEGIN, PLIC_THRESHOLD_CLAIM_END
};
use crate::vplic::VPlic;

impl BaseDeviceOps<GuestPhysAddrRange> for VPlic {
    fn emu_type(&self) -> EmuDeviceType {
        todo!()
    }

    fn address_range(&self) -> GuestPhysAddrRange {
        GuestPhysAddrRange::from_start_size(
            self.emulated_base_addr.into(),
            PLIC_THRESHOLD_CLAIM_END - PLIC_PRIO_BEGIN,
        )
    }

    fn handle_read(&self, addr: GuestPhysAddr, width: AccessWidth) -> AxResult<usize> {
        let offset = addr.as_usize() - self.emulated_base_addr;

        let val = match width {
            AccessWidth::Dword => {
                let mut inner = self.inner.lock();
                if (PLIC_PRIO_BEGIN..=PLIC_PRIO_END).contains(&offset) {
                    let irq = offset / 4;
                    inner.prio[irq] as usize
                } else if (PLIC_PENDING_BEGIN..=PLIC_PENDING_END).contains(&offset) {
                    let word = (offset - PLIC_PENDING_BEGIN) / 4;
                    inner.pending[word] as usize
                } else if (PLIC_ENABLE_BEGIN..=PLIC_ENABLE_END).contains(&offset) {
                    let ctx = (offset - PLIC_ENABLE_BEGIN) / CONTEXT_ENABLE_STRIDE;
                    let word = ((offset - PLIC_ENABLE_BEGIN) % CONTEXT_ENABLE_STRIDE) / 4;
                    inner.enable[ctx][word] as usize
                } else if (PLIC_THRESHOLD_CLAIM_BEGIN..=PLIC_THRESHOLD_CLAIM_END).contains(&offset) {
                    let ctx = (offset - PLIC_THRESHOLD_CLAIM_BEGIN) / CONTEXT_STRIDE;
                    let local = (offset - PLIC_THRESHOLD_CLAIM_BEGIN) % CONTEXT_STRIDE;
                    match local {
                        0 => inner.threshold[ctx] as usize,
                        4 => inner.claim[ctx] as usize,
                        _ => 0,
                    }
                } else {
                    0
                }
            }
            _ => 0,
        };

        Ok(val)
    }

    fn handle_write(&self, addr: GuestPhysAddr, width: AccessWidth, val: usize) -> AxResult {
        let offset = addr.as_usize() - self.emulated_base_addr;

        if width != AccessWidth::Dword {
            return Ok(());
        }

        let mut inner = self.inner.lock();
        if (PLIC_PRIO_BEGIN..=PLIC_PRIO_END).contains(&offset) {
            let irq = offset / 4;
            inner.prio[irq] = val as u32;
        } else if (PLIC_ENABLE_BEGIN..=PLIC_ENABLE_END).contains(&offset) {
            let ctx = (offset - PLIC_ENABLE_BEGIN) / CONTEXT_ENABLE_STRIDE;
            let word = ((offset - PLIC_ENABLE_BEGIN) % CONTEXT_ENABLE_STRIDE) / 4;
            inner.enable[ctx][word] = val as u32;
        } else if (PLIC_THRESHOLD_CLAIM_BEGIN..=PLIC_THRESHOLD_CLAIM_END).contains(&offset) {
            let ctx = (offset - PLIC_THRESHOLD_CLAIM_BEGIN) / CONTEXT_STRIDE;
            let local = (offset - PLIC_THRESHOLD_CLAIM_BEGIN) % CONTEXT_STRIDE;
            match local {
                0 => inner.threshold[ctx] = val as u32,
                4 => inner.claim[ctx] = 0, // complete_irq
                _ => {}
            }
        }

        Ok(())
    }
}