use crate::consts::*;
use spin::Mutex;
use alloc::vec::Vec;
use alloc::vec;

const BITS_PER_WORD: usize = 32;

pub struct Context {
    pub enable: Vec<u32>,
    pub threshold: u32,
    pub claim: u32,
}

pub struct VPlic {
    pub emulated_base_addr: usize,
    pub max_harts: usize,
    pub max_contexts: usize,
    pub inner: Mutex<VPlicInner>,
}

pub struct VPlicInner {
    pub prio: Vec<u32>,
    pub pending: Vec<u32>,
    pub contexts: Vec<Context>,
}

impl VPlic {

    pub fn new_with_base(emulated_base_addr: usize) -> Self {
        Self::new(emulated_base_addr, MAX_HARTS)
    }
    
    pub fn new(emulated_base_addr: usize, max_harts: usize) -> Self {
        let max_contexts = max_harts * CONTEXT_PER_HART;
        let prio = vec![0; PLIC_MAX_IRQ + 1];
        let pending = vec![0; (PLIC_MAX_IRQ + BITS_PER_WORD) / BITS_PER_WORD];
        let enable_template = vec![0; (PLIC_MAX_IRQ + BITS_PER_WORD) / BITS_PER_WORD];
        let contexts = (0..max_contexts)
            .map(|_| Context {
                enable: enable_template.clone(),
                threshold: 0,
                claim: 0,
            })
            .collect();
        Self {
            emulated_base_addr,
            max_harts,
            max_contexts,
            inner: Mutex::new(VPlicInner {
                prio,
                pending,
                contexts,
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
        if word >= inner.pending.len() {
            return 0;
        }
        inner.pending[word]
    }

    pub fn set_pending_word(&self, word: usize, val: u32) {
        let mut inner = self.inner.lock();
        if word >= inner.pending.len() {
            return;
        }
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
        (inner.contexts[context].enable[index] & (1 << bit)) != 0
    }

    pub fn set_enable_word(&self, context: usize, word: usize, val: u32) {
        let mut inner = self.inner.lock();
        if context >= inner.contexts.len() || word >= inner.contexts[context].enable.len() {
            return;
        }
        inner.contexts[context].enable[word] = val;
    }

    pub fn get_enable_word(&self, context: usize, word: usize) -> u32 {
        let inner = self.inner.lock();
        if context >= inner.contexts.len() || word >= inner.contexts[context].enable.len() {
            return 0;
        }
        inner.contexts[context].enable[word]
    }

    pub fn get_threshold(&self, context: usize) -> u32 {
        let inner = self.inner.lock();
        inner.contexts[context].threshold
    }

    pub fn set_threshold(&self, context: usize, threshold: u32) {
        let mut inner = self.inner.lock();
        inner.contexts[context].threshold = threshold;
    }

    pub fn get_claim(&self, context: usize) -> u32 {
        let inner = self.inner.lock();
        inner.contexts[context].claim
    }

    pub fn set_claim(&self, context: usize, claim: u32) {
        let mut inner = self.inner.lock();
        inner.contexts[context].claim = claim;
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

