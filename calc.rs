use crate::*;
use std::{hash::Hash, thread};

impl<T: Clone + Eq + Hash> Config<T> {
    /// Calculates probabilities of existences of table items in each picking result
    /// of length `pick_amount`. In non-repetitive mode, the multi-thread tree algorithm
    /// may be used.
    ///
    /// Preorder traversal is performed in each thread. Something like depth-first or
    /// postorder algorithm may achieve higher precision (consider the error produced
    /// while adding a small floating-point number to a much larger number, which is
    /// the current way), but it will probably increase complexity and memory usage.
    ///
    /// TODO: figure out why its single-thread performance is about 7% slower than
    /// the single-thread C++ version compiled with `clang++` without `-march=native`,
    /// and unsafe operations can't make this Rust program faster.
    /// It is faster than the C++ version compiled with GCC, though.
    pub fn calc_probabilities(&self, pick_amount: usize) -> Result<Table<T>, Error> {
        if pick_amount == 0 {
            return Ok(self.table.keys().map(|k| (k.clone(), 0.)).collect());
        }

        if !self.repetitive {
            if pick_amount > self.table.len() {
                return Err(Error::InvalidAmount);
            }
            if pick_amount == self.table.len() {
                return Ok(self.table.keys().map(|k| (k.clone(), 1.)).collect());
            }
            if self.is_fair() {
                let prob = (pick_amount as f64) / (self.table.len() as f64);
                return Ok(self.table.keys().map(|k| (k.clone(), prob)).collect());
            }
        }

        // map to values within range 0. ~ 1.
        let table: Vec<_> = {
            let raw_table = self.vec_table()?;
            let grid_width: f64 = raw_table.iter().map(|(_, v)| v).sum();
            raw_table
                .into_iter()
                .map(|(k, v)| (k, v / grid_width))
                .collect()
        };

        if pick_amount == 1 {
            return Ok(table.into_iter().collect());
        }
        if self.repetitive {
            return Ok(table
                .into_iter()
                .map(|(k, v)| (k, 1. - (1. - v).powi(pick_amount as i32)))
                .collect());
        }

        // -------- calc for general non-repetitive cases --------

        let table_val: Vec<_> = table.iter().map(|(_, v)| *v).collect();
        let mut calc_result = table.clone();

        let cnt_threads = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .max(table.len());
        let cnt_calc_groups = table.len().div_ceil(cnt_threads);
        let mut calc_groups = Vec::with_capacity(cnt_calc_groups);
        let mut table_picked = vec![false; table.len()];
        for i in 0..cnt_calc_groups {
            let mut calcs = Vec::with_capacity(cnt_threads);
            for j in 0..cnt_threads {
                let i_th = i * cnt_threads + j;
                if i_th >= table.len() {
                    break;
                }
                table_picked[i_th] = true;
                let calc_stack =
                    CalcStack::new(table_val.clone(), pick_amount, table_picked.clone());
                calcs.push((i_th, Some(calc_stack)));
                table_picked[i_th] = false;
            }
            calc_groups.push(calcs);
        }

        for group in calc_groups.into_iter() {
            let mut thread_hdls = Vec::with_capacity(cnt_threads);
            for (i, mut calc) in group.into_iter() {
                let calc = calc.take().unwrap();
                thread_hdls.push(thread::spawn(move || (i, calc.calc())));
            }
            for hdl in thread_hdls {
                let (i_th, sub_result) = hdl.join().map_err(|_| Error::ThreadError)?;
                for (i, &sub_prob) in sub_result.iter().enumerate() {
                    calc_result[i].1 += table_val[i_th] * sub_prob;
                }
            }
        }

        Ok(calc_result.into_iter().collect())
    }
}

#[derive(Clone, Debug)]
struct CalcStack {
    // Do not modify table and stack_size
    table: Vec<f64>,   // size: table.len()
    stack_size: usize, // = pick_amount - initial amount of picked items

    stack: Vec<(usize, f64)>, // maximum size: stack_size
    table_picked: Vec<bool>,  // size: table.len()
    rem_width: f64,           // current sum of grid cell widths of unpicked items

    result: Vec<f64>, // size: table.len()
}

impl CalcStack {
    // Do not construct it by other means
    // pick_amount includes items that were already picked
    fn new(table: Vec<f64>, pick_amount: usize, table_picked: Vec<bool>) -> Self {
        assert!(table.len() == table_picked.len());
        let table_len = table.len();

        let mut stack_size = pick_amount;
        let mut rem_width = 0.;
        for (i, &picked) in table_picked.iter().enumerate() {
            if !picked {
                rem_width += table[i];
            } else {
                if stack_size == 0 {
                    break; // something is wrong
                }
                stack_size -= 1;
            }
        }

        Self {
            table,
            stack: Vec::with_capacity(stack_size),
            stack_size,
            table_picked,
            rem_width,
            result: vec![0.; table_len],
        }
    }

    fn calc(mut self) -> Vec<f64> {
        loop {
            let mut got_next = self.go_down();
            if !got_next {
                got_next = self.go_right();
            }
            if !got_next {
                got_next = self.go_up_right();
            }
            if !got_next {
                return self.result;
            }
        }
    }

    #[inline(always)]
    fn go_down(&mut self) -> bool {
        if self.stack.len() >= self.stack_size {
            return false;
        }

        let i_next;
        if let Some(i) = self.next_unpicked(0) {
            i_next = i;
        } else {
            return false;
        };

        let parent_prob = self.stack.last().map(|t| t.1).unwrap_or(1.);
        let prob = parent_prob * self.table[i_next] / self.rem_width;

        self.stack.push((i_next, prob));
        self.table_picked[i_next] = true;
        self.rem_width -= self.table[i_next];
        self.result[i_next] += prob;
        true
    }

    #[inline(always)]
    fn go_right(&mut self) -> bool {
        let i_prev;
        if let Some(&(i, _)) = self.stack.last() {
            i_prev = i;
        } else {
            return false;
        }

        let i_next;
        if let Some(i) = self.next_unpicked(i_prev + 1) {
            i_next = i;
        } else {
            return false;
        };

        let stack_level = self.stack.len();
        let parent_prob = if stack_level >= 2 {
            self.stack[stack_level - 2].1
        } else {
            1.
        };
        let parent_rem_width = self.rem_width + self.table[i_prev];
        let prob = parent_prob * self.table[i_next] / parent_rem_width;

        *self.stack.last_mut().unwrap() = (i_next, prob);
        self.table_picked[i_prev] = false;
        self.table_picked[i_next] = true;
        self.rem_width = parent_rem_width - self.table[i_next];
        self.result[i_next] += prob;
        true
    }

    fn go_up_right(&mut self) -> bool {
        while let Some((i_prev, _)) = self.stack.pop() {
            self.table_picked[i_prev] = false;
            self.rem_width += self.table[i_prev];
            if self.go_right() {
                return true;
            }
        }
        false
    }

    fn next_unpicked(&self, least_index: usize) -> Option<usize> {
        self.table_picked
            .iter()
            .enumerate()
            .skip(least_index)
            .find(|(_, &picked)| !picked)
            .map(|(i, _)| i)
    }
}
