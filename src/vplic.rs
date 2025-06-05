use crate::consts::*;
use spin::Mutex;

const BITS_PER_WORD: usize = 32;

pub struct VPlic {
    pub emulated_base_addr: usize,
    pub inner: Mutex<VPlicInner>,
}

pub struct VPlicInner {
    pub prio: [u32; PLIC_MAX_IRQ + 1],
    pub pending: [u32; (PLIC_MAX_IRQ + BITS_PER_WORD) / BITS_PER_WORD],
    pub enable: [[u32; (PLIC_MAX_IRQ + BITS_PER_WORD) / BITS_PER_WORD]; MAX_CONTEXTS],
    pub threshold: [u32; MAX_CONTEXTS],
    pub claim: [u32; MAX_CONTEXTS],
}

impl VPlic {
    pub fn new(emulated_base_addr: usize) -> Self {
        Self {
            emulated_base_addr,
            inner: Mutex::new(VPlicInner {
                prio: [0; PLIC_MAX_IRQ + 1],
                pending: [0; (PLIC_MAX_IRQ + BITS_PER_WORD) / BITS_PER_WORD],
                enable: [[0; (PLIC_MAX_IRQ + BITS_PER_WORD) / BITS_PER_WORD]; MAX_CONTEXTS],
                threshold: [0; MAX_CONTEXTS],
                claim: [0; MAX_CONTEXTS],
            }),
        }
    }

    fn index_and_bit(irq: usize) -> (usize, usize) {
        (irq / BITS_PER_WORD, irq % BITS_PER_WORD)
    }

    pub fn get_prio(&self, irq: usize) -> u32 {
        let inner = self.inner.lock();
        inner.prio[irq]
    }

    pub fn set_prio(&self, irq: usize, prio: u32) {
        let mut inner = self.inner.lock();
        inner.prio[irq] = prio;
    }

    pub fn get_pending(&self, irq: usize) -> bool {
        let (index, bit) = Self::index_and_bit(irq);
        let inner = self.inner.lock();
        (inner.pending[index] & (1 << bit)) != 0
    }

    pub fn set_pending(&self, irq: usize) {
        let (index, bit) = Self::index_and_bit(irq);
        let mut inner = self.inner.lock();
        inner.pending[index] |= 1 << bit;
    }

    pub fn get_pending_word(&self, word: usize) -> u32 {
        let inner = self.inner.lock();
        inner.pending[word]
    }

    pub fn set_pending_word(&self, word: usize, val: u32) {
        let mut inner = self.inner.lock();
        inner.pending[word] = val;
    }

    pub fn clear_pending(&self, irq: usize) {
        let (index, bit) = Self::index_and_bit(irq);
        let mut inner = self.inner.lock();
        inner.pending[index] &= !(1 << bit);
    }

    pub fn get_enable(&self, context: usize, irq: usize) -> bool {
        let (index, bit) = Self::index_and_bit(irq);
        let inner = self.inner.lock();
        (inner.enable[context][index] & (1 << bit)) != 0
    }

    pub fn set_enable_word(&self, context: usize, word: usize, val: u32) {
        let mut inner = self.inner.lock();
        inner.enable[context][word] = val;
    }

    pub fn get_enable_word(&self, context: usize, word: usize) -> u32 {
        let inner = self.inner.lock();
        inner.enable[context][word]
    }

    pub fn get_threshold(&self, context: usize) -> u32 {
        let inner = self.inner.lock();
        inner.threshold[context]
    }

    pub fn set_threshold(&self, context: usize, threshold: u32) {
        let mut inner = self.inner.lock();
        inner.threshold[context] = threshold;
    }

    pub fn get_claim(&self, context: usize) -> u32 {
        let inner = self.inner.lock();
        inner.claim[context]
    }

    pub fn set_claim(&self, context: usize, claim: u32) {
        let mut inner = self.inner.lock();
        inner.claim[context] = claim;
    }

    pub fn claim_irq(&self, context: usize) -> Option<usize> {
        let threshold = self.get_threshold(context);
        let mut best_irq = None;
        let mut best_prio = 0;

        for irq in 1..=PLIC_MAX_IRQ {
            let prio = self.get_prio(irq);
            if prio > threshold && prio > best_prio && self.get_pending(irq) && self.get_enable(context, irq) {
                best_irq = Some(irq);
                best_prio = prio;
            }
        }

        if let Some(irq) = best_irq {
            self.clear_pending(irq);
            self.set_claim(context, irq as u32);
        }

        best_irq
    }

    pub fn complete_irq(&self, context: usize, _irq: usize) {
        self.set_claim(context, 0);
    }
}